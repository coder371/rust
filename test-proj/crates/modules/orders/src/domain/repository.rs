use super::model::Order;
use qumra_kernel::{OrderId, OutboxRecord, StoreId};

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("تعارض نسخة: الطلب تغيّر أثناء المعالجة")]
    VersionConflict,
    #[error("خطأ تخزين: {0}")]
    Storage(String),
}

/// عقد التخزين. لا أثر لمونجو هنا — ولا يجب أن يظهر.
///
/// لاحظ أن `insert` و`save` تأخذان الأحداث مع الكيان: هذا التوقيع
/// يجبر أي تنفيذ على كتابتهما معاً، فلا يمكن حفظ طلب بلا حدثه.
#[async_trait::async_trait]
pub trait OrderRepository: Send + Sync + 'static {
    async fn insert(&self, order: &Order, events: &[OutboxRecord]) -> Result<(), RepoError>;

    /// كتابة متفائلة: تفشل لو تغيّرت النسخة تحت اليد.
    async fn save(
        &self,
        order: &Order,
        expected_version: u32,
        events: &[OutboxRecord],
    ) -> Result<(), RepoError>;

    /// كل قراءة مقيّدة بالمتجر — العزل شرط في التوقيع نفسه.
    async fn find(&self, store: &StoreId, id: &OrderId) -> Result<Option<Order>, RepoError>;

    async fn list(&self, store: &StoreId, limit: i64) -> Result<Vec<Order>, RepoError>;
}
