use super::model::Notification;
use qumra_kernel::StoreId;

#[derive(Debug, thiserror::Error)]
pub enum NotifyError {
    #[error("تعذّر تأليف الرسالة: {0}")]
    Compose(String),
    #[error("خطأ تخزين: {0}")]
    Storage(String),
}

#[async_trait::async_trait]
pub trait NotificationRepository: Send + Sync + 'static {
    async fn record(&self, n: &Notification) -> Result<(), NotifyError>;
    async fn list(&self, store: &StoreId, limit: i64) -> Result<Vec<Notification>, NotifyError>;
}
