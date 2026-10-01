use kernel::{Actor, DomainResult, OrderId, user::User};

impl crate::ServiceHub {
    /// 🔗 `orders` وبعدين `identity` — هنا الترتيب مقصود:
    /// لازم نتأكد الأوردر تابع للشريك الأول، قبل ما نجيب بيانات العميل أصلًا.
    pub async fn order_customer(&self, actor: &Actor, order_id: &OrderId) -> DomainResult<User> {
        let order = self.orders().find_for_tenant(actor, order_id).await?;
        self.identity().by_id(actor, &order.user_id).await
    }
}
