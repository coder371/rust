import { useEffect, useRef } from "react";

import type { Entry } from "../types";

const time = new Intl.DateTimeFormat("ar-EG", { hour: "2-digit", minute: "2-digit" });

export function Thread({ entries, me }: { entries: Entry[]; me: string }) {
  const bottomRef = useRef<HTMLDivElement>(null);
  const scrollerRef = useRef<HTMLDivElement>(null);
  /** بنتابع لو المستخدم طالع لفوق يقرا، عشان ما نخطفش الشاشة منه. */
  const pinnedRef = useRef(true);

  useEffect(() => {
    if (pinnedRef.current) {
      bottomRef.current?.scrollIntoView({ block: "end" });
    }
  }, [entries]);

  function onScroll() {
    const scroller = scrollerRef.current;
    if (!scroller) return;

    const distance = scroller.scrollHeight - scroller.scrollTop - scroller.clientHeight;
    pinnedRef.current = distance < 80;
  }

  return (
    <div className="thread" ref={scrollerRef} onScroll={onScroll}>
      <div className="thread__inner">
        {entries.length === 0 && <p className="thread__empty">لسه مفيش كلام هنا. ابدأ إنت.</p>}

        {entries.map((entry, index) => {
          if (entry.kind === "system") {
            return (
              <p className="entry entry--system" key={entry.id}>
                {entry.text}
              </p>
            );
          }

          const previous = entries[index - 1];
          // رسالة متتابعة لنفس المتكلم في نفس الدقيقة: بنشيل الترقين
          // عشان النص يفضل عمود متصل بدل ما يتقطّع بأسماء مكررة.
          const isRun =
            previous?.kind === "message" &&
            previous.message.author === entry.message.author &&
            entry.message.sent_at - previous.message.sent_at < 60_000;

          return (
            <article className={`entry${isRun ? " entry--run" : ""}`} key={entry.message.id}>
              {!isRun && (
                <header className="entry__rubric">
                  <span
                    className={`entry__author${entry.message.author === me ? " entry__author--me" : ""}`}
                  >
                    {entry.message.author}
                  </span>
                  <time className="entry__time" dateTime={new Date(entry.message.sent_at).toISOString()}>
                    {time.format(entry.message.sent_at)}
                  </time>
                </header>
              )}
              <p className="entry__body">{entry.message.body}</p>
            </article>
          );
        })}

        <div ref={bottomRef} />
      </div>
    </div>
  );
}
