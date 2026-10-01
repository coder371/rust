import { useRef, useState, type KeyboardEvent } from "react";

export function Composer({
  disabled,
  onSend,
  onTyping,
}: {
  disabled: boolean;
  onSend: (body: string) => boolean;
  onTyping: () => void;
}) {
  const [draft, setDraft] = useState("");
  const inputRef = useRef<HTMLTextAreaElement>(null);

  function resize() {
    const input = inputRef.current;
    if (!input) return;

    input.style.height = "auto";
    input.style.height = `${input.scrollHeight}px`;
  }

  function submit() {
    const body = draft.trim();
    if (!body) return;

    if (onSend(body)) {
      setDraft("");
      requestAnimationFrame(resize);
    }
  }

  function onKeyDown(event: KeyboardEvent<HTMLTextAreaElement>) {
    // Enter يبعت، Shift+Enter بيعمل سطر جديد.
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      submit();
    }
  }

  return (
    <div className="composer">
      <div className="composer__inner">
        <textarea
          ref={inputRef}
          className="composer__input"
          rows={1}
          value={draft}
          disabled={disabled}
          placeholder={disabled ? "مستني الاتصال…" : "اكتب رسالة"}
          aria-label="رسالة جديدة"
          onChange={(event) => {
            setDraft(event.target.value);
            resize();
            onTyping();
          }}
          onKeyDown={onKeyDown}
        />
        <button
          className="composer__send"
          type="button"
          onClick={submit}
          disabled={disabled || draft.trim().length === 0}
          aria-label="ابعت"
        >
          <svg width="15" height="15" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <path
              d="M14 8 2 2.5 4 8l-2 5.5L14 8Z"
              stroke="currentColor"
              strokeWidth="1.6"
              strokeLinejoin="round"
              fill="currentColor"
            />
          </svg>
        </button>
      </div>
      <p className="composer__hint">Enter يبعت · Shift+Enter سطر جديد</p>
    </div>
  );
}
