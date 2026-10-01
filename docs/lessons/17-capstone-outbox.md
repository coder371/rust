# 17 — مشروع: Postgres + RabbitMQ (Outbox + Idempotency)

📄 الكود: [`examples/17_capstone_outbox.rs`](../examples/17_capstone_outbox.rs)

```bash
docker compose up -d
cargo run --example 17_capstone_outbox -- worker     # terminal 1
cargo run --example 17_capstone_outbox -- relay      # terminal 2
cargo run --example 17_capstone_outbox -- create 5   # terminal 3
```

## المشكلة: Dual Write

```js
// Node — الكود ده فيه bug حتى لو شكله سليم
await db.query('INSERT INTO orders ...');
await channel.publish('events', 'order.created', payload);   // 💥 لو وقع هنا؟
```

- الـ insert نجح والـ publish فشل → الـ order موجود ومحدش عرف (مفيش email، مفيش خصم stock).
- ولو عكست الترتيب → event اتنشر لـ order مش موجود.

مفيش transaction واحدة بتجمع Postgres و RabbitMQ. **المشكلة دي مش مشكلة Rust ولا Node — دي مشكلة distributed systems.**

## الحل: Transactional Outbox

```
┌──────────── Postgres transaction ────────────┐
│ INSERT INTO orders ...                       │
│ INSERT INTO outbox (event_id, payload) ...   │   ← الاتنين أو ولا حاجة
└──────────────────────────────────────────────┘
                    │
        relay: SELECT ... FROM outbox WHERE published_at IS NULL
               FOR UPDATE SKIP LOCKED
                    │ publish + wait for confirm
                    │ UPDATE outbox SET published_at = now()
                    ▼
               [RabbitMQ] ──► worker
```

## وده بيخلّي الرسايل ممكن تتكرر

لو الـ relay نشر ووقع قبل الـ `UPDATE published_at` → هيعيد النشر. عشان كده:

## Idempotent Consumer

```sql
INSERT INTO processed_events (event_id) VALUES ($1) ON CONFLICT DO NOTHING
-- rows_affected = 0 → اتعالجت قبل كده → skip + ack
-- rows_affected = 1 → أول مرة → اعمل الشغل في نفس الـ transaction → commit → ack
```

الـ INSERT والشغل الحقيقي في **نفس الـ transaction**. لو الشغل فشل، الـ INSERT بيتعمله rollback، والرسالة لما ترجع هتتعالج عادي.

## النتيجة: Effectively-Once

- **Outbox** → الـ event مش هيضيع أبداً (at-least-once publishing).
- **Idempotency** → التكرار مش بيأثر.
- = كل order بيتأكد **مرة واحدة بالظبط** في الـ effect النهائي.

## اللي اتعلمته في المشروع ده

| المفهوم | فين |
|---|---|
| modules جوه ملف (`mod db`, `mod bus`, `mod commands`) | الدرس 08 |
| `anyhow` + `.context()` | الدرس 05 |
| `tokio::select!` + `interval` + `ctrl_c` | الدرس 11 |
| serde + JSONB | الدرس 12 |
| transactions + `FOR UPDATE SKIP LOCKED` + `ON CONFLICT` | الدرس 13 |
| topic exchange + confirms + manual ack | الدروس 15-16 |

## جرّب بنفسك

1. شغّل `create 5` من غير relay. بص على جدول `outbox`. بعدين شغّل الـ relay.
2. اعمل duplicate يدوي: `UPDATE outbox SET published_at = NULL;` والـ relay شغال → شوف الـ worker بيطبع ♻️ duplicate.
3. شغّل relay اتنين في نفس الوقت — `SKIP LOCKED` بيمنع إنهم يبعتوا نفس الـ row.
4. زوّد service تاني (worker تاني بـ queue تانية) بيعمل "إرسال email" على نفس الـ event.
5. **التحدي الكبير:** حوّل المثال ده لـ workspace زي `test-proj`: crate `contracts` (الـ events)، crate `orders-api` (axum + create)، crate `relay`، crate `worker`.
