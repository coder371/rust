// نفس إعدادات نسخة راست بالظبط، عشان المقارنة تكون على الكود مش على الضبط.
import "dotenv/config";

const num = (key, fallback) => {
  const value = Number(process.env[key]);
  return Number.isFinite(value) ? value : fallback;
};

const jwtSecret = process.env.JWT_SECRET ?? "";

if (!jwtSecret.trim()) {
  throw new Error("JWT_SECRET غير موجود — انسخ .env.example إلى .env");
}

export const config = {
  jwtSecret,
  jwtTtlSeconds: num("JWT_TTL_SECONDS", 86_400),
  bindHost: process.env.BIND_HOST ?? "127.0.0.1",
  bindPort: num("BIND_PORT", 3001),
  allowedOrigins: (process.env.ALLOWED_ORIGINS ?? "http://localhost:5173")
    .split(",")
    .map((origin) => origin.trim())
    .filter(Boolean),
  historyLimit: num("HISTORY_LIMIT", 100),
  rateLimit: num("RATE_LIMIT", 15),
  rateWindowMs: num("RATE_WINDOW_SECS", 5) * 1000,
  maxMessageChars: num("MAX_MESSAGE_CHARS", 2000),
};
