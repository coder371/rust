use kernel::{Actor, DomainResult, Page, order::Order};
use service_runtime::ServiceHealth;
use svc_billing::RevenueReport;

/// لوحة الإدارة — 3 خدمات + فحص صحة البلاتفورم كله في ريكوست واحد.
#[derive(Debug, Clone)]
pub struct AdminDashboard {
    pub users_count: u64,
    pub revenue: RevenueReport,
    pub recent_orders: Vec<Order>,
    pub services: Vec<ServiceHealth>,
}

impl crate::ServiceHub {
    /// 🔀 `identity` + `billing` + `orders` على التوازي، وبعدين فحص كل الخدمات المسجّلة.
    pub async fn admin_dashboard(&self, actor: &Actor) -> DomainResult<AdminDashboard> {
        actor.require_admin()?;

        let (users_count, revenue, recent_orders) = futures::try_join!(
            self.identity().count(),
            self.billing().revenue(actor),
            self.orders().recent(actor, Page::new(0, 5)),
        )?;

        Ok(AdminDashboard {
            users_count,
            revenue,
            recent_orders,
            services: self.registry().health().await,
        })
    }
}
