use async_trait::async_trait;
use qumra_inventory::InventoryModule;
use qumra_kernel::{ProductId, ReservationId, StoreId};
use qumra_orders::application::{InventoryPort, PortError, ReserveItem};

/// الوصلة الوحيدة بين موديول الطلبات وموديول المخزون.
///
/// اليوم: نداء دالة داخل نفس العملية.
/// غداً بعد الاستخراج: `HttpInventoryAdapter` ينفّذ نفس الـ trait فوق reqwest.
/// موديول الطلبات لا يتغيّر فيه سطر واحد في الحالتين.
pub struct InProcessInventoryAdapter {
    inventory: InventoryModule,
}

impl InProcessInventoryAdapter {
    pub fn new(inventory: InventoryModule) -> Self {
        Self { inventory }
    }
}

#[async_trait]
impl InventoryPort for InProcessInventoryAdapter {
    async fn reserve(
        &self,
        store: &StoreId,
        items: &[ReserveItem],
    ) -> Result<ReservationId, PortError> {
        let pairs: Vec<(ProductId, u32)> = items
            .iter()
            .map(|i| (i.product_id.clone(), i.quantity))
            .collect();

        // ترجمة أخطاء المخزون إلى لغة الـ port — الطلبات لا ترى StockError
        self.inventory
            .ops
            .reserve(store, &pairs)
            .await
            .map(|r| r.id)
            .map_err(|e| match e {
                qumra_inventory::domain::StockError::Insufficient(p) => PortError::OutOfStock(p),
                other => PortError::Unavailable(other.to_string()),
            })
    }

    async fn release(&self, store: &StoreId, id: &ReservationId) -> Result<(), PortError> {
        self.inventory
            .ops
            .release(store, id)
            .await
            .map_err(|e| PortError::Unavailable(e.to_string()))
    }
}
