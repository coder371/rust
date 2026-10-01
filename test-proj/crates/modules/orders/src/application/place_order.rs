use super::ports::{InventoryPort, ReserveItem};
use crate::domain::{Order, OrderError, OrderLine, OrderRepository};
use qumra_contracts::orders::{ORDER_CREATED, OrderCreatedV1};
use qumra_kernel::{Clock, CustomerId, Money, OutboxRecord, Permission, ProductId, TenantContext};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct PlaceOrderLine {
    pub product_id: ProductId,
    pub quantity: u32,
    pub unit_price: Money,
}

#[derive(Debug, Clone)]
pub struct PlaceOrderCommand {
    pub customer_id: CustomerId,
    pub lines: Vec<PlaceOrderLine>,
}

/// حالة استخدام واحدة في ملف واحد.
/// تنسّق بين الـ domain والـ ports، ولا تعرف مونجو ولا RabbitMQ.
pub struct PlaceOrder {
    orders: Arc<dyn OrderRepository>,
    inventory: Arc<dyn InventoryPort>,
    clock: Arc<dyn Clock>,
}

impl PlaceOrder {
    pub fn new(
        orders: Arc<dyn OrderRepository>,
        inventory: Arc<dyn InventoryPort>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self { orders, inventory, clock }
    }

    pub async fn exec(
        &self,
        ctx: &TenantContext,
        cmd: PlaceOrderCommand,
    ) -> Result<Order, ApplicationError> {
        ctx.require(Permission::OrdersWrite)?;

        // ١) حجز المخزون عبر port — لا نعرف من ينفّذه
        let items: Vec<ReserveItem> = cmd
            .lines
            .iter()
            .map(|l| ReserveItem { product_id: l.product_id.clone(), quantity: l.quantity })
            .collect();
        let reservation = self.inventory.reserve(&ctx.store_id, &items).await?;

        // ٢) بناء الكيان — الثوابت تُفرض هنا وليس في الـ resolver
        let lines = cmd
            .lines
            .into_iter()
            .map(|l| OrderLine {
                product_id: l.product_id,
                quantity: l.quantity,
                unit_price: l.unit_price,
            })
            .collect();

        let order = match Order::place(
            ctx.store_id.clone(),
            cmd.customer_id,
            lines,
            self.clock.now_ms(),
        ) {
            Ok(o) => o,
            Err(e) => {
                // تعويض: نحرّر الحجز قبل الخروج
                let _ = self.inventory.release(&ctx.store_id, &reservation).await;
                return Err(e.into());
            }
        };

        // ٣) الحدث يُبنى هنا: الـ application هي التي تعرف عقود contracts
        let event = OutboxRecord::new(
            ORDER_CREATED,
            ctx.store_id.as_str(),
            order.placed_at_ms,
            &OrderCreatedV1 {
                order_id: order.id.to_string(),
                customer_id: order.customer_id.to_string(),
                total_minor: order.total.minor,
                currency: order.total.currency.code().to_string(),
                line_count: order.lines.len(),
            },
        )
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        // ٤) الطلب وحدثه في وحدة ذرّية واحدة
        if let Err(e) = self.orders.insert(&order, &[event]).await {
            let _ = self.inventory.release(&ctx.store_id, &reservation).await;
            return Err(e.into());
        }

        tracing::info!(order = %order.id, total = order.total.minor, "طلب جديد");
        Ok(order)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error(transparent)]
    Order(#[from] OrderError),
    #[error(transparent)]
    Port(#[from] super::ports::PortError),
    #[error(transparent)]
    Repo(#[from] crate::domain::RepoError),
    #[error(transparent)]
    Auth(#[from] qumra_kernel::AuthError),
    #[error("خطأ داخلي: {0}")]
    Internal(String),
}
