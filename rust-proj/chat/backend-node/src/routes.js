import { config } from "./config.js";
import { state } from "./state.js";
import { createToken, verifyToken } from "./auth/jwt.js";
import { hashPassword, verifyPassword } from "./auth/password.js";

const MIN_USERNAME = 3;
const MAX_USERNAME = 32;
const MIN_PASSWORD = 8;
const MAX_ROOM_NAME = 48;

/** بيتأكد من التوكن في هيدر Authorization. */
function authenticate(request, reply) {
  const header = request.headers.authorization;
  const token = header?.startsWith("Bearer ") ? header.slice(7) : null;
  const claims = token ? verifyToken(token) : null;

  if (!claims) {
    reply.code(401).send({ error: "مفيش توكن" });
    return null;
  }

  return claims.sub;
}

function validate(body) {
  const username = (body?.username ?? "").trim();
  const password = body?.password ?? "";

  if (username.length < MIN_USERNAME || username.length > MAX_USERNAME) {
    return { error: `اسم المستخدم لازم يكون بين ${MIN_USERNAME} و ${MAX_USERNAME} حرف` };
  }
  if (/\s/.test(username)) return { error: "اسم المستخدم ماينفعش يحتوي مسافات" };
  if (password.length < MIN_PASSWORD) {
    return { error: `الباسورد لازم ${MIN_PASSWORD} حروف على الأقل` };
  }

  return { username, password };
}

const session = (username) => ({
  token: createToken(username),
  username,
  expires_in: config.jwtTtlSeconds,
});

export function registerRoutes(app) {
  app.get("/health", async () => "ok");

  app.post("/api/auth/register", async (request, reply) => {
    const parsed = validate(request.body);
    if (parsed.error) return reply.code(400).send({ error: parsed.error });

    const passwordHash = await hashPassword(parsed.password);

    if (!state.insertUser(parsed.username, passwordHash)) {
      return reply.code(409).send({ error: "الاسم ده محجوز" });
    }

    return session(parsed.username);
  });

  app.post("/api/auth/login", async (request, reply) => {
    const username = (request.body?.username ?? "").trim();
    const stored = state.users.get(username);

    // نفس الرسالة في الحالتين عشان ما نكشفش إن الاسم موجود.
    const invalid = () => reply.code(401).send({ error: "اسم المستخدم أو الباسورد غلط" });

    if (!stored) return invalid();
    if (!(await verifyPassword(request.body?.password ?? "", stored.passwordHash))) return invalid();

    return session(username);
  });

  app.get("/api/auth/me", async (request, reply) => {
    const username = authenticate(request, reply);
    if (!username) return;

    return { username, created_at: state.users.get(username)?.createdAt ?? null };
  });

  app.get("/api/rooms", async (request, reply) => {
    if (!authenticate(request, reply)) return;

    return state.roomSummaries();
  });

  app.post("/api/rooms", async (request, reply) => {
    const username = authenticate(request, reply);
    if (!username) return;

    const name = (request.body?.name ?? "").trim();

    if (!name || name.length > MAX_ROOM_NAME) {
      return reply.code(400).send({ error: `اسم الغرفة لازم يكون بين 1 و ${MAX_ROOM_NAME} حرف` });
    }

    const room = state.createRoom(name, username);
    if (!room) return reply.code(409).send({ error: "في غرفة بنفس الاسم" });

    return room.summary();
  });

  app.get("/api/rooms/:roomId/messages", async (request, reply) => {
    if (!authenticate(request, reply)) return;

    const room = state.rooms.get(request.params.roomId);
    if (!room) return reply.code(404).send({ error: "الغرفة مش موجودة" });

    return room.history;
  });
}
