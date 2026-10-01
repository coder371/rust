//! الدرس 15 (أ) — RabbitMQ: الـ Publisher
//! قبل ما تشغّل:  docker compose up -d rabbitmq
//! شغّل الـ consumer الأول في terminal:   cargo run --example 15_rabbit_consumer
//! وبعدين في terminal تاني:              cargo run --example 15_rabbit_publisher -- 5
//! لوحة التحكم: http://localhost:15672  (guest/guest) — شوف الـ queue والرسايل
//!
//! ═══ المقارنة مع Node (amqplib) ═══
//!   amqp.connect(url)                 → Connection::connect(url, props)
//!   conn.createConfirmChannel()       → conn.create_channel() + confirm_select()
//!   ch.assertQueue(q, {durable:true}) → queue_declare(q, QueueDeclareOptions{durable:true,..}, args)
//!   ch.sendToQueue(q, Buffer, opts)   → basic_publish("", q, opts, &bytes, props)
//!   ch.consume(q, cb)                 → basic_consume(...) بيرجع Stream — بتلف عليه بـ while let
//!   msg.content                       → delivery.data  (Vec<u8>)
//!   ch.ack(msg)                       → delivery.ack(...)
//!
//! الفرق الفلسفي: في amqplib الـ consume بياخد callback ← push model.
//! في lapin الـ Consumer عبارة عن Stream ← pull model: إنت اللي بتسحب الرسالة الجاية.
//! ده بيخليك تتحكم في الـ concurrency والـ backpressure بنفسك بشكل واضح.

use lapin::options::{BasicPublishOptions, ConfirmSelectOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Connection, ConnectionProperties};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const QUEUE: &str = "lesson15.tasks";

// الـ contract بين الـ publisher والـ consumer. في مشروع حقيقي بيتحط في crate مشتركة
// (زي crates/contracts في test-proj عندك) عشان الاتنين يستخدموا نفس النوع.
#[derive(Debug, Serialize, Deserialize)]
struct SendEmailTask {
    id: Uuid,
    to: String,
    subject: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let count: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);
    let url = std::env::var("AMQP_URL")
        .unwrap_or_else(|_| "amqp://guest:guest@localhost:5672/%2f".into());

    // ═══ Connection و Channel ═══
    // Connection = TCP connection واحدة (غالية) → واحدة للتطبيق كله
    // Channel    = virtual connection جواها (رخيصة) → واحد لكل "استخدام" (publisher / consumer)
    // timeout(...) بيرجع Result<Result<Connection, lapin::Error>, Elapsed>
    // فـ `??`: الأولى للـ timeout، والتانية لـ error الـ connect نفسه
    let conn = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        Connection::connect(&url, ConnectionProperties::default()),
    )
    .await
    .map_err(|_| {
        anyhow::anyhow!("RabbitMQ مش بيرد على {url} — شغّلت docker compose up -d rabbitmq؟")
    })??;
    let channel = conn.create_channel().await?;

    // Publisher confirms: الـ broker بيأكدلك إنه استلم الرسالة وخزّنها.
    // من غيرها basic_publish بتبعت وخلاص — ممكن الرسالة تضيع لو الـ broker وقع.
    channel
        .confirm_select(ConfirmSelectOptions::default())
        .await?;

    // الـ declare idempotent: لو موجود بنفس الإعدادات مش بيعمل حاجة. الاتنين (pub/sub) بيعملوه.
    // durable = الـ queue نفسها تعيش بعد restart للـ broker
    channel
        .queue_declare(
            QUEUE.into(),
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    for i in 1..=count {
        let task = SendEmailTask {
            id: Uuid::new_v4(),
            to: format!("user{i}@x.com"),
            subject: format!("Order #{i}"),
        };
        let payload = serde_json::to_vec(&task)?;

        let confirm = channel
            .basic_publish(
                "".into(),    // "" = default exchange: بيوصل للـ queue اللي اسمها = الـ routing key
                QUEUE.into(), // routing key
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default()
                    .with_content_type("application/json".into())
                    .with_delivery_mode(2) // 2 = persistent: الرسالة تتخزن على الديسك
                    .with_message_id(task.id.to_string().into()),
            )
            .await? // ← الرسالة اتبعتت
            .await?; // ← استنينا الـ confirm من الـ broker
        println!(
            "📤 published {} → {} (ack from broker: {})",
            task.id,
            task.to,
            confirm.is_ack()
        );
    }

    conn.close(200, "bye".into()).await?;
    Ok(())
}
