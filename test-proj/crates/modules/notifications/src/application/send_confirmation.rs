use super::ports::MessageComposer;
use crate::domain::{Notification, NotificationRepository, NotifyError};
use qumra_kernel::{Clock, StoreId};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct OrderSummary {
    pub order_id: String,
    pub customer_id: String,
    pub total_minor: i64,
    pub currency: String,
}

pub struct SendOrderConfirmation {
    repo: Arc<dyn NotificationRepository>,
    composer: Arc<dyn MessageComposer>,
    clock: Arc<dyn Clock>,
}

impl SendOrderConfirmation {
    pub fn new(
        repo: Arc<dyn NotificationRepository>,
        composer: Arc<dyn MessageComposer>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self { repo, composer, clock }
    }

    pub async fn exec(
        &self,
        store: &StoreId,
        order: &OrderSummary,
    ) -> Result<Notification, NotifyError> {
        let body = self.composer.order_confirmation(order).await?;

        let n = Notification {
            id: format!("ntf-{}", order.order_id),
            store_id: store.clone(),
            order_id: order.order_id.clone(),
            channel: "sms".to_string(),
            body,
            sent_at_ms: self.clock.now_ms(),
        };

        self.repo.record(&n).await?;
        tracing::info!(order = %n.order_id, "أُرسل إشعار");
        Ok(n)
    }
}
