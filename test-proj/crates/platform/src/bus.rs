use lapin::{
    BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind,
    options::{BasicPublishOptions, ExchangeDeclareOptions},
    types::FieldTable,
};

pub const EXCHANGE: &str = "qumra.events";

/// اتصال واحد بـ RabbitMQ + قناة نشر مشتركة، يُبنيان عند الإقلاع.
/// المستهلكون يفتحون قنواتهم الخاصة عبر `channel()`.
pub struct EventBus {
    _conn: Connection,
    publish_ch: Channel,
}

impl EventBus {
    pub async fn connect(uri: &str) -> anyhow::Result<Self> {
        let conn = Connection::connect(uri, ConnectionProperties::default()).await?;
        let ch = conn.create_channel().await?;

        ch.exchange_declare(
            EXCHANGE.into(),
            ExchangeKind::Topic,
            ExchangeDeclareOptions { durable: true, ..Default::default() },
            FieldTable::default(),
        )
        .await?;

        Ok(Self { _conn: conn, publish_ch: ch })
    }

    pub async fn channel(&self) -> anyhow::Result<Channel> {
        Ok(self._conn.create_channel().await?)
    }

    pub async fn publish(&self, routing_key: &str, payload: &[u8]) -> anyhow::Result<()> {
        self.publish_ch
            .basic_publish(
                EXCHANGE.into(),
                routing_key.into(),
                BasicPublishOptions::default(),
                payload,
                BasicProperties::default().with_delivery_mode(2), // رسالة معمّرة
            )
            .await?
            .await?; // انتظار تأكيد الناشر
        Ok(())
    }
}
