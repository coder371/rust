# 16 — RabbitMQ Patterns للـ Production

📄 الكود: [`examples/16_rabbit_patterns.rs`](../examples/16_rabbit_patterns.rs)

```bash
cargo run --example 16_rabbit_patterns -- worker billing 'order.*'   # terminal 1
cargo run --example 16_rabbit_patterns -- worker audit '#'           # terminal 2
cargo run --example 16_rabbit_patterns -- publish                    # terminal 3
```

## 1) Topic Exchange = Pub/Sub

```
                      ┌── binding "order.*" ──► lesson16.billing ──► billing worker
publish "order.paid" ─► [shop.events] ──┤
                      └── binding "#" ────────► lesson16.audit   ──► audit worker
```

- كل **service** ليه **queue خاصة بيه** → كل واحد بياخد نسخة من الرسالة.
- كذا **instance** من نفس الـ service على **نفس الـ queue** → بيتقسموا الشغل.
- `*` = كلمة واحدة، `#` = صفر أو أكتر.

## 2) Retry بتأخير (من غير plugins)

المشكلة: لو عملت `nack(requeue=true)` والـ DB واقعة، الرسالة هترجع فوراً وتفشل تاني فوراً → loop بيحرق CPU.

الحل: **retry queue بـ TTL**:
1. الـ handler فشل → انشر الرسالة في `*.retry` مع header `x-retries+1` → ack للأصلية.
2. الـ `*.retry` مفيهاش consumer. بعد `x-message-ttl` (3 ثواني) الرسالة بتنتهي.
3. الـ `x-dead-letter-*` بتاع الـ retry queue بيرجّعها للـ main queue.
4. بعد `MAX_RETRIES` → الرسالة تروح **DLQ** وتقعد هناك لحد ما حد يبص عليها.

## 3) DLQ (Dead Letter Queue)

الـ main queue معمولها `x-dead-letter-routing-key = *.dlq`. أي `nack(requeue=false)` (زي JSON بايظ) بيروح هناك تلقائي. **مفيش رسالة بتضيع، ومفيش رسالة بتعمل infinite loop.**

## 4) Concurrency

```rust
ch.basic_qos(4, ...)          // الـ broker يبعت لحد 4 رسايل من غير ack
tasks.spawn(async move { handle(...).await })   // كل رسالة في task
```
الـ prefetch هو الـ concurrency limit الطبيعي. مش محتاج Semaphore.

في Node: `ch.prefetch(4)` + الـ callback async — نفس الفكرة. الفرق إن tokio هيوزّعهم على كل الـ cores.

## 5) Graceful Shutdown

```rust
tokio::select! {
    _ = ctrl_c => break,               // جه signal
    next = consumer.next() => { ... }  // جت رسالة
}
ch.basic_cancel(tag, ...).await?;      // 1) بطّل تستقبل
while tasks.join_next().await.is_some() {}   // 2) استنى اللي شغال يخلص
ch.close(...).await?;                  // 3) اقفل
```

ده مهم جداً في Kubernetes: لما pod بيتقفل بيجيله `SIGTERM`، ولو قفلت فجأة الرسايل اللي في النص هترجع وتتعالج تاني (مش هتضيع، بس duplicate). (في production استخدم `tokio::signal::unix::signal(SignalKind::terminate())` كمان.)

## فصل الـ business logic عن الـ infrastructure

لاحظ إن `process()` مش عارفة أي حاجة عن RabbitMQ. بتاخد `&Event` وترجع `Result`. و`handle()` هي الـ glue (parse → process → ack/retry/dlq). ده بيخليك تعمل test لـ `process()` من غير broker.

## جرّب بنفسك

1. شغّل worker الـ billing بس، وابعت. شوف `order.paid` بيفشل ويرجع بعد 3 ثواني وينجح.
2. شوف `order.cancelled` بيروح `lesson16.billing.dlq` بعد 3 محاولات (لوحة التحكم → Queues → Get messages وشوف الـ headers).
3. ابعت 20 event واقفل الـ worker في النص بـ Ctrl+C. عدّ اللي اتعالج.
4. اكتب أمر `replay-dlq <name>` بيسحب كل الرسايل من الـ DLQ ويرجّعها للـ main queue (بـ `basic_get`).
