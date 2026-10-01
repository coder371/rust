//! عقود الأحداث بين الموديولات.
//!
//! القاعدة الحاكمة: المستهلك لا يستورد نوعاً من crate الناشر أبداً —
//! الطرفان يعتمدان على هذا الـ crate وحده، فلا يوجد بينهما ارتباط بناء.
use serde::{Deserialize, Serialize};

/// الغلاف المشترك لكل رسالة على الناقل.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<T> {
    pub event_id: String,
    pub routing_key: String,
    pub store_id: String,
    pub occurred_at_ms: i64,
    pub payload: T,
}

pub mod orders {
    use serde::{Deserialize, Serialize};

    // الإصدار جزء من مفتاح التوجيه: V2 يُنشر بالتوازي حتى يهاجر كل المستهلكين
    pub const ORDER_CREATED: &str = "orders.created.v1";
    pub const ORDER_PAID: &str = "orders.paid.v1";

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OrderCreatedV1 {
        pub order_id: String,
        pub customer_id: String,
        pub total_minor: i64,
        pub currency: String,
        pub line_count: usize,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OrderPaidV1 {
        pub order_id: String,
        pub customer_id: String,
        pub total_minor: i64,
        pub currency: String,
    }
}
