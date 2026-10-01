use std::sync::Arc;

use crate::{Service, ServiceHealth, ServiceStatus};

/// بيمسك الخدمات المسجّلة كلها ويكلّمهم مع بعض.
///
/// ده الجزء اللي بيخلّي "أكتر من خدمة في نفس الوقت" حاجة حقيقية:
/// [`ServiceRegistry::health`] بيشغّل كل الفحوصات على التوازي مش بالدور.
#[derive(Default, Clone)]
pub struct ServiceRegistry {
    services: Vec<Arc<dyn Service>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, service: Arc<dyn Service>) -> &mut Self {
        self.services.push(service);
        self
    }

    pub fn len(&self) -> usize {
        self.services.len()
    }

    pub fn is_empty(&self) -> bool {
        self.services.is_empty()
    }

    /// أسماء الخدمات الشغّالة دلوقتي — بيتطبعوا عند التشغيل.
    pub fn names(&self) -> Vec<&'static str> {
        self.services.iter().map(|s| s.name()).collect()
    }

    /// فحص كل الخدمات **في نفس اللحظة**.
    pub async fn health(&self) -> Vec<ServiceHealth> {
        futures::future::join_all(self.services.iter().map(|s| s.health())).await
    }

    /// أسوأ حالة موجودة — لأن البلاتفورم بيبقى بحال أضعف خدمة فيه.
    pub fn worst(report: &[ServiceHealth]) -> ServiceStatus {
        report
            .iter()
            .map(|h| h.status)
            .max()
            .unwrap_or(ServiceStatus::Up)
    }
}
