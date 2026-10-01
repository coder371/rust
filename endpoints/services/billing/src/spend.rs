use kernel::{DomainResult, Money, Page, UserId, order::OrderStatus};

impl super::BillingService {
    /// إجمالي اللي دفعه عميل واحد — المدفوع بس.
    /// مالهاش actor لأنها جوّاية: اللي بينادي هو اللي بيتأكد من الصلاحية.
    pub async fn user_spend(&self, user_id: &UserId) -> DomainResult<Money> {
        let orders = self
            .orders
            .list_by_user(user_id, Page::new(0, Page::MAX_LIMIT))
            .await?;

        Ok(orders
            .iter()
            .filter(|o| o.status == OrderStatus::Paid)
            .map(|o| o.total)
            .sum())
    }
}
