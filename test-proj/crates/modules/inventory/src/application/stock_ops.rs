use crate::domain::{Reservation, StockError, StockItem, StockRepository};
use qumra_kernel::{Permission, ProductId, ReservationId, StoreId, TenantContext};
use std::sync::Arc;

pub struct StockOps {
    repo: Arc<dyn StockRepository>,
}

impl StockOps {
    pub fn new(repo: Arc<dyn StockRepository>) -> Self {
        Self { repo }
    }

    pub async fn adjust(
        &self,
        ctx: &TenantContext,
        product: &ProductId,
        delta: i64,
    ) -> Result<StockItem, StockError> {
        ctx.require(Permission::InventoryWrite)
            .map_err(|e| StockError::Storage(e.to_string()))?;
        self.repo.adjust(&ctx.store_id, product, delta).await
    }

    pub async fn get(
        &self,
        ctx: &TenantContext,
        product: &ProductId,
    ) -> Result<Option<StockItem>, StockError> {
        ctx.require(Permission::InventoryRead)
            .map_err(|e| StockError::Storage(e.to_string()))?;
        self.repo.find(&ctx.store_id, product).await
    }

    /// تُستدعى من الأدابتر نيابةً عن موديول الطلبات — بلا سياق GraphQL.
    pub async fn reserve(
        &self,
        store: &StoreId,
        items: &[(ProductId, u32)],
    ) -> Result<Reservation, StockError> {
        self.repo.reserve(store, items).await
    }

    pub async fn release(
        &self,
        store: &StoreId,
        id: &ReservationId,
    ) -> Result<(), StockError> {
        self.repo.release(store, id).await
    }
}
