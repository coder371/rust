use service_runtime::{ServiceHealth, ServiceRegistry, ServiceStatus};

#[derive(Debug, Clone)]
pub struct PlatformHealth {
    /// حال البلاتفورم = حال أضعف خدمة فيه.
    pub status: ServiceStatus,
    pub services: Vec<ServiceHealth>,
}

impl crate::ServiceHub {
    /// 🔀 بيفحص **كل** الخدمات المسجّلة في نفس اللحظة.
    /// مالهاش actor: ده إند بوينت تشغيلي، مش بيرجّع بيانات.
    pub async fn platform_health(&self) -> PlatformHealth {
        let services = self.registry().health().await;

        PlatformHealth {
            status: ServiceRegistry::worst(&services),
            services,
        }
    }
}
