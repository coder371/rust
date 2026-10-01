use super::model::{Reservation, StockItem};
use qumra_kernel::{ProductId, ReservationId, StoreId};

#[derive(Debug, thiserror::Error)]
pub enum StockError {
    #[error("المخزون غير كافٍ للصنف {0}")]
    Insufficient(String),
    #[error("الحجز غير موجود")]
    ReservationNotFound,
    #[error("خطأ تخزين: {0}")]
    Storage(String),
}

#[async_trait::async_trait]
pub trait StockRepository: Send + Sync + 'static {
    async fn adjust(
        &self,
        store: &StoreId,
        product: &ProductId,
        delta: i64,
    ) -> Result<StockItem, StockError>;

    async fn find(
        &self,
        store: &StoreId,
        product: &ProductId,
    ) -> Result<Option<StockItem>, StockError>;

    /// حجز ذرّي لكل صنف: يفشل ولا يترك أثراً جزئياً.
    async fn reserve(
        &self,
        store: &StoreId,
        items: &[(ProductId, u32)],
    ) -> Result<Reservation, StockError>;

    async fn release(&self, store: &StoreId, id: &ReservationId) -> Result<(), StockError>;
}
