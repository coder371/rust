use kernel::{
    Actor, DomainError, DomainResult, OrderId, Page, UserId,
    order::Order,
};

impl super::OrdersService {
    /// أوردرات الشريك الحالي بس — الفلترة بالـ tenant جوّه الخدمة مش في البوابة.
    pub async fn for_tenant(&self, actor: &Actor, page: Page) -> DomainResult<Vec<Order>> {
        let tenant = actor.require_tenant()?;
        self.orders.list_by_tenant(tenant, page).await
    }

    /// أوردرات عميل واحد. بتستعملها الفلوهات اللي بتجمّع أكتر من خدمة.
    pub async fn for_user(&self, user_id: &UserId, page: Page) -> DomainResult<Vec<Order>> {
        self.orders.list_by_user(user_id, page).await
    }

    /// آخر الأوردرات — للإدارة بس.
    pub async fn recent(&self, actor: &Actor, page: Page) -> DomainResult<Vec<Order>> {
        actor.require_admin()?;
        self.orders.list(page).await
    }

    /// أوردر واحد بشرط إنه تابع للشريك اللي بيسأل.
    /// المش‑تابع بيرجع NotFound مش Forbidden — مش بنسرّب إنه موجود.
    pub async fn find_for_tenant(&self, actor: &Actor, id: &OrderId) -> DomainResult<Order> {
        let tenant = actor.require_tenant()?;

        let order = self
            .orders
            .find(id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("order {id}")))?;

        if order.tenant.as_deref() != Some(tenant) {
            return Err(DomainError::NotFound(format!("order {id}")));
        }

        Ok(order)
    }
}
