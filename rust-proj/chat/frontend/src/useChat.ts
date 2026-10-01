import { useCallback, useEffect, useRef, useState } from "react";

import { socketUrl } from "./api";
import type { ChatMessage, ConnectionStatus, Entry, ServerEvent } from "./types";

/** كل قد إيه نبعت إشارة "بيكتب" على الأكتر. */
const TYPING_THROTTLE = 2_000;
/** بعد قد إيه نعتبر إن الشخص بطّل كتابة. */
const TYPING_TIMEOUT = 4_000;
/** تأخير إعادة المحاولة، بيزيد مع كل فشل. */
const BACKOFF_STEPS = [500, 1_000, 2_000, 4_000, 8_000];

type Options = {
  token: string;
  roomId: string | null;
  username: string;
};

export function useChat({ token, roomId, username }: Options) {
  const [entries, setEntries] = useState<Entry[]>([]);
  const [presence, setPresence] = useState<string[]>([]);
  const [typing, setTyping] = useState<string[]>([]);
  const [status, setStatus] = useState<ConnectionStatus>("closed");
  const [notice, setNotice] = useState<string | null>(null);

  const socketRef = useRef<WebSocket | null>(null);
  const attemptRef = useRef(0);
  const retryRef = useRef<number | null>(null);
  const lastTypingSentRef = useRef(0);
  const typingTimersRef = useRef(new Map<string, number>());

  const markTyping = useCallback(
    (user: string) => {
      if (user === username) return;

      setTyping((current) => (current.includes(user) ? current : [...current, user]));

      const timers = typingTimersRef.current;
      window.clearTimeout(timers.get(user));

      timers.set(
        user,
        window.setTimeout(() => {
          setTyping((current) => current.filter((name) => name !== user));
          timers.delete(user);
        }, TYPING_TIMEOUT),
      );
    },
    [username],
  );

  useEffect(() => {
    if (!roomId) return;

    // علم إلغاء خاص بهذه الدورة وحدها.
    //
    // ماينفعش يكون ref مشترك: في وضع التطوير React بيعمل mount ثم
    // cleanup ثم mount تاني. لو العلم مشترك، الـ onclose بتاع السوكِت
    // الأول بيلاقيه اترجع true فيفتح اتصال إضافي — والنتيجة سوكِتين
    // على نفس الغرفة وكل رسالة بتوصل مرتين.
    let cancelled = false;

    setEntries([]);
    setPresence([]);
    setTyping([]);

    const connect = () => {
      if (cancelled) return;

      setStatus(attemptRef.current === 0 ? "connecting" : "reconnecting");

      const socket = new WebSocket(socketUrl(token, roomId));
      socketRef.current = socket;

      socket.onopen = () => {
        attemptRef.current = 0;
        setStatus("open");
      };

      socket.onmessage = (event) => {
        const payload = JSON.parse(event.data) as ServerEvent;

        switch (payload.type) {
          case "history":
            setEntries(payload.messages.map((message) => ({ kind: "message", message })));
            break;

          case "message": {
            const { type: _ignored, ...message } = payload;
            setEntries((current) => [...current, { kind: "message", message: message as ChatMessage }]);
            // وصلت رسالة من حد كان بيكتب، يبقى خلص.
            setTyping((current) => current.filter((name) => name !== message.author));
            break;
          }

          case "presence":
            setPresence(payload.users);
            break;

          case "typing":
            markTyping(payload.user);
            break;

          case "system":
            setEntries((current) => [
              ...current,
              { kind: "system", id: `sys-${Date.now()}-${current.length}`, text: payload.text },
            ]);
            break;

          case "error":
            setNotice(payload.text);
            break;
        }
      };

      socket.onclose = () => {
        if (cancelled) return;

        const delay = BACKOFF_STEPS[Math.min(attemptRef.current, BACKOFF_STEPS.length - 1)];
        attemptRef.current += 1;
        setStatus("reconnecting");
        retryRef.current = window.setTimeout(connect, delay);
      };
    };

    connect();

    return () => {
      cancelled = true;
      if (retryRef.current) window.clearTimeout(retryRef.current);
      typingTimersRef.current.forEach((timer) => window.clearTimeout(timer));
      typingTimersRef.current.clear();
      attemptRef.current = 0;
      socketRef.current?.close();
      socketRef.current = null;
      setStatus("closed");
    };
  }, [token, roomId, markTyping]);

  const send = useCallback((body: string) => {
    const socket = socketRef.current;
    if (socket?.readyState !== WebSocket.OPEN) return false;

    socket.send(JSON.stringify({ type: "send", body }));
    return true;
  }, []);

  /** بتتبعت وقت الكتابة، مخنوقة عشان ما نغرقش الخادم بكل ضغطة زرار. */
  const sendTyping = useCallback(() => {
    const socket = socketRef.current;
    if (socket?.readyState !== WebSocket.OPEN) return;

    const now = Date.now();
    if (now - lastTypingSentRef.current < TYPING_THROTTLE) return;

    lastTypingSentRef.current = now;
    socket.send(JSON.stringify({ type: "typing" }));
  }, []);

  return { entries, presence, typing, status, notice, setNotice, send, sendTyping };
}
