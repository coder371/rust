export type ChatMessage = {
  id: string;
  room_id: string;
  author: string;
  body: string;
  sent_at: number;
};

export type RoomSummary = {
  id: string;
  name: string;
  created_by: string;
  created_at: number;
  members: number;
};

export type Session = {
  token: string;
  username: string;
};

/** الأحداث الجاية من الخادم. الحقل `type` بيحدد الشكل. */
export type ServerEvent =
  | { type: "history"; messages: ChatMessage[] }
  | ({ type: "message" } & ChatMessage)
  | { type: "presence"; users: string[] }
  | { type: "typing"; user: string }
  | { type: "system"; text: string }
  | { type: "error"; text: string };

/** سطر معروض في العمود: رسالة أو إشعار من الخادم. */
export type Entry =
  | { kind: "message"; message: ChatMessage }
  | { kind: "system"; id: string; text: string };

export type ConnectionStatus = "connecting" | "open" | "reconnecting" | "closed";
