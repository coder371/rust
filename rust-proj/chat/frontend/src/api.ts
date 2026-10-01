import type { RoomSummary, Session } from "./types";

/** منفذ الباك. الفرونت على 5173 والباك على 3000. */
const API_PORT = 3000;

/**
 * عنوان الباك.
 *
 * لو `VITE_API_URL` مش متحطّة، بنبني العنوان من المضيف اللي فتحت بيه
 * الصفحة. كده نفس البناء يشتغل من `localhost` ومن الـ IP على الشبكة
 * من غير أي تغيير في الإعدادات.
 */
const BASE =
  import.meta.env.VITE_API_URL || `${window.location.protocol}//${window.location.hostname}:${API_PORT}`;

/** خطأ جاي من الـ API برسالة صالحة للعرض. */
export class ApiError extends Error {}

async function request<T>(path: string, init: RequestInit = {}, token?: string): Promise<T> {
  let response: Response;

  try {
    response = await fetch(`${BASE}${path}`, {
      ...init,
      headers: {
        "Content-Type": "application/json",
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
        ...init.headers,
      },
    });
  } catch {
    throw new ApiError("مفيش اتصال بالخادم. اتأكد إنه شغال.");
  }

  if (!response.ok) {
    const body = await response.json().catch(() => null);
    throw new ApiError(body?.error ?? `الطلب فشل (${response.status})`);
  }

  return response.json() as Promise<T>;
}

export const api = {
  register: (username: string, password: string) =>
    request<Session & { expires_in: number }>("/api/auth/register", {
      method: "POST",
      body: JSON.stringify({ username, password }),
    }),

  login: (username: string, password: string) =>
    request<Session & { expires_in: number }>("/api/auth/login", {
      method: "POST",
      body: JSON.stringify({ username, password }),
    }),

  rooms: (token: string) => request<RoomSummary[]>("/api/rooms", {}, token),

  createRoom: (token: string, name: string) =>
    request<RoomSummary>("/api/rooms", { method: "POST", body: JSON.stringify({ name }) }, token),
};

export function socketUrl(token: string, roomId: string) {
  const url = new URL(BASE);
  url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
  url.pathname = "/ws";
  url.searchParams.set("token", token);
  url.searchParams.set("room", roomId);
  return url.toString();
}
