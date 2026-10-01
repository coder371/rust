//! الدرس 15 (ب) — RabbitMQ: الـ Consumer
//! شغّل:  cargo run --example 15_rabbit_consumer
//! جرّب: شغّل نسختين من الـ consumer في نفس الوقت وابعت 10 رسايل → هيتوزعوا عليهم (work queue).
//! جرّب: اقفل الـ consumer بـ Ctrl+C وهو شغال على رسالة → الرسالة مش هتضيع، هترجع للـ queue.

use futures::StreamExt;
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicNackOptions, BasicQosOptions, QueueDeclareOptions,
};
use lapin::types::FieldTable;
use lapin::{Connection, ConnectionProperties};
use serde::Deserialize;
use std::time::Duration;
use uuid::Uuid;

const QUEUE: &str = "lesson15.tasks";

#[derive(Debug, Deserialize)]
struct SendEmailTask {
    id: Uuid,
    to: String,
    subject: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let url = std::env::var("AMQP_URL")
        .unwrap_or_else(|_| "amqp://guest:guest@localhost:5672/%2f".into());
    let conn = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        Connection::connect(&url, ConnectionProperties::default()),
    )
    .await
    .map_err(|_| {
        anyhow::anyhow!("RabbitMQ مش بيرد على {url} — شغّلت docker compose up -d rabbitmq؟")
    })??;
    let channel = conn.create_channel().await?;

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

    // prefetch = أقصى عدد رسايل "مش متأكدة" (unacked) معايا في نفس الوقت.
    // زي ch.prefetch(1) في amqplib. من غيره الـ broker هيرمي عليك الـ queue كلها مرة واحدة.
    channel.basic_qos(1, BasicQosOptions::default()).await?;

    let mut consumer = channel
        .basic_consume(
            QUEUE.into(),
            "lesson15-consumer".into(),
            BasicConsumeOptions::default(), // no_ack: false → manual ack (ده اللي عايزه دايماً تقريباً)
            FieldTable::default(),
        )
        .await?;
    println!("👂 waiting on '{QUEUE}' ... (Ctrl+C للخروج)");

    // الـ Consumer = Stream<Item = Result<Delivery>>
    while let Some(delivery) = consumer.next().await {
        let delivery = delivery?;

        // 1) parse — لو الرسالة بايظة (poison message) مفيش فايدة من إعادة المحاولة
        let task: SendEmailTask = match serde_json::from_slice(&delivery.data) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("☠️ invalid message: {e} — rejecting without requeue");
                // requeue: false → الرسالة تتشال (أو تروح DLQ لو معمول — الدرس 16)
                delivery
                    .nack(BasicNackOptions {
                        requeue: false,
                        ..Default::default()
                    })
                    .await?;
                continue;
            }
        };

        // 2) process
        println!(
            "📥 [{}] sending '{}' to {} (redelivered={})",
            task.id, task.subject, task.to, delivery.redelivered
        );
        tokio::time::sleep(Duration::from_millis(500)).await;

        // 3) ack بعد ما الشغل يخلص — مش قبله! (at-least-once delivery)
        //    لو الـ process وقعت قبل الـ ack، الـ broker هيبعت الرسالة تاني لـ consumer تاني.
        //    النتيجة: الرسالة ممكن تتعالج مرتين → لازم الـ handler يبقى idempotent (الدرس 17).
        delivery.ack(BasicAckOptions::default()).await?;
        println!("   ✅ done");
    }
    Ok(())
}
