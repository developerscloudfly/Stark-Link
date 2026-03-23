import { WebSocket } from "ws";

/** Generic relay message envelope */
export interface RelayMessage {
  type: string;
  payload: any;
  sender?: string;
  room?: string;
}

/** Internal client tracking info */
export interface ClientInfo {
  id: string;
  roomCode: string | null;
  ws: WebSocket;
  isAlive: boolean;
}

// ── Server → Client message types ──────────────────────────────────────

export interface RoomCreatedMessage {
  type: "room_created";
  payload: {
    roomCode: string;
    clientId: string;
  };
}

export interface RoomJoinedMessage {
  type: "room_joined";
  payload: {
    roomCode: string;
    clientId: string;
    peers: string[];
  };
}

export interface PeerJoinedMessage {
  type: "peer_joined";
  payload: {
    clientId: string;
    roomCode: string;
  };
}

export interface PeerLeftMessage {
  type: "peer_left";
  payload: {
    clientId: string;
    roomCode: string;
  };
}

export interface RelayDataMessage {
  type: "relay";
  payload: any;
  sender: string;
  room: string;
}

export interface ErrorMessage {
  type: "error";
  payload: {
    message: string;
    code?: string;
  };
}

export type ServerMessage =
  | RoomCreatedMessage
  | RoomJoinedMessage
  | PeerJoinedMessage
  | PeerLeftMessage
  | RelayDataMessage
  | ErrorMessage;
