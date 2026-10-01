# 15 — RabbitMQ: الأساسيات

📄 الكود: [`examples/15_rabbit_publisher.rs`](../examples/15_rabbit_publisher.rs) · [`examples/15_rabbit_consumer.rs`](../examples/15_rabbit_consumer.rs)

```bash
docker compose up -d rabbitmq
cargo run --example 15_rabbit_consumer            # terminal 1
cargo run --example 15_rabbit_publisher -- 5      # terminal 2
# لوحة التحكم: http://localhost:15672  (guest / guest)
```

## المكتبة

| Node | Rust |
|---|---|
| `amqplib` | **`lapin`** ⭐ (الأشهر — نفس اللي في test-proj) |
| `amqp-connection-manager` | `lapin` + `enable_auto_recover()` |

## Push مقابل Pull

```js
// amqplib: callback (push) — الرسايل بتترمي عليك
ch.consume('tasks', async (msg) => {
  await handle(msg);
  ch.ack(msg);
});
```
```rust
// lapin: Stream (pull) — إنت بتسحب الرسالة الجاية
let mut consumer = ch.basic_consume(...).await?;
while let Some(delivery) = consumer.next().await {
    let delivery = delivery?;
    handle(&delivery).await?;
    delivery.ack(BasicAckOptions::default()).await?;
}
```

الـ pull model أوضح: إنت اللي بتقرر تعالج واحدة واحدة، ولا تعمل `spawn` لكل واحدة (الدرس 16)، ولا توقف (graceful shutdown).

## المفاهيم الأساسية (زي Node بالظبط — دي مفاهيم AMQP مش Rust)

| المفهوم | الشرح |
|---|---|
| **Connection** | TCP connection واحدة. غالية. واحدة للتطبيق. |
| **Channel** | اتصال افتراضي جوه الـ connection. رخيص. واحد للـ publisher وواحد لكل consumer. |
| **Exchange** | بيستقبل الرسايل ويوزّعها. `""` = default exchange (بيبعت للـ queue اللي اسمها = routing key). |
| **Queue** | المكان اللي الرسايل بتستنى فيه. |
| **durable** | الـ queue تعيش بعد restart للـ broker. |
| **delivery_mode = 2** | الرسالة نفسها تتخزن على الديسك (persistent). |
| **publisher confirms** | الـ broker يأكدلك إنه استلم (`confirm_select`). من غيرها ممكن الرسالة تضيع. |
| **prefetch (qos)** | أقصى عدد رسايل unacked معاك. |
| **ack** | "خلصت، امسحها". |
| **nack(requeue=true)** | "فشلت، رجّعها". |
| **nack(requeue=false)** | "ارميها" (أو ابعتها للـ DLQ لو متظبط). |

## القاعدة الذهبية: ack بعد الشغل، مش قبله

```
استلم → عالج → ack      ✅ at-least-once: لو وقعت قبل الـ ack، الرسالة هترجع
استلم → ack → عالج      ❌ at-most-once: لو وقعت، الرسالة ضاعت
```

at-least-once معناها إن الرسالة **ممكن تيجي مرتين**. عشان كده الـ consumer لازم يبقى **idempotent** (الدرس 17).

## Contract مشترك

الـ publisher والـ consumer لازم يتفقوا على شكل الرسالة. في Node غالباً بتكون TS type في package مشتركة. في Rust: struct بـ serde في crate مشتركة (زي `crates/contracts` في test-proj). لو حد غيّر الـ struct، الاتنين هيعملوا compile error مع بعض.

## جرّب بنفسك

1. شغّل consumer اتنين وابعت 10 رسايل. شوف التوزيع (round-robin).
2. غيّر الـ prefetch لـ 5 وكرر. إيه اللي اختلف؟
3. اقفل الـ consumer بـ Ctrl+C وهو شغال على رسالة، وشوف في لوحة التحكم إن الرسالة رجعت `Ready`.
4. ابعت رسالة بـ JSON بايظ من لوحة التحكم (Queues → Publish message) وشوف الـ nack.
