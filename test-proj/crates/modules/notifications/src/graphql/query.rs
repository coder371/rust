use super::tenant;
use crate::NotificationsModule;
use async_graphql::{Context, MergedObject, Object, Result, SimpleObject};

#[derive(SimpleObject)]
pub struct NotificationDto {
    pub id: String,
    pub order_id: String,
    pub channel: String,
    pub body: String,
    pub sent_at_ms: i64,
}

#[derive(Default)]
pub struct NotificationsListQuery;

#[Object]
impl NotificationsListQuery {
    async fn notifications(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 20)] limit: i64,
    ) -> Result<Vec<NotificationDto>> {
        let m = ctx.data_unchecked::<NotificationsModule>();
        let list = m.queries.list(tenant(ctx)?, limit).await?;
        Ok(list
            .into_iter()
            .map(|n| NotificationDto {
                id: n.id,
                order_id: n.order_id,
                channel: n.channel,
                body: n.body,
                sent_at_ms: n.sent_at_ms,
            })
            .collect())
    }
}

#[derive(MergedObject, Default)]
pub struct NotificationsQuery(NotificationsListQuery);
