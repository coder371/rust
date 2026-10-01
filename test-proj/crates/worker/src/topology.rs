use lapin::{
    Channel, ExchangeKind,
    options::{ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions},
    types::{AMQPValue, FieldTable},
};
use qumra_platform::{EventBus, bus::EXCHANGE};

pub const DLX: &str = "qumra.dlx";
/// طابور لكل (مستهلك × حدث): فشل الإشعارات لا يعطّل التحليلات
pub const Q_NOTIF_ORDER_CREATED: &str = "notif.orders.created";

pub async fn declare(bus: &EventBus) -> anyhow::Result<Channel> {
    let ch = bus.channel().await?;

    ch.exchange_declare(
        DLX.into(),
        ExchangeKind::Topic,
        ExchangeDeclareOptions { durable: true, ..Default::default() },
        FieldTable::default(),
    )
    .await?;

    // الرسالة المرفوضة تذهب للـ DLX بدل أن تدور إلى الأبد
    let mut args = FieldTable::default();
    args.insert("x-dead-letter-exchange".into(), AMQPValue::LongString(DLX.into()));

    ch.queue_declare(
        Q_NOTIF_ORDER_CREATED.into(),
        QueueDeclareOptions { durable: true, ..Default::default() },
        args,
    )
    .await?;

    ch.queue_bind(
        Q_NOTIF_ORDER_CREATED.into(),
        EXCHANGE.into(),
        qumra_contracts::orders::ORDER_CREATED.into(),
        QueueBindOptions::default(),
        FieldTable::default(),
    )
    .await?;

    // طابور الرسائل الميتة، للفحص اليدوي
    ch.queue_declare(
        "qumra.dlq".into(),
        QueueDeclareOptions { durable: true, ..Default::default() },
        FieldTable::default(),
    )
    .await?;
    ch.queue_bind(
        "qumra.dlq".into(),
        DLX.into(),
        "#".into(),
        QueueBindOptions::default(),
        FieldTable::default(),
    )
    .await?;

    tracing::info!(queue = Q_NOTIF_ORDER_CREATED, "طوبولوجيا الأحداث جاهزة");
    Ok(ch)
}
