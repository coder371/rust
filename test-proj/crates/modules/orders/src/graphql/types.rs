use crate::domain::Order;
use async_graphql::{InputObject, SimpleObject};

/// أنواع الواجهة منفصلة عن الكيان: تغيير الـ domain لا يكسر العقد
/// المنشور للعملاء، وإضافة حقل للواجهة لا تُلزم بتغيير التخزين.
#[derive(SimpleObject)]
pub struct OrderDto {
    pub id: String,
    pub customer_id: String,
    pub status: String,
    pub total_minor: i64,
    pub currency: String,
    pub placed_at_ms: i64,
    pub version: u32,
    pub lines: Vec<OrderLineDto>,
}

#[derive(SimpleObject)]
pub struct OrderLineDto {
    pub product_id: String,
    pub quantity: u32,
    pub unit_price_minor: i64,
}

impl From<Order> for OrderDto {
    fn from(o: Order) -> Self {
        Self {
            id: o.id.to_string(),
            customer_id: o.customer_id.to_string(),
            status: o.status.as_str().to_string(),
            total_minor: o.total.minor,
            currency: o.total.currency.code().to_string(),
            placed_at_ms: o.placed_at_ms,
            version: o.version,
            lines: o
                .lines
                .into_iter()
                .map(|l| OrderLineDto {
                    product_id: l.product_id.to_string(),
                    quantity: l.quantity,
                    unit_price_minor: l.unit_price.minor,
                })
                .collect(),
        }
    }
}

#[derive(InputObject)]
pub struct PlaceOrderInput {
    pub customer_id: String,
    pub lines: Vec<PlaceOrderLineInput>,
}

#[derive(InputObject)]
pub struct PlaceOrderLineInput {
    pub product_id: String,
    pub quantity: u32,
    pub unit_price_minor: i64,
}
