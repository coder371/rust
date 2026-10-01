//! الدرس 16 — RabbitMQ Patterns للـ production
//! قبل ما تشغّل:  docker compose up -d rabbitmq
//!
//! افتح 3 terminals:
//!   1) cargo run --example 16_rabbit_patterns -- worker billing 'order.*'      ← أحداث الـ orders بس
//!   2) cargo run --example 16_rabbit_patterns -- worker audit '#'              ← كل حاجة
//!   3) cargo run --example 16_rabbit_patterns -- publish
//!
//! وبعدين اضغط Ctrl+C على أي worker وشوف الـ graceful shutdown.
//!
//! اللي هتتعلمه:
//!   1) Topic exchange  → pub/sub: رسالة واحدة توصل لكذا service، كل واحد بـ queue خاصة بيه
//!   2) Retry بتأخير    → retry queue بـ TTL + dead-letter بيرجّع الرسالة للـ queue الأصلية
//!   3) DLQ             → الرسايل اللي فشلت نهائياً تتركن في queue تتفحص بعدين (مش تضيع ومش infinite loop)
//!   4) Concurrency     → prefetch N + tokio::spawn لكل رسالة (N رسايل بالتوازي)
//!   5) Graceful shutdown → نوقف استقبال، نستنى الشغل الحالي يخلص، نقفل
//!
//! الـ Topology لـ worker اسمه billing:
//!
//!   publisher ──► [shop.events] (topic)
//!                     │ binding: order.*
//!                     ▼
//!              ┌─► lesson16.billing ──► worker ──(نجح)──► ack
//!              │        │                 │
//!              │        │ nack(requeue=false)   (فشل وعدّى الـ retries)──► lesson16.billing.dlq
//!              │        └──── DLX ──────────────────────────────────────► lesson16.billing.dlq
//!              │                          │ (فشل ولسه فيه محاولات)
//!              │                          ▼
//!              └── TTL 3s ◄── lesson16.billing.retry

use anyhow::bail;
use futures::StreamExt;
use lapin::message::Delivery;
use lapin::options::*;
use lapin::types::{AMQPValue, FieldTable};
use lapin::{BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::task::JoinSet;
use uuid::Uuid;

const EXCHANGE: &str = "shop.events";
const MAX_RETRIES: i32 = 3;
const RETRY_DELAY_MS: i32 = 3_000;
const CONCURRENCY: u16 = 4;

#[derive(Debug, Serialize, Deserialize)]
struct Event {
    id: Uuid,
    kind: String,
    // للتجربة: الـ handler هيفشل أول `fail_times` محاولات
    fail_times: i32,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
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

    match args.get(1).map(String::as_str) {
        Some("publish") => publish(&conn).await?,
        Some("worker") => {
            let name = args.get(2).cloned().unwrap_or_else(|| "billing".into());
            let binding = args.get(3).cloned().unwrap_or_else(|| "order.*".into());
            worker(&conn, &name, &binding).await?
        }
        _ => bail!("usage: 16_rabbit_patterns publish | worker <name> <binding-pattern>"),
    }
    conn.close(200, "bye".into()).await?;
    Ok(())
}

async fn declare_exchange(ch: &Channel) -> lapin::Result<()> {
    ch.exchange_declare(
        EXCHANGE.into(),
        ExchangeKind::Topic,
        ExchangeDeclareOptions {
            durable: true,
            ..Default::default()
        },
        FieldTable::default(),
    )
    .await
}

// ═══════════════════════ Publisher ═══════════════════════
async fn publish(conn: &Connection) -> anyhow::Result<()> {
    let ch = conn.create_channel().await?;
    ch.confirm_select(ConfirmSelectOptions::default()).await?;
    declare_exchange(&ch).await?;

    // routing keys بتتكتب كـ كلمات بينها نقط: <entity>.<action>
    // في الـ binding:  *  = كلمة واحدة بالظبط  |  # = صفر أو أكتر من الكلمات
    let events = [
        ("order.created", 0),   // هينجح من أول مرة
        ("order.paid", 1),      // هيفشل مرة، ويرجع بعد 3 ثواني وينجح
        ("order.cancelled", 9), // هيفشل على طول → بعد 3 retries يروح الـ DLQ
        ("user.registered", 0), // billing مش هيشوفه (order.*)، audit هيشوفه (#)
    ];
    for (key, fail_times) in events {
        let ev = Event {
            id: Uuid::new_v4(),
            kind: key.into(),
            fail_times,
        };
        ch.basic_publish(
            EXCHANGE.into(),
            key.into(),
            BasicPublishOptions::default(),
            &serde_json::to_vec(&ev)?,
            BasicProperties::default()
                .with_delivery_mode(2)
                .with_content_type("application/json".into()),
        )
        .await?
        .await?;
        println!("📤 {key} (fail_times={fail_times}) id={}", ev.id);
    }
    Ok(())
}

// ═══════════════════════ Worker ═══════════════════════
struct Queues {
    main: String,
    retry: String,
    dlq: String,
}

async fn declare_topology(ch: &Channel, name: &str, binding: &str) -> anyhow::Result<Queues> {
    declare_exchange(ch).await?;
    let q = Queues {
        main: format!("lesson16.{name}"),
        retry: format!("lesson16.{name}.retry"),
        dlq: format!("lesson16.{name}.dlq"),
    };
    let durable = QueueDeclareOptions {
        durable: true,
        ..Default::default()
    };

    // DLQ: queue عادية، محدش بيستهلك منها أوتوماتيك — بتتفحص يدوي أو بـ tool
    ch.queue_declare(q.dlq.as_str().into(), durable, FieldTable::default())
        .await?;

    // Main queue: أي رسالة تتعمل nack(requeue=false) → تروح الـ DLQ
    // (الـ default exchange "" + routing key = اسم الـ DLQ)
    let mut main_args = FieldTable::default();
    main_args.insert(
        "x-dead-letter-exchange".into(),
        AMQPValue::LongString("".into()),
    );
    main_args.insert(
        "x-dead-letter-routing-key".into(),
        AMQPValue::LongString(q.dlq.as_str().into()),
    );
    ch.queue_declare(q.main.as_str().into(), durable, main_args)
        .await?;
    ch.queue_bind(
        q.main.as_str().into(),
        EXCHANGE.into(),
        binding.into(),
        QueueBindOptions::default(),
        FieldTable::default(),
    )
    .await?;

    // Retry queue: مفيش consumer عليها. الرسالة بتقعد RETRY_DELAY_MS وبعدين تنتهي (expire)
    // فبتتعمل dead-letter → ترجع للـ main queue. ده "delayed retry" من غير plugins.
    let mut retry_args = FieldTable::default();
    retry_args.insert("x-message-ttl".into(), AMQPValue::LongInt(RETRY_DELAY_MS));
    retry_args.insert(
        "x-dead-letter-exchange".into(),
        AMQPValue::LongString("".into()),
    );
    retry_args.insert(
        "x-dead-letter-routing-key".into(),
        AMQPValue::LongString(q.main.as_str().into()),
    );
    ch.queue_declare(q.retry.as_str().into(), durable, retry_args)
        .await?;

    Ok(q)
}

async fn worker(conn: &Connection, name: &str, binding: &str) -> anyhow::Result<()> {
    let ch = conn.create_channel().await?;
    // confirms عشان لما نعيد نشر رسالة (retry/DLQ) نتأكد إنها وصلت قبل ما نعمل ack للأصلية
    ch.confirm_select(ConfirmSelectOptions::default()).await?;
    let queues = declare_topology(&ch, name, binding).await?;

    // prefetch = CONCURRENCY → الـ broker يبعتلنا لحد 4 رسايل من غير ack
    ch.basic_qos(CONCURRENCY, BasicQosOptions::default())
        .await?;
    let tag = format!("{name}-{}", &Uuid::new_v4().to_string()[..8]);
    let mut consumer = ch
        .basic_consume(
            queues.main.as_str().into(),
            tag.as_str().into(),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;
    println!(
        "👂 [{name}] bound with '{binding}' on {} (concurrency={CONCURRENCY})",
        queues.main
    );

    let mut tasks = JoinSet::new();
    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            // لو جه Ctrl+C: نخرج من الـ loop
            _ = &mut shutdown => {
                println!("\n🛑 [{name}] shutdown requested — stop consuming...");
                break;
            }
            next = consumer.next() => {
                let Some(delivery) = next else { break };
                let delivery = delivery?;
                let ch = ch.clone(); // Channel رخيص في الـ clone
                let retry_q = queues.retry.clone();
                let dlq = queues.dlq.clone();
                let name = name.to_string();
                // كل رسالة في task لوحدها → لحد CONCURRENCY بالتوازي (الـ prefetch هو الحد)
                tasks.spawn(async move {
                    if let Err(e) = handle(&ch, &name, delivery, &retry_q, &dlq).await {
                        eprintln!("[{name}] handler infra error: {e:#}");
                    }
                });
            }
            // ننضّف الـ tasks اللي خلصت عشان الـ JoinSet ميكبرش
            Some(_) = tasks.join_next(), if !tasks.is_empty() => {}
        }
    }

    // Graceful shutdown:
    // 1) نلغي الـ consumer → الـ broker يبطّل يبعت رسايل جديدة
    ch.basic_cancel(tag.as_str().into(), BasicCancelOptions::default())
        .await?;
    // 2) نستنى الرسايل اللي في النص تخلص وتتعمل ack
    println!(
        "⏳ [{name}] waiting for {} in-flight message(s)...",
        tasks.len()
    );
    while tasks.join_next().await.is_some() {}
    // 3) أي رسالة كانت اتبعتت ومتعملش لها ack هترجع للـ queue لما الـ channel يتقفل
    ch.close(200, "shutdown".into()).await?;
    println!("👋 [{name}] bye");
    Ok(())
}

fn retries_of(d: &Delivery) -> i32 {
    d.properties
        .headers()
        .as_ref()
        .and_then(|h| h.inner().get("x-retries").cloned())
        .and_then(|v| match v {
            AMQPValue::LongInt(n) => Some(n),
            _ => None,
        })
        .unwrap_or(0)
}

/// الـ business logic الحقيقي — مش عارف أي حاجة عن RabbitMQ
async fn process(name: &str, ev: &Event, attempt: i32) -> Result<(), String> {
    tokio::time::sleep(Duration::from_millis(300)).await;
    if attempt < ev.fail_times {
        return Err(format!("simulated failure #{}", attempt + 1));
    }
    println!("   ✅ [{name}] processed {} {}", ev.kind, ev.id);
    Ok(())
}

/// الـ glue بين RabbitMQ والـ business logic: parse → process → ack / retry / dlq
async fn handle(
    ch: &Channel,
    name: &str,
    d: Delivery,
    retry_q: &str,
    dlq: &str,
) -> anyhow::Result<()> {
    let ev: Event = match serde_json::from_slice(&d.data) {
        Ok(ev) => ev,
        Err(e) => {
            eprintln!("   ☠️ [{name}] bad payload ({e}) → DLQ");
            d.nack(BasicNackOptions {
                requeue: false,
                ..Default::default()
            })
            .await?; // DLX → DLQ
            return Ok(());
        }
    };
    let attempt = retries_of(&d);
    println!(
        "📥 [{name}] {} attempt={} routing_key={}",
        ev.kind,
        attempt + 1,
        d.routing_key
    );

    match process(name, &ev, attempt).await {
        Ok(()) => {
            d.ack(BasicAckOptions::default()).await?;
        }
        Err(err) => {
            let (target, next) = if attempt + 1 >= MAX_RETRIES {
                (dlq, "DLQ ☠️")
            } else {
                (retry_q, "retry ⏳")
            };
            println!("   ❌ [{name}] {err} → {next}");
            let mut headers = d.properties.headers().clone().unwrap_or_default();
            headers.insert("x-retries".into(), AMQPValue::LongInt(attempt + 1));
            headers.insert(
                "x-last-error".into(),
                AMQPValue::LongString(err.as_str().into()),
            );
            // ننشر النسخة الجديدة الأول، وبعد ما الـ broker يأكد → ack للأصلية.
            // لو وقعنا في النص: الأصلية هترجع (duplicate ممكن، ضياع لأ) — at-least-once.
            ch.basic_publish(
                "".into(),
                target.into(),
                BasicPublishOptions::default(),
                &d.data,
                d.properties.clone().with_headers(headers),
            )
            .await? // اتبعتت
            .await?; // الـ broker أكد (confirm_select متفعّل على الـ channel)
            d.ack(BasicAckOptions::default()).await?;
        }
    }
    Ok(())
}
