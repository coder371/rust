use crate::domain::{Notification, NotificationRepository, NotifyError};
use qumra_kernel::{Permission, TenantContext};
use std::sync::Arc;

pub struct NotificationQueries {
    repo: Arc<dyn NotificationRepository>,
}

impl NotificationQueries {
    pub fn new(repo: Arc<dyn NotificationRepository>) -> Self {
        Self { repo }
    }

    pub async fn list(
        &self,
        ctx: &TenantContext,
        limit: i64,
    ) -> Result<Vec<Notification>, NotifyError> {
        ctx.require(Permission::NotificationsRead)
            .map_err(|e| NotifyError::Storage(e.to_string()))?;
        self.repo.list(&ctx.store_id, limit.clamp(1, 100)).await
    }
}
