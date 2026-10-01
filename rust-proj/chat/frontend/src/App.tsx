import { useCallback, useEffect, useState } from "react";

import { ApiError, api } from "./api";
import { AuthScreen } from "./components/AuthScreen";
import { Composer } from "./components/Composer";
import { Rail } from "./components/Rail";
import { Thread } from "./components/Thread";
import type { ConnectionStatus, RoomSummary, Session } from "./types";
import { useChat } from "./useChat";

const STORAGE_KEY = "majlis.session";

const STATUS_LABEL: Record<ConnectionStatus, string> = {
  connecting: "بيوصل",
  open: "متصل",
  reconnecting: "بيحاول تاني",
  closed: "مقطوع",
};

function loadSession(): Session | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? (JSON.parse(raw) as Session) : null;
  } catch {
    return null;
  }
}

export default function App() {
  const [session, setSession] = useState<Session | null>(loadSession);
  const [rooms, setRooms] = useState<RoomSummary[]>([]);
  const [activeRoomId, setActiveRoomId] = useState<string | null>(null);
  const [railError, setRailError] = useState<string | null>(null);

  const { entries, presence, typing, status, notice, setNotice, send, sendTyping } = useChat({
    token: session?.token ?? "",
    roomId: session ? activeRoomId : null,
    username: session?.username ?? "",
  });

  const signOut = useCallback(() => {
    localStorage.removeItem(STORAGE_KEY);
    setSession(null);
    setRooms([]);
    setActiveRoomId(null);
  }, []);

  const refreshRooms = useCallback(async () => {
    if (!session) return;

    try {
      const list = await api.rooms(session.token);
      setRooms(list);
      setActiveRoomId((current) => current ?? list[0]?.id ?? null);
    } catch (caught) {
      // توكن منتهي أو ملغي: نخرج بدل ما نفضل نحاول.
      if (caught instanceof ApiError && caught.message.includes("توكن")) {
        signOut();
        return;
      }
      setRailError(caught instanceof ApiError ? caught.message : "مقدرناش نجيب الغرف");
    }
  }, [session, signOut]);

  useEffect(() => {
    void refreshRooms();
  }, [refreshRooms]);

  // عدّاد الحاضرين في القائمة بيتحدّث مع كل تغيير في الغرفة المفتوحة.
  useEffect(() => {
    if (!activeRoomId) return;

    setRooms((current) =>
      current.map((room) =>
        room.id === activeRoomId ? { ...room, members: presence.length } : room,
      ),
    );
  }, [presence, activeRoomId]);

  useEffect(() => {
    if (!notice) return;

    const timer = window.setTimeout(() => setNotice(null), 4_000);
    return () => window.clearTimeout(timer);
  }, [notice, setNotice]);

  function startSession(next: Session) {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
    setSession(next);
  }

  async function createRoom(name: string) {
    if (!session) return;

    try {
      const room = await api.createRoom(session.token, name);
      setRooms((current) => [...current, room]);
      setActiveRoomId(room.id);
      setRailError(null);
    } catch (caught) {
      setRailError(caught instanceof ApiError ? caught.message : "مقدرناش نعمل الغرفة");
    }
  }

  if (!session) {
    return <AuthScreen onSession={startSession} />;
  }

  const activeRoom = rooms.find((room) => room.id === activeRoomId) ?? null;

  return (
    <div className="app">
      <div className="room-view">
        {activeRoom ? (
          <>
            <header className="room-view__head">
              <h2 className="room-view__title">{activeRoom.name}</h2>
              {status === "open" && <p className="room-view__count">{presence.length} هنا</p>}
              <p className="status" data-state={status}>
                <span className="status__dot" />
                {STATUS_LABEL[status]}
              </p>
            </header>

            <Thread entries={entries} me={session.username} />

            <div>
              <p className="typing" aria-live="polite">
                {typing.length === 1 && `${typing[0]} بيكتب…`}
                {typing.length > 1 && `${typing.length} أشخاص بيكتبوا…`}
              </p>
              <Composer disabled={status !== "open"} onSend={send} onTyping={sendTyping} />
            </div>
          </>
        ) : (
          <div className="empty-state">
            <p>{railError ?? "اختار غرفة من على اليمين، أو اعمل واحدة جديدة."}</p>
          </div>
        )}
      </div>

      <Rail
        rooms={rooms}
        activeRoomId={activeRoomId}
        presence={presence}
        me={session.username}
        status={status}
        onSelect={setActiveRoomId}
        onCreate={createRoom}
        onSignOut={signOut}
      />

      {notice && (
        <p className="toast" role="status">
          {notice}
        </p>
      )}
    </div>
  );
}
