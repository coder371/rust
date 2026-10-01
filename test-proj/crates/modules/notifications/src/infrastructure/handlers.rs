use crate::NotificationsModule;
use crate::application::OrderSummary;
// النوع يأتي من العقود المشتركة، لا من crate الطلبات
use qumra_contracts::{Envelope, orders::OrderCreatedV1};
use qumra_kernel::StoreId;

/// معالج حدث `orders.created.v1`.
///
/// التسليم في RabbitMQ «مرة واحدة على الأقل»، فأول شيء نفعله هو حجز
/// معرّف الحدث في Redis. لو كان محجوزاً فالحدث معالَج من قبل ونخرج بسلام.
pub async fn on_order_created(
    m: &NotificationsModule,
    ev: Envelope<OrderCreatedV1>,
) -> anyhow::Result<()> {
    if !m.idempotency.claim(&ev.event_id, 7 * 24 * 3600).await? {
        tracing::debug!(id = %ev.event_id, "حدث مكرّر — تم تجاهله");
        return Ok(());
    }

    let summary = OrderSummary {
        order_id: ev.payload.order_id,
        customer_id: ev.payload.customer_id,
        total_minor: ev.payload.total_minor,
        currency: ev.payload.currency,
    };

    m.send_confirmation
        .exec(&StoreId::new(ev.store_id), &summary)
        .await?;

    Ok(())
}
