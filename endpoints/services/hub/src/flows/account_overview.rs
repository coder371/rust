use kernel::{Actor, DomainResult, Money, Page, UserId, order::Order, user::User};

/// صورة كاملة لعميل واحد — مجمّعة من 3 خدمات.
#[derive(Debug, Clone)]
pub struct AccountOverview {
    pub user: User,
    pub orders: Vec<Order>,
    pub total_spent: Money,
}

impl crate::ServiceHub {
    /// 🔀 بيلمس `identity` + `orders` + `billing` **في نفس اللحظة**.
    ///
    /// التلاتة ماعندهمش اعتماد على بعض، فمفيش أي سبب نستناهم بالدور:
    /// الوقت الكلي = أبطأ خدمة فيهم، مش مجموعهم.
    pub async fn account_overview(
        &self,
        actor: &Actor,
        id: &UserId,
    ) -> DomainResult<AccountOverview> {
        actor.require_admin()?;

        let (user, orders, total_spent) = futures::try_join!(
            self.identity().by_id(actor, id),
            self.orders().for_user(id, Page::new(0, Page::MAX_LIMIT)),
            self.billing().user_spend(id),
        )?;

        Ok(AccountOverview {
            user,
            orders,
            total_spent,
        })
    }
}
