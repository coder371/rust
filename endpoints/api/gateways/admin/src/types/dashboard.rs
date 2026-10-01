use async_graphql::SimpleObject;

use super::{Order, RevenueReport, ServiceHealth};

/// نتيجة فلو `admin_dashboard`: 3 خدمات + فحص البلاتفورم في ريكوست واحد.
#[derive(SimpleObject)]
#[graphql(name = "Dashboard")]
pub struct Dashboard {
    pub users_count: u64,
    pub revenue: RevenueReport,
    pub recent_orders: Vec<Order>,
    pub services: Vec<ServiceHealth>,
}

impl From<services::AdminDashboard> for Dashboard {
    fn from(d: services::AdminDashboard) -> Self {
        Self {
            users_count: d.users_count,
            revenue: d.revenue.into(),
            recent_orders: d.recent_orders.into_iter().map(Order::from).collect(),
            services: d.services.into_iter().map(ServiceHealth::from).collect(),
        }
    }
}
