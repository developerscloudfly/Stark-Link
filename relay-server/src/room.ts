import { WebSocket } from "ws";
import { ClientInfo } from "./types";

const ROOM_CODE_LENGTH = 6;
const ROOM_CODE_CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const MAX_CLIENTS_PER_ROOM = 10;

/**
 * Generate a unique 6-character alphanumeric room code (uppercase).
 * Checks against existing codes to avoid collisions.
 */
export function generateRoomCode(existingCodes: Set<string>): string {
  let code: string;
  let attempts = 0;
  const maxAttempts = 100;

  do {
    code = "";
    for (let i = 0; i < ROOM_CODE_LENGTH; i++) {
      code += ROOM_CODE_CHARS.charAt(
        Math.floor(Math.random() * ROOM_CODE_CHARS.length)
      );
    }
    attempts++;
    if (attempts >= maxAttempts) {
      throw new Error("Failed to generate unique room code after max attempts");
    }
  } while (existingCodes.has(code));

  return code;
}

/**
 * Represents a communication room that clients can join.
 * The server never inspects message payloads — it only forwards them.
 */
export class Room {
  public readonly id: string;
  public readonly code: string;
  public readonly createdAt: Date;
  private clients: Map<string, ClientInfo>;

  constructor(code: string) {
    this.id = code;
    this.code = code;
    this.createdAt = new Date();
    this.clients = new Map();
  }

  /**
   * Add a client to the room.
   * Returns false if room is full.
   */
  addClient(client: ClientInfo): boolean {
    if (this.clients.size >= MAX_CLIENTS_PER_ROOM) {
      return false;
    }
    this.clients.set(client.id, client);
    return true;
  }

  /** Remove a client from the room by ID. */
  removeClient(clientId: string): boolean {
    return this.clients.delete(clientId);
  }

  /** Get count of connected clients. */
  getClientCount(): number {
    return this.clients.size;
  }

  /** Get list of all client IDs in this room. */
  getClientIds(): string[] {
    return Array.from(this.clients.keys());
  }

  /** Check if room is empty. */
  isEmpty(): boolean {
    return this.clients.size === 0;
  }

  /** Check if room is full. */
  isFull(): boolean {
    return this.clients.size >= MAX_CLIENTS_PER_ROOM;
  }

  /**
   * Broadcast a message to all clients in the room except the sender.
   * Silently skips clients whose WebSocket is not in OPEN state.
   */
  broadcast(message: string, excludeClientId?: string): void {
    for (const [id, client] of this.clients) {
      if (id === excludeClientId) continue;
      if (client.ws.readyState === WebSocket.OPEN) {
        client.ws.send(message);
      }
    }
  }

  /**
   * Send a message to every client in the room (including sender).
   */
  broadcastAll(message: string): void {
    for (const [, client] of this.clients) {
      if (client.ws.readyState === WebSocket.OPEN) {
        client.ws.send(message);
      }
    }
  }
}
