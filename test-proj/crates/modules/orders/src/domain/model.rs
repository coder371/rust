use super::error::OrderError;
use qumra_kernel::{CustomerId, Money, OrderId, ProductId, StoreId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Paid,
    Cancelled,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderStatus::Pending => "PENDING",
            OrderStatus::Paid => "PAID",
            OrderStatus::Cancelled => "CANCELLED",
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrderLine {
    pub product_id: ProductId,
    pub quantity: u32,
    pub unit_price: Money,
}

impl OrderLine {
    pub fn subtotal(&self) -> Result<Money, OrderError> {
        Ok(self.unit_price.mul(self.quantity)?)
    }
}

/// الكيان الجذر. لا يمكن بناؤه إلا عبر `place`، فلا وجود لطلب غير صالح
/// في أي مكان في النظام — بما في ذلك ما يُقرأ من قاعدة البيانات.
#[derive(Debug, Clone)]
pub struct Order {
    pub id: OrderId,
    pub store_id: StoreId,
    pub customer_id: CustomerId,
    pub lines: Vec<OrderLine>,
    pub total: Money,
    pub status: OrderStatus,
    pub placed_at_ms: i64,
    pub version: u32,
}

impl Order {
    pub fn place(
        store_id: StoreId,
        customer_id: CustomerId,
        lines: Vec<OrderLine>,
        now_ms: i64,
    ) -> Result<Self, OrderError> {
        let first = lines.first().ok_or(OrderError::EmptyOrder)?;
        let currency = first.unit_price.currency;

        for l in &lines {
            if l.quantity == 0 {
                return Err(OrderError::InvalidQuantity(l.product_id.to_string()));
            }
            if l.unit_price.currency != currency {
                return Err(OrderError::MixedCurrency);
            }
        }

        let mut total = Money::zero(currency);
        for l in &lines {
            total = total.add(l.subtotal()?)?;
        }

        Ok(Self {
            id: OrderId::generate(),
            store_id,
            customer_id,
            lines,
            total,
            status: OrderStatus::Pending,
            placed_at_ms: now_ms,
            version: 1,
        })
    }

    /// إعادة بناء من التخزين. لا تتحقق من الثوابت لأن الكيان
    /// مرّ بها وقت الإنشاء — لكنها الطريق الوحيد الآخر لبناء Order.
    pub fn rehydrate(
        id: OrderId,
        store_id: StoreId,
        customer_id: CustomerId,
        lines: Vec<OrderLine>,
        total: Money,
        status: OrderStatus,
        placed_at_ms: i64,
        version: u32,
    ) -> Self {
        Self { id, store_id, customer_id, lines, total, status, placed_at_ms, version }
    }

    /// آلة الحالات: الانتقال الوحيد المسموح به نحو Paid.
    pub fn mark_paid(&mut self) -> Result<(), OrderError> {
        match self.status {
            OrderStatus::Pending => {
                self.status = OrderStatus::Paid;
                self.version += 1;
                Ok(())
            }
            OrderStatus::Cancelled => Err(OrderError::AlreadyCancelled),
            other => Err(OrderError::InvalidTransition { from: other }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qumra_kernel::Currency;

    fn line(qty: u32, minor: i64, cur: Currency) -> OrderLine {
        OrderLine {
            product_id: ProductId::new("p1"),
            quantity: qty,
            unit_price: Money::new(minor, cur),
        }
    }

    #[test]
    fn يرفض_الطلب_الفارغ() {
        let r = Order::place(StoreId::new("s1"), CustomerId::new("c1"), vec![], 0);
        assert!(matches!(r, Err(OrderError::EmptyOrder)));
    }

    #[test]
    fn يرفض_خلط_العملات() {
        let lines = vec![line(1, 100, Currency::Sar), line(1, 100, Currency::Aed)];
        let r = Order::place(StoreId::new("s1"), CustomerId::new("c1"), lines, 0);
        assert!(matches!(r, Err(OrderError::MixedCurrency)));
    }

    #[test]
    fn يحسب_الإجمالي_بالوحدة_الصغرى() {
        let lines = vec![line(3, 1500, Currency::Sar), line(2, 500, Currency::Sar)];
        let o = Order::place(StoreId::new("s1"), CustomerId::new("c1"), lines, 0).unwrap();
        assert_eq!(o.total.minor, 3 * 1500 + 2 * 500);
    }

    #[test]
    fn لا_يمكن_دفع_طلب_ملغي() {
        let lines = vec![line(1, 100, Currency::Sar)];
        let mut o = Order::place(StoreId::new("s1"), CustomerId::new("c1"), lines, 0).unwrap();
        o.status = OrderStatus::Cancelled;
        assert!(matches!(o.mark_paid(), Err(OrderError::AlreadyCancelled)));
    }
}
