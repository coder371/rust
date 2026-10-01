//! خدمة الفلوس: التقارير وحسابات الصرف.
//! بتشوف نفس مستودع الأوردرات، بس بتجاوب على أسئلة تانية خالص.

use std::sync::Arc;

use async_trait::async_trait;
use kernel::order::OrderRepo;
use service_runtime::{Service, ServiceHealth};

mod revenue;
mod spend;

pub use revenue::RevenueReport;

pub struct BillingService {
    pub(crate) orders: Arc<dyn OrderRepo>,
}

impl BillingService {
    pub fn new(orders: Arc<dyn OrderRepo>) -> Self {
        Self { orders }
    }
}

#[async_trait]
impl Service for BillingService {
    fn name(&self) -> &'static str {
        "billing"
    }

    fn description(&self) -> &'static str {
        "تقارير الإيرادات والصرف"
    }

    async fn health(&self) -> ServiceHealth {
        match self.orders.list(kernel::Page::new(0, 1)).await {
            Ok(_) => ServiceHealth::up(self.name(), "reachable"),
            Err(err) => ServiceHealth::down(self.name(), &err),
        }
    }
}
