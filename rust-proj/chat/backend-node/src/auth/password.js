// نفس معاملات Argon2id اللي راست بتستخدمها افتراضياً، ونفس حد التوازي.
import { availableParallelism } from "node:os";

import argon2 from "argon2";

/** مطابقة للافتراضي في كريت argon2 بتاع راست: m=19456، t=2، p=1. */
const OPTIONS = {
  type: argon2.argon2id,
  memoryCost: 19_456,
  timeCost: 2,
  parallelism: 1,
};

/**
 * حد عدد عمليات التجزئة المتوازية.
 *
 * نفس سبب السيمافور في نسخة راست: كل عملية بتحجز ~١٩ ميجا، ومن غير حد
 * دفعة تسجيلات كبيرة بتفجّر الذاكرة.
 */
const SLOTS = availableParallelism();

let active = 0;
const waiting = [];

async function withSlot(task) {
  if (active >= SLOTS) await new Promise((resolve) => waiting.push(resolve));

  active += 1;
  try {
    return await task();
  } finally {
    active -= 1;
    waiting.shift()?.();
  }
}

export const hashPassword = (password) => withSlot(() => argon2.hash(password, OPTIONS));

/** `false` = الباسورد غلط. الرمي = الهاش نفسه تالف. */
export const verifyPassword = (password, passwordHash) =>
  withSlot(async () => {
    try {
      return await argon2.verify(passwordHash, password);
    } catch {
      return false;
    }
  });
