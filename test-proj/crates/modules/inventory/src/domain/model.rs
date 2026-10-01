use qumra_kernel::{ProductId, ReservationId, StoreId};

#[derive(Debug, Clone)]
pub struct StockItem {
    pub store_id: StoreId,
    pub product_id: ProductId,
    pub on_hand: i64,
    pub reserved: i64,
}

impl StockItem {
    /// المتاح للبيع = الموجود ناقص المحجوز. ثابت بسيط لكنه مركز المنطق كله.
    pub fn available(&self) -> i64 {
        self.on_hand - self.reserved
    }
}

#[derive(Debug, Clone)]
pub struct Reservation {
    pub id: ReservationId,
    pub store_id: StoreId,
    pub items: Vec<(ProductId, u32)>,
    pub released: bool,
}
