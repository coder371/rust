// الحالة المشتركة في الذاكرة — نفس تصميم state.rs.
import { randomUUID } from "node:crypto";

import { config } from "./config.js";

export class Room {
  constructor(name, createdBy) {
    this.id = randomUUID();
    this.name = name;
    this.createdBy = createdBy;
    this.createdAt = Date.now();

    /** @type {Set<import('ws').WebSocket>} */
    this.sockets = new Set();
    /** @type {Map<string, number>} اسم المستخدم -> عدد اتصالاته */
    this.members = new Map();
    /** @type {object[]} */
    this.history = [];
  }

  /**
   * بيسلسل الحدث **مرة واحدة** ويبعت نفس السلسلة لكل المشتركين.
   *
   * ده نفس قرار نسخة راست: السلسلة تحصل مرة لكل رسالة بدل مرة لكل
   * مشترك. من غيره الغرفة اللي فيها ١٠٠ شخص بتعمل ١٠٠ عملية JSON.
   */
  publish(event) {
    const payload = JSON.stringify(event);

    for (const socket of this.sockets) {
      if (socket.readyState === socket.OPEN) socket.send(payload);
    }
  }

  pushMessage(message) {
    if (this.history.length === config.historyLimit) this.history.shift();
    this.history.push(message);

    this.publish({ type: "message", ...message });
  }

  /** بيرجّع true لو دي أول جلسة للمستخدم في الغرفة. */
  join(username) {
    const count = (this.members.get(username) ?? 0) + 1;
    this.members.set(username, count);
    return count === 1;
  }

  /** بيرجّع true لو دي آخر جلسة له. */
  leave(username) {
    const count = this.members.get(username);
    if (count === undefined) return false;

    if (count <= 1) {
      this.members.delete(username);
      return true;
    }

    this.members.set(username, count - 1);
    return false;
  }

  memberNames() {
    return [...this.members.keys()].sort();
  }

  summary() {
    return {
      id: this.id,
      name: this.name,
      created_by: this.createdBy,
      created_at: this.createdAt,
      members: this.members.size,
    };
  }
}

class AppState {
  constructor() {
    /** @type {Map<string, { passwordHash: string, createdAt: number }>} */
    this.users = new Map();
    /** @type {Map<string, Room>} */
    this.rooms = new Map();

    const general = new Room("عام", "system");
    this.rooms.set(general.id, general);
  }

  insertUser(username, passwordHash) {
    if (this.users.has(username)) return false;

    this.users.set(username, { passwordHash, createdAt: Date.now() });
    return true;
  }

  createRoom(name, createdBy) {
    for (const room of this.rooms.values()) {
      if (room.name === name) return null;
    }

    const room = new Room(name, createdBy);
    this.rooms.set(room.id, room);
    return room;
  }

  roomSummaries() {
    return [...this.rooms.values()]
      .sort((a, b) => a.createdAt - b.createdAt)
      .map((room) => room.summary());
  }
}

export const state = new AppState();
