//! خدمة الأوردرات. بتقرأ الأوردرات بس — بيانات العميل نفسه شغل `svc-identity`،
//! واللي عايز الاتنين مع بعض بيعدّي على الهَب في `services`.

use std::sync::Arc;

use async_trait::async_trait;
use kernel::order::OrderRepo;
use service_runtime::{Service, ServiceHealth};

mod read;

pub struct OrdersService {
    pub(crate) orders: Arc<dyn OrderRepo>,
}

impl OrdersService {
    pub fn new(orders: Arc<dyn OrderRepo>) -> Self {
        Self { orders }
    }
}

#[async_trait]
impl Service for OrdersService {
    fn name(&self) -> &'static str {
        "orders"
    }

    fn description(&self) -> &'static str {
        "الأوردرات وتوزيعها على الشركاء"
    }

    async fn health(&self) -> ServiceHealth {
        match self.orders.list(kernel::Page::new(0, 1)).await {
            Ok(rows) if rows.is_empty() => ServiceHealth::degraded(self.name(), "no orders"),
            Ok(_) => ServiceHealth::up(self.name(), "reachable"),
            Err(err) => ServiceHealth::down(self.name(), &err),
        }
    }
}
