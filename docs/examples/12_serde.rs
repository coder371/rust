//! الدرس 12 — JSON و Serde
//! شغّل:  cargo run --example 12_serde
//!
//! في Node: JSON.parse بيرجع any. لو عايز أمان بتستخدم zod/joi/class-validator.
//! في Rust: serde بيحوّل JSON ↔ struct مباشرة، والـ parse نفسه هو الـ validation:
//!          لو field ناقص أو نوعه غلط → Err واضح. مفيش "undefined is not a function".
//!
//! serde مهم جداً للجاي: الـ rows من الـ DB، الـ documents في Mongo، والرسايل في RabbitMQ
//! كلها هتعدي من هنا.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")] // Rust بتستخدم snake_case، والـ JSON غالباً camelCase
struct CreateOrder {
    customer_id: Uuid,
    items: Vec<Item>,
    #[serde(default)] // لو مش موجود → Default (هنا false)
    express: bool,
    #[serde(skip_serializing_if = "Option::is_none")] // متحطهوش في الـ JSON لو None
    coupon: Option<String>,
    #[serde(default = "Utc::now")]
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Item {
    sku: String,
    qty: u32, // لو جالك -1 أو "3" → error
    #[serde(rename = "unit_price")]
    price: f64,
}

// enum بـ tag — ده الشكل المثالي للـ events في RabbitMQ
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
enum Event {
    OrderCreated { order_id: Uuid, total: f64 },
    OrderCancelled { order_id: Uuid, reason: String },
    Ping,
}

fn main() -> anyhow::Result<()> {
    println!("── JSON → struct ──");
    let body = r#"{
        "customerId": "7b8e6f1c-2a3d-4b5e-8f90-123456789abc",
        "items": [{ "sku": "TSHIRT", "qty": 2, "unit_price": 150.0 }]
    }"#;
    let order: CreateOrder = serde_json::from_str(body)?;
    println!("{order:#?}");

    println!("── struct → JSON ──");
    println!("{}", serde_json::to_string_pretty(&order)?); // لاحظ coupon مش موجود

    println!("── validation ببلاش ──");
    let bad_inputs = [
        r#"{"items": []}"#,                             // customerId ناقص
        r#"{"customerId": "not-a-uuid", "items": []}"#, // uuid غلط
        r#"{"customerId": "7b8e6f1c-2a3d-4b5e-8f90-123456789abc", "items": [{"sku":"A","qty":-1,"unit_price":1}]}"#,
    ];
    for b in bad_inputs {
        match serde_json::from_str::<CreateOrder>(b) {
            Ok(_) => println!("  ok?!"),
            Err(e) => println!("  ❌ {e}"),
        }
    }

    println!("── tagged enums (events) ──");
    let events = vec![
        Event::OrderCreated {
            order_id: Uuid::new_v4(),
            total: 300.0,
        },
        Event::OrderCancelled {
            order_id: Uuid::nil(),
            reason: "out of stock".into(),
        },
        Event::Ping,
    ];
    for e in &events {
        let s = serde_json::to_string(e)?;
        println!("  {s}");
        let back: Event = serde_json::from_str(&s)?;
        if let Event::OrderCreated { total, .. } = back {
            println!("    ↳ parsed back, total={total}")
        }
    }

    println!("── dynamic JSON (زي any) ──");
    // لما مش عارف الشكل مسبقاً: serde_json::Value
    let v: Value = json!({ "user": { "name": "Ahmed", "tags": ["a", "b"] }, "n": 5 });
    println!("  name = {}", v["user"]["name"]); // "Ahmed" (بالـ quotes لأنه Value)
    println!("  name str = {:?}", v["user"]["name"].as_str());
    println!("  missing = {}", v["nope"]["deep"]); // null — مش crash
    println!("  pointer = {:?}", v.pointer("/user/tags/1"));

    // bytes (زي payload الرسالة في RabbitMQ)
    let bytes: Vec<u8> = serde_json::to_vec(&events[0])?;
    let parsed: Event = serde_json::from_slice(&bytes)?;
    println!("  from bytes: {parsed:?}");
    Ok(())
}
