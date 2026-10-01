import { useState, type FormEvent } from "react";

import { ApiError, api } from "../api";
import { Mark } from "./Mark";
import type { Session } from "../types";

type Mode = "login" | "register";

export function AuthScreen({ onSession }: { onSession: (session: Session) => void }) {
  const [mode, setMode] = useState<Mode>("login");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function submit(event: FormEvent) {
    event.preventDefault();
    setError(null);
    setBusy(true);

    try {
      const call = mode === "login" ? api.login : api.register;
      const session = await call(username.trim(), password);
      onSession({ token: session.token, username: session.username });
    } catch (caught) {
      setError(caught instanceof ApiError ? caught.message : "حصل خطأ غير متوقع");
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="auth">
      <div className="auth__card">
        <h1 className="auth__mark mark">
          <Mark size={26} />
          مجلس
        </h1>
        <p className="auth__tagline">غرف شات. ادخل باسمك وابدأ.</p>

        <form className="auth__form" onSubmit={submit}>
          {error && (
            <p className="alert" role="alert">
              {error}
            </p>
          )}

          <label className="field">
            <span className="field__label">اسم المستخدم</span>
            <input
              className="input"
              value={username}
              onChange={(event) => setUsername(event.target.value)}
              autoComplete="username"
              placeholder="من ٣ لـ ٣٢ حرف، بدون مسافات"
              required
            />
          </label>

          <label className="field">
            <span className="field__label">الباسورد</span>
            <input
              className="input"
              type="password"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              autoComplete={mode === "login" ? "current-password" : "new-password"}
              placeholder="٨ حروف على الأقل"
              required
            />
          </label>

          <button className="button" type="submit" disabled={busy}>
            {busy ? "لحظة…" : mode === "login" ? "ادخل" : "اعمل حساب"}
          </button>
        </form>

        <p className="auth__switch">
          {mode === "login" ? "لسه مالكش حساب؟ " : "عندك حساب؟ "}
          <button
            className="button button--quiet"
            type="button"
            onClick={() => {
              setMode(mode === "login" ? "register" : "login");
              setError(null);
            }}
          >
            {mode === "login" ? "اعمل واحد" : "ادخل بيه"}
          </button>
        </p>
      </div>
    </main>
  );
}
