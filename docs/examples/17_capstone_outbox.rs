//! الدرس 17 — مشروع صغير: Postgres + RabbitMQ مع بعض (Transactional Outbox + Idempotent Consumer)
//! قبل ما تشغّل:  docker compose up -d
//!
//!   terminal 1:  cargo run --example 17_capstone_outbox -- worker
//!   terminal 2:  cargo run --example 17_capstone_outbox -- relay
//!   terminal 3:  cargo run --example 17_capstone_outbox -- create 5
//!
//! ═══ المشكلة (موجودة في Node بالظبط) ═══
//!   await db.orders.insert(order)          ✅
//!   await channel.publish('order.created') 💥 الـ process وقعت هنا
//! → الـ order اتسجل بس محدش في الـ system عرف. أو العكس: نشرت event والـ DB عملت rollback.
//! مفيش "transaction" بتجمع Postgres و RabbitMQ.
//!
//! ═══ الحل: Transactional Outbox ═══
//!   1) create: في نفس الـ DB transaction → INSERT order + INSERT outbox(event)   (يا الاتنين يا ولا حاجة)
//!   2) relay:  loop بيقرا outbox اللي لسه متنشرش → publish (مع confirm) → UPDATE published_at
//!   3) worker: بيستقبل الـ event. ممكن ييجي مرتين (relay وقع بعد publish وقبل update) →
//!      لازم idempotent: جدول processed_events بـ PRIMARY KEY على event_id
//!
//! وده كمان مثال على تنظيم الكود في modules (الدرس 08): db / bus / commands.

use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const EXCHANGE: &str = "lesson17.events";
const QUEUE: &str = "lesson17.order-confirmer";

/// الـ contract المشترك بين الـ producer والـ consumer
#[derive(Debug, Serialize, Deserialize)]
struct OrderCreated {
    event_id: Uuid,
    order_id: Uuid,
    user_id: Uuid,
    total: i64,
}

// ─────────────────────────── infra ───────────────────────────
mod db {
    use sqlx::postgres::{PgPool, PgPoolOptions};

    pub async fn connect() -> anyhow::Result<PgPool> {
        let url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://app:app@localhost:5432/learn".into());
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(pool)
    }
}

mod bus {
    use lapin::options::*;
    use lapin::types::FieldTable;
    use lapin::{Channel, Connection, ConnectionProperties, ExchangeKind};

    pub async fn connect() -> anyhow::Result<(Connection, Channel)> {
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
        let ch = conn.create_channel().await?;
        ch.confirm_select(ConfirmSelectOptions::default()).await?;
        ch.exchange_declare(
            super::EXCHANGE.into(),
            ExchangeKind::Topic,
            ExchangeDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;
        Ok((conn, ch))
    }
}

// ─────────────────────────── commands ───────────────────────────
mod commands {
    use super::*;
    use futures::StreamExt;
    use lapin::BasicProperties;
    use lapin::options::*;
    use lapin::types::FieldTable;
    use sqlx::{FromRow, PgPool};
    use std::time::Duration;

    /// (1) إنشاء orders — الـ order والـ event في transaction واحدة
    pub async fn create(pool: &PgPool, n: usize) -> anyhow::Result<()> {
        // user للتجربة
        let user_id = Uuid::new_v4();
        sqlx::query("INSERT INTO users (id, email, name, balance) VALUES ($1, $2, 'Capstone', 0)")
            .bind(user_id)
            .bind(format!("{user_id}@capstone.test"))
            .execute(pool)
            .await?;

        for i in 1..=n {
            let order_id = Uuid::new_v4();
            let total = 100 * i as i64;
            let event = OrderCreated {
                event_id: Uuid::new_v4(),
                order_id,
                user_id,
                total,
            };

            let mut tx = pool.begin().await?;
            sqlx::query("INSERT INTO orders (id, user_id, total) VALUES ($1, $2, $3)")
                .bind(order_id)
                .bind(user_id)
                .bind(total)
                .execute(&mut *tx)
                .await?;
            sqlx::query("INSERT INTO outbox (event_id, routing_key, payload) VALUES ($1, 'order.created', $2)")
                .bind(event.event_id)
                .bind(sqlx::types::Json(&event))
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            println!(
                "📝 order {order_id} total={total} (+ outbox event {})",
                event.event_id
            );
        }
        Ok(())
    }

    #[derive(FromRow)]
    struct OutboxRow {
        id: i64,
        event_id: Uuid,
        routing_key: String,
        payload: sqlx::types::Json<serde_json::Value>,
    }

    /// (2) الـ relay: outbox → RabbitMQ
    pub async fn relay(pool: &PgPool, ch: &lapin::Channel) -> anyhow::Result<()> {
        println!("🔁 relay running (Ctrl+C للخروج)");
        let mut tick = tokio::time::interval(Duration::from_millis(500));
        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => { println!("👋 relay stopped"); return Ok(()) }
                _ = tick.tick() => {}
            }
            let mut tx = pool.begin().await?;
            // SKIP LOCKED → لو شغّلت كذا relay مع بعض، كل واحد ياخد rows مختلفة (مفيش تكرار ولا انتظار)
            let rows: Vec<OutboxRow> = sqlx::query_as(
                "SELECT id, event_id, routing_key, payload FROM outbox
                 WHERE published_at IS NULL ORDER BY id LIMIT 50 FOR UPDATE SKIP LOCKED",
            )
            .fetch_all(&mut *tx)
            .await?;

            for row in &rows {
                ch.basic_publish(
                    EXCHANGE.into(),
                    row.routing_key.as_str().into(),
                    BasicPublishOptions::default(),
                    &serde_json::to_vec(&row.payload.0)?,
                    BasicProperties::default()
                        .with_delivery_mode(2)
                        .with_content_type("application/json".into())
                        .with_message_id(row.event_id.to_string().into()),
                )
                .await?
                .await?; // confirm
                sqlx::query("UPDATE outbox SET published_at = now() WHERE id = $1")
                    .bind(row.id)
                    .execute(&mut *tx)
                    .await?;
                println!(
                    "📤 relayed outbox#{} {} {}",
                    row.id, row.routing_key, row.event_id
                );
            }
            tx.commit().await?;
        }
    }

    /// (3) الـ worker: بيأكد الـ order — idempotent
    pub async fn worker(pool: &PgPool, ch: &lapin::Channel) -> anyhow::Result<()> {
        ch.queue_declare(
            QUEUE.into(),
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;
        ch.queue_bind(
            QUEUE.into(),
            EXCHANGE.into(),
            "order.created".into(),
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;
        ch.basic_qos(10, BasicQosOptions::default()).await?;
        let mut consumer = ch
            .basic_consume(
                QUEUE.into(),
                "".into(),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;
        println!("👂 worker waiting on {QUEUE} (Ctrl+C للخروج)");

        loop {
            let delivery = tokio::select! {
                _ = tokio::signal::ctrl_c() => { println!("👋 worker stopped"); return Ok(()) }
                next = consumer.next() => match next { Some(d) => d?, None => return Ok(()) },
            };
            let Ok(ev) = serde_json::from_slice::<OrderCreated>(&delivery.data) else {
                delivery
                    .nack(BasicNackOptions {
                        requeue: false,
                        ..Default::default()
                    })
                    .await?;
                continue;
            };

            match confirm_order(pool, &ev).await {
                Ok(true) => println!("✅ confirmed order {} (event {})", ev.order_id, ev.event_id),
                Ok(false) => println!("♻️ duplicate event {} — skipped", ev.event_id),
                Err(e) => {
                    // error مؤقت (DB وقعت مثلاً) → رجّعها الـ queue تتعالج تاني
                    eprintln!("❌ {e:#} — requeue");
                    delivery
                        .nack(BasicNackOptions {
                            requeue: true,
                            ..Default::default()
                        })
                        .await?;
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            }
            delivery.ack(BasicAckOptions::default()).await?;
        }
    }

    /// true = اتعالج دلوقتي، false = كان متعالج قبل كده
    async fn confirm_order(pool: &PgPool, ev: &OrderCreated) -> anyhow::Result<bool> {
        let mut tx = pool.begin().await?;
        // الـ INSERT والـ UPDATE في نفس الـ transaction:
        // لو الـ event اتعالج قبل كده → ON CONFLICT → 0 rows → نعرف إنه duplicate
        let inserted = sqlx::query(
            "INSERT INTO processed_events (event_id) VALUES ($1) ON CONFLICT DO NOTHING",
        )
        .bind(ev.event_id)
        .execute(&mut *tx)
        .await?
        .rows_affected();
        if inserted == 0 {
            return Ok(false); // tx بتعمل rollback لوحدها (ومفيش حاجة تترجع أصلاً)
        }
        sqlx::query("UPDATE orders SET status = 'confirmed' WHERE id = $1")
            .bind(ev.order_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(true)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let pool = db::connect()
        .await
        .context("Postgres مش شغال؟ docker compose up -d")?;

    match args.get(1).map(String::as_str) {
        Some("create") => {
            let n = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(3);
            commands::create(&pool, n).await?;
        }
        Some("relay") => {
            let (_conn, ch) = bus::connect().await.context("RabbitMQ مش شغال؟")?;
            commands::relay(&pool, &ch).await?;
        }
        Some("worker") => {
            let (_conn, ch) = bus::connect().await.context("RabbitMQ مش شغال؟")?;
            commands::worker(&pool, &ch).await?;
        }
        _ => bail!("usage: 17_capstone_outbox create [n] | relay | worker"),
    }
    Ok(())
}
