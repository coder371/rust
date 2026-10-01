use crate::topology;
use futures::StreamExt;
use lapin::options::{BasicAckOptions, BasicConsumeOptions, BasicNackOptions, BasicQosOptions};
use lapin::types::FieldTable;
use qumra_api::Modules;
use qumra_contracts::{Envelope, orders::OrderCreatedV1};
use qumra_notifications::infrastructure::handlers;
use qumra_platform::AppState;

/// حلقة الاستهلاك: تفكّ الغلاف، تنادي معالج الموديول، ثم تؤكّد.
/// الرسالة الفاشلة تُرفض بلا إعادة صفّ فتذهب للـ DLQ.
pub async fn run(state: AppState, modules: Modules) -> anyhow::Result<()> {
    let ch = topology::declare(&state.bus).await?;
    ch.basic_qos(16, BasicQosOptions::default()).await?;

    let mut consumer = ch
        .basic_consume(
            topology::Q_NOTIF_ORDER_CREATED.into(),
            "qumra-worker".into(),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    tracing::info!("العامل يستمع للأحداث");

    while let Some(next) = consumer.next().await {
        let delivery = match next {
            Ok(d) => d,
            Err(e) => {
                tracing::error!(error = %e, "خطأ في الاستهلاك");
                continue;
            }
        };

        let result = match serde_json::from_slice::<Envelope<OrderCreatedV1>>(&delivery.data) {
            Ok(ev) => handlers::on_order_created(&modules.notifications, ev).await,
            Err(e) => Err(anyhow::anyhow!("رسالة غير صالحة: {e}")),
        };

        match result {
            Ok(()) => {
                delivery.acker.ack(BasicAckOptions::default()).await?;
            }
            Err(e) => {
                tracing::error!(error = %e, "فشلت معالجة الحدث — إلى الـ DLQ");
                delivery
                    .acker
                    .nack(BasicNackOptions { requeue: false, ..Default::default() })
                    .await?;
            }
        }
    }

    Ok(())
}
