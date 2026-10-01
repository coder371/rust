// نفس تصميم ws.rs: تحقّق قبل الترقية، تاريخ عند الدخول، حضور،
// حدّ معدّل لكل اتصال، وأخطاء خاصة بالعميل وحده.
import { randomUUID } from "node:crypto";

import { WebSocketServer } from "ws";

import { config } from "./config.js";
import { state } from "./state.js";
import { verifyToken } from "./auth/jwt.js";

const HEARTBEAT_MS = 30_000;

/** حدّ معدّل بنافذة ثابتة، واحد لكل اتصال. */
class RateLimiter {
  constructor() {
    this.windowStart = Date.now();
    this.count = 0;
  }

  allow() {
    const now = Date.now();

    if (now - this.windowStart >= config.rateWindowMs) {
      this.windowStart = now;
      this.count = 0;
    }

    this.count += 1;
    return this.count <= config.rateLimit;
  }
}

export function attachWebSocket(server) {
  // noServer عشان نتحقق من التوكن والغرفة **قبل** الترقية، فالرفض
  // يرجع رد HTTP بسبب واضح بدل ما نفتح سوكِت ونقفله.
  const wss = new WebSocketServer({ noServer: true });

  server.on("upgrade", (request, socket, head) => {
    const url = new URL(request.url, "http://localhost");
    const claims = verifyToken(url.searchParams.get("token") ?? "");

    const reject = (status, reason) => {
      socket.write(`HTTP/1.1 ${status} ${reason}\r\nConnection: close\r\n\r\n`);
      socket.destroy();
    };

    if (!claims) return reject(401, "Unauthorized");
    if (!state.users.has(claims.sub)) return reject(401, "Unauthorized");

    const room = state.rooms.get(url.searchParams.get("room") ?? "");
    if (!room) return reject(404, "Not Found");

    wss.handleUpgrade(request, socket, head, (ws) => handleSocket(ws, room, claims.sub));
  });
}

function handleSocket(ws, room, username) {
  const limiter = new RateLimiter();

  // التاريخ الأول، وبعدين نضيفه للمشتركين — نفس ترتيب نسخة راست
  // عشان مفيش رسالة تضيع في اللحظة اللي بينهم.
  ws.send(JSON.stringify({ type: "history", messages: room.history }));
  room.sockets.add(ws);

  if (room.join(username)) {
    room.publish({ type: "system", text: `${username} دخل الغرفة` });
  }
  room.publish({ type: "presence", users: room.memberNames() });

  /** خطأ يخصّ الاتصال ده وحده، مش بث للغرفة. */
  const notify = (text) => {
    if (ws.readyState === ws.OPEN) ws.send(JSON.stringify({ type: "error", text }));
  };

  ws.on("message", (raw) => {
    let event;
    try {
      event = JSON.parse(raw);
    } catch {
      return;
    }

    if (event?.type === "typing") {
      if (limiter.allow()) room.publish({ type: "typing", user: username });
      return;
    }

    if (event?.type !== "send") return;

    const body = (event.body ?? "").trim();
    if (!body) return;

    if ([...body].length > config.maxMessageChars) {
      return notify(`الرسالة أطول من ${config.maxMessageChars} حرف`);
    }
    if (!limiter.allow()) return notify("بتبعت بسرعة — استنى شوية");

    room.pushMessage({
      id: randomUUID(),
      room_id: room.id,
      author: username,
      body,
      sent_at: Date.now(),
    });
  });

  const heartbeat = setInterval(() => {
    if (ws.readyState === ws.OPEN) ws.ping();
  }, HEARTBEAT_MS);

  ws.on("close", () => {
    clearInterval(heartbeat);
    room.sockets.delete(ws);

    if (room.leave(username)) {
      room.publish({ type: "system", text: `${username} خرج من الغرفة` });
    }
    room.publish({ type: "presence", users: room.memberNames() });
  });

  ws.on("error", () => ws.close());
}
