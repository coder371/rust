//! ❷ البيزنس لوجيك — الهَب: نقطة واحدة بتشغّل كل الخدمات جنب بعض.
//!
//! كل خدمة (`svc-identity` / `svc-orders` / `svc-billing`) مستقلة وماتعرفش أختها.
//! اللي عايز يجمّع أكتر من واحدة بيعمل ده هنا في `flows/` — يوزكيس في ملف،
//! وبينادي الخدمات **على التوازي** مش واحدة ورا التانية.
//!
//! البوابات بتاخد `Arc<ServiceHub>` وخلاص، فأي خدمة جديدة بتبقى متاحة
//! لكل البوابات من غير ما نلمس ولا بوابة.

use std::sync::Arc;

use kernel::{order::OrderRepo, user::UserRepo};
use svc_billing::BillingService;
use svc_identity::IdentityService;
use svc_orders::OrdersService;

mod flows;

pub use flows::{AccountOverview, AdminDashboard, PlatformHealth};
pub use service_runtime::{Service, ServiceHealth, ServiceRegistry, ServiceStatus};
pub use svc_identity::AuthService;
pub use svc_billing::RevenueReport;

pub struct ServiceHub {
    auth: Arc<AuthService>,
    identity: Arc<IdentityService>,
    orders: Arc<OrdersService>,
    billing: Arc<BillingService>,
    registry: ServiceRegistry,
}

impl ServiceHub {
    /// التركيب بيحصل مرة واحدة عند التشغيل: المستودعات داخلة، والخدمات بتتسجّل.
    ///
    /// عايز تضيف خدمة؟ كريت `svc-*` جديد + سطرين هنا. البوابات ماتتلمسش.
    pub fn new(users: Arc<dyn UserRepo>, orders: Arc<dyn OrderRepo>, auth: AuthService) -> Self {
        let identity = Arc::new(IdentityService::new(users));
        let orders_svc = Arc::new(OrdersService::new(orders.clone()));
        let billing = Arc::new(BillingService::new(orders));

        let mut registry = ServiceRegistry::new();
        registry
            .register(identity.clone())
            .register(orders_svc.clone())
            .register(billing.clone());

        Self {
            auth: Arc::new(auth),
            identity,
            orders: orders_svc,
            billing,
            registry,
        }
    }

    pub fn auth(&self) -> &AuthService {
        &self.auth
    }

    pub fn identity(&self) -> &IdentityService {
        &self.identity
    }

    pub fn orders(&self) -> &OrdersService {
        &self.orders
    }

    pub fn billing(&self) -> &BillingService {
        &self.billing
    }

    pub fn registry(&self) -> &ServiceRegistry {
        &self.registry
    }
}
