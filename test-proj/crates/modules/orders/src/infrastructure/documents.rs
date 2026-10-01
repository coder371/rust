use crate::domain::{Order, OrderLine, OrderStatus};
use qumra_kernel::{Currency, CustomerId, Money, OrderId, ProductId, StoreId};
use serde::{Deserialize, Serialize};

/// شكل التخزين، منفصل عن الكيان عمداً.
/// تغيير مخطط قاعدة البيانات لا يجبرك على تغيير الـ domain، والعكس صحيح.
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderDocument {
    #[serde(rename = "_id")]
    pub id: String,
    pub store_id: String,
    pub customer_id: String,
    pub lines: Vec<LineDocument>,
    pub total_minor: i64,
    pub currency: String,
    pub status: String,
    pub placed_at_ms: i64,
    pub version: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LineDocument {
    pub product_id: String,
    pub quantity: u32,
    pub unit_price_minor: i64,
}

fn currency_code(c: Currency) -> String {
    c.code().to_string()
}

fn parse_currency(s: &str) -> Currency {
    match s {
        "AED" => Currency::Aed,
        "EGP" => Currency::Egp,
        _ => Currency::Sar,
    }
}

fn parse_status(s: &str) -> OrderStatus {
    match s {
        "PAID" => OrderStatus::Paid,
        "CANCELLED" => OrderStatus::Cancelled,
        _ => OrderStatus::Pending,
    }
}

impl From<&Order> for OrderDocument {
    fn from(o: &Order) -> Self {
        Self {
            id: o.id.to_string(),
            store_id: o.store_id.to_string(),
            customer_id: o.customer_id.to_string(),
            lines: o
                .lines
                .iter()
                .map(|l| LineDocument {
                    product_id: l.product_id.to_string(),
                    quantity: l.quantity,
                    unit_price_minor: l.unit_price.minor,
                })
                .collect(),
            total_minor: o.total.minor,
            currency: currency_code(o.total.currency),
            status: o.status.as_str().to_string(),
            placed_at_ms: o.placed_at_ms,
            version: o.version,
        }
    }
}

impl From<OrderDocument> for Order {
    fn from(d: OrderDocument) -> Self {
        let currency = parse_currency(&d.currency);
        Order::rehydrate(
            OrderId::new(d.id),
            StoreId::new(d.store_id),
            CustomerId::new(d.customer_id),
            d.lines
                .into_iter()
                .map(|l| OrderLine {
                    product_id: ProductId::new(l.product_id),
                    quantity: l.quantity,
                    unit_price: Money::new(l.unit_price_minor, currency),
                })
                .collect(),
            Money::new(d.total_minor, currency),
            parse_status(&d.status),
            d.placed_at_ms,
            d.version,
        )
    }
}
