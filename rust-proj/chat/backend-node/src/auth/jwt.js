import jwt from "jsonwebtoken";

import { config } from "../config.js";

export const createToken = (username) =>
  jwt.sign({ sub: username }, config.jwtSecret, { expiresIn: config.jwtTtlSeconds });

/** بيرجّع الـ claims، أو null لو التوكن مرفوض. */
export function verifyToken(token) {
  try {
    return jwt.verify(token, config.jwtSecret);
  } catch {
    return null;
  }
}
