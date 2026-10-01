use super::tenant;
use crate::InventoryModule;
use async_graphql::{Context, MergedObject, Object, Result, SimpleObject};
use qumra_kernel::ProductId;

#[derive(SimpleObject)]
pub struct StockDto {
    pub product_id: String,
    pub on_hand: i64,
    pub reserved: i64,
    pub available: i64,
}

#[derive(Default)]
pub struct StockQuery;

#[Object]
impl StockQuery {
    async fn stock(&self, ctx: &Context<'_>, product_id: String) -> Result<Option<StockDto>> {
        let m = ctx.data_unchecked::<InventoryModule>();
        let item = m.ops.get(tenant(ctx)?, &ProductId::new(product_id)).await?;
        Ok(item.map(|s| StockDto {
            product_id: s.product_id.to_string(),
            on_hand: s.on_hand,
            reserved: s.reserved,
            available: s.available(),
        }))
    }
}

#[derive(Default)]
pub struct AdjustStockMutation;

#[Object]
impl AdjustStockMutation {
    /// إدخال أو تعديل رصيد صنف (موجب للإضافة، سالب للخصم)
    async fn adjust_stock(
        &self,
        ctx: &Context<'_>,
        product_id: String,
        delta: i64,
    ) -> Result<StockDto> {
        let m = ctx.data_unchecked::<InventoryModule>();
        let s = m
            .ops
            .adjust(tenant(ctx)?, &ProductId::new(product_id), delta)
            .await?;
        Ok(StockDto {
            product_id: s.product_id.to_string(),
            on_hand: s.on_hand,
            reserved: s.reserved,
            available: s.available(),
        })
    }
}

#[derive(MergedObject, Default)]
pub struct InventoryQuery(StockQuery);

#[derive(MergedObject, Default)]
pub struct InventoryMutation(AdjustStockMutation);
