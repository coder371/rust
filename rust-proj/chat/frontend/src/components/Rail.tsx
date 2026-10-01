import { useState, type FormEvent } from "react";

import type { ConnectionStatus, RoomSummary } from "../types";
import { Mark } from "./Mark";

export function Rail({
  rooms,
  activeRoomId,
  presence,
  me,
  status,
  onSelect,
  onCreate,
  onSignOut,
}: {
  rooms: RoomSummary[];
  activeRoomId: string | null;
  presence: string[];
  me: string;
  status: ConnectionStatus;
  onSelect: (roomId: string) => void;
  onCreate: (name: string) => Promise<void>;
  onSignOut: () => void;
}) {
  const [name, setName] = useState("");
  const [busy, setBusy] = useState(false);

  async function create(event: FormEvent) {
    event.preventDefault();
    const trimmed = name.trim();
    if (!trimmed) return;

    setBusy(true);
    try {
      await onCreate(trimmed);
      setName("");
    } finally {
      setBusy(false);
    }
  }

  return (
    <nav className="rail">
      <div className="rail__head">
        <h1 className="rail__mark mark">
          <Mark size={17} />
          مجلس
        </h1>
      </div>

      <div className="rail__body">
        <h2 className="rail__section">الغرف</h2>

        {rooms.map((room) => (
          <button
            key={room.id}
            className="room"
            aria-current={room.id === activeRoomId}
            onClick={() => onSelect(room.id)}
          >
            <span className="room__name">{room.name}</span>
            <span className="room__count">{room.members}</span>
          </button>
        ))}

        <form className="new-room" onSubmit={create}>
          <input
            className="input"
            value={name}
            onChange={(event) => setName(event.target.value)}
            placeholder="غرفة جديدة"
            aria-label="اسم الغرفة الجديدة"
            maxLength={48}
          />
          <button className="button" type="submit" disabled={busy || !name.trim()}>
            زوّد
          </button>
        </form>

        {activeRoomId && status === "open" && (
          <>
            <h2 className="rail__section">هنا دلوقتي — {presence.length}</h2>
            {presence.map((user) => (
              <p className="presence" key={user}>
                <span className="presence__dot" />
                {user}
              </p>
            ))}
          </>
        )}
      </div>

      <div className="rail__foot">
        <span className="rail__me">{me}</span>
        <button className="button button--quiet" type="button" onClick={onSignOut}>
          اخرج
        </button>
      </div>
    </nav>
  );
}
