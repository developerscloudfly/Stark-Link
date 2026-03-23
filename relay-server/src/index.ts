import { createServer, IncomingMessage, ServerResponse } from "http";
import { WebSocketServer, WebSocket } from "ws";
import { v4 as uuidv4 } from "uuid";
import { Room, generateRoomCode } from "./room";
import { ClientInfo, RelayMessage, ServerMessage } from "./types";

// ── Configuration ──────────────────────────────────────────────────────

const PORT = parseInt(process.env.PORT || "8080", 10);
const HEARTBEAT_INTERVAL_MS = 30_000;
const HEARTBEAT_TIMEOUT_MS = 10_000;

// ── State ──────────────────────────────────────────────────────────────

const rooms = new Map<string, Room>();
const clients = new Map<string, ClientInfo>();

/** Set of all active room codes, used for collision-free generation. */
function getActiveRoomCodes(): Set<string> {
  return new Set(rooms.keys());
}

// ── HTTP Server (health endpoint) ──────────────────────────────────────

const server = createServer((req: IncomingMessage, res: ServerResponse) => {
  if (req.method === "GET" && req.url === "/health") {
    const body = JSON.stringify({
      status: "ok",
      uptime: process.uptime(),
      rooms: rooms.size,
      clients: clients.size,
      timestamp: new Date().toISOString(),
    });
    res.writeHead(200, {
      "Content-Type": "application/json",
      "Cache-Control": "no-cache",
    });
    res.end(body);
    return;
  }

  res.writeHead(404, { "Content-Type": "application/json" });
  res.end(JSON.stringify({ error: "Not found" }));
});

// ── WebSocket Server ───────────────────────────────────────────────────

const wss = new WebSocketServer({ server });

wss.on("connection", (ws: WebSocket) => {
  const clientId = uuidv4();
  const client: ClientInfo = {
    id: clientId,
    roomCode: null,
    ws,
    isAlive: true,
  };
  clients.set(clientId, client);

  console.log(`[connect] Client ${clientId} connected (total: ${clients.size})`);

  // ── Heartbeat ──
  ws.on("pong", () => {
    client.isAlive = true;
  });

  // ── Message handling ──
  ws.on("message", (data: Buffer | string) => {
    let message: RelayMessage;

    try {
      const raw = typeof data === "string" ? data : data.toString("utf-8");
      message = JSON.parse(raw);
    } catch {
      sendMessage(ws, {
        type: "error",
        payload: { message: "Invalid JSON", code: "INVALID_JSON" },
      });
      return;
    }

    if (!message.type) {
      sendMessage(ws, {
        type: "error",
        payload: { message: "Missing message type", code: "MISSING_TYPE" },
      });
      return;
    }

    handleMessage(client, message);
  });

  // ── Disconnect ──
  ws.on("close", () => {
    handleDisconnect(client);
  });

  ws.on("error", (err: Error) => {
    console.error(`[error] Client ${clientId}:`, err.message);
    handleDisconnect(client);
  });
});

// ── Message Router ─────────────────────────────────────────────────────

function handleMessage(client: ClientInfo, message: RelayMessage): void {
  switch (message.type) {
    case "create_room":
      handleCreateRoom(client);
      break;
    case "join_room":
      handleJoinRoom(client, message);
      break;
    case "leave_room":
      handleLeaveRoom(client);
      break;
    case "relay":
      handleRelay(client, message);
      break;
    default:
      sendMessage(client.ws, {
        type: "error",
        payload: {
          message: `Unknown message type: ${message.type}`,
          code: "UNKNOWN_TYPE",
        },
      });
  }
}

// ── Handlers ───────────────────────────────────────────────────────────

function handleCreateRoom(client: ClientInfo): void {
  // Leave current room if already in one
  if (client.roomCode) {
    leaveRoom(client);
  }

  try {
    const code = generateRoomCode(getActiveRoomCodes());
    const room = new Room(code);
    rooms.set(code, room);

    room.addClient(client);
    client.roomCode = code;

    console.log(
      `[room] Created room ${code} by client ${client.id} (total rooms: ${rooms.size})`
    );

    sendMessage(client.ws, {
      type: "room_created",
      payload: { roomCode: code, clientId: client.id },
    });
  } catch (err: any) {
    sendMessage(client.ws, {
      type: "error",
      payload: {
        message: "Failed to create room: " + err.message,
        code: "ROOM_CREATE_FAILED",
      },
    });
  }
}

function handleJoinRoom(client: ClientInfo, message: RelayMessage): void {
  const roomCode = (message.payload?.roomCode || message.payload?.room || "")
    .toString()
    .toUpperCase()
    .trim();

  if (!roomCode) {
    sendMessage(client.ws, {
      type: "error",
      payload: { message: "Room code is required", code: "MISSING_ROOM_CODE" },
    });
    return;
  }

  const room = rooms.get(roomCode);
  if (!room) {
    sendMessage(client.ws, {
      type: "error",
      payload: { message: "Room not found", code: "ROOM_NOT_FOUND" },
    });
    return;
  }

  // Leave current room if already in one
  if (client.roomCode) {
    leaveRoom(client);
  }

  if (room.isFull()) {
    sendMessage(client.ws, {
      type: "error",
      payload: { message: "Room is full (max 10 clients)", code: "ROOM_FULL" },
    });
    return;
  }

  const existingPeers = room.getClientIds();
  room.addClient(client);
  client.roomCode = roomCode;

  console.log(
    `[room] Client ${client.id} joined room ${roomCode} (${room.getClientCount()} clients)`
  );

  // Notify the joining client
  sendMessage(client.ws, {
    type: "room_joined",
    payload: {
      roomCode,
      clientId: client.id,
      peers: existingPeers,
    },
  });

  // Notify existing peers
  room.broadcast(
    JSON.stringify({
      type: "peer_joined",
      payload: { clientId: client.id, roomCode },
    } satisfies ServerMessage),
    client.id
  );
}

function handleLeaveRoom(client: ClientInfo): void {
  if (!client.roomCode) {
    sendMessage(client.ws, {
      type: "error",
      payload: { message: "Not in a room", code: "NOT_IN_ROOM" },
    });
    return;
  }
  leaveRoom(client);
}

function handleRelay(client: ClientInfo, message: RelayMessage): void {
  if (!client.roomCode) {
    sendMessage(client.ws, {
      type: "error",
      payload: {
        message: "Must join a room before relaying messages",
        code: "NOT_IN_ROOM",
      },
    });
    return;
  }

  const room = rooms.get(client.roomCode);
  if (!room) {
    sendMessage(client.ws, {
      type: "error",
      payload: { message: "Room no longer exists", code: "ROOM_NOT_FOUND" },
    });
    return;
  }

  // Zero-knowledge relay: forward payload as-is, never inspect it
  const relayMsg: ServerMessage = {
    type: "relay",
    payload: message.payload,
    sender: client.id,
    room: client.roomCode,
  };

  room.broadcast(JSON.stringify(relayMsg), client.id);
}

// ── Helpers ────────────────────────────────────────────────────────────

function leaveRoom(client: ClientInfo): void {
  const roomCode = client.roomCode;
  if (!roomCode) return;

  const room = rooms.get(roomCode);
  client.roomCode = null;

  if (!room) return;

  room.removeClient(client.id);

  // Notify remaining peers
  room.broadcastAll(
    JSON.stringify({
      type: "peer_left",
      payload: { clientId: client.id, roomCode },
    } satisfies ServerMessage)
  );

  console.log(
    `[room] Client ${client.id} left room ${roomCode} (${room.getClientCount()} remaining)`
  );

  // Cleanup empty rooms
  if (room.isEmpty()) {
    rooms.delete(roomCode);
    console.log(
      `[room] Destroyed empty room ${roomCode} (total rooms: ${rooms.size})`
    );
  }
}

function handleDisconnect(client: ClientInfo): void {
  if (client.roomCode) {
    leaveRoom(client);
  }
  clients.delete(client.id);
  console.log(
    `[disconnect] Client ${client.id} disconnected (total: ${clients.size})`
  );
}

function sendMessage(ws: WebSocket, message: ServerMessage): void {
  if (ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify(message));
  }
}

// ── Heartbeat interval ────────────────────────────────────────────────

const heartbeatInterval = setInterval(() => {
  for (const [id, client] of clients) {
    if (!client.isAlive) {
      console.log(`[heartbeat] Client ${id} timed out, disconnecting`);
      client.ws.terminate();
      handleDisconnect(client);
      continue;
    }

    client.isAlive = false;
    if (client.ws.readyState === WebSocket.OPEN) {
      client.ws.ping();
    }
  }
}, HEARTBEAT_INTERVAL_MS);

// ── Graceful shutdown ──────────────────────────────────────────────────

function shutdown(): void {
  console.log("[server] Shutting down...");
  clearInterval(heartbeatInterval);

  wss.close(() => {
    server.close(() => {
      console.log("[server] Shutdown complete");
      process.exit(0);
    });
  });

  // Force exit after 5 seconds
  setTimeout(() => {
    console.error("[server] Forced shutdown after timeout");
    process.exit(1);
  }, 5000);
}

process.on("SIGINT", shutdown);
process.on("SIGTERM", shutdown);

// ── Start ──────────────────────────────────────────────────────────────

server.listen(PORT, () => {
  console.log(`[server] Stark-Link relay server listening on port ${PORT}`);
  console.log(`[server] Health check: http://localhost:${PORT}/health`);
});
