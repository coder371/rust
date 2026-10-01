use qumra_kernel::{ProductId, ReservationId, StoreId};

#[derive(Debug, thiserror::Error)]
pub enum PortError {
    #[error("المخزون غير كافٍ للصنف {0}")]
    OutOfStock(String),
    #[error("تعذّر الوصول للخدمة: {0}")]
    Unavailable(String),
}

#[derive(Debug, Clone)]
pub struct ReserveItem {
    pub product_id: ProductId,
    pub quantity: u32,
}

/// ما يحتاجه موديول الطلبات من المخزون — دون أن يعرف أن هناك موديول
/// اسمه inventory أصلاً، ولا أين يعيش.
///
/// هذا الـ trait هو نقطة الاستخراج المستقبلية: تنفيذه اليوم نداء داخل
/// العملية، وغداً نداء HTTP، والكود هنا لا يتغيّر.
#[async_trait::async_trait]
pub trait InventoryPort: Send + Sync + 'static {
    async fn reserve(
        &self,
        store: &StoreId,
        items: &[ReserveItem],
    ) -> Result<ReservationId, PortError>;

    async fn release(&self, store: &StoreId, id: &ReservationId) -> Result<(), PortError>;
}
