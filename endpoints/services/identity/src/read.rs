use kernel::{Actor, DomainError, DomainResult, Page, UserId, user::User};

impl super::IdentityService {
    /// قراءة موثوقة — البوابة اللي بتناديها هي اللي ضامنة إن المنفّذ مسموح له.
    pub async fn by_id(&self, _actor: &Actor, id: &UserId) -> DomainResult<User> {
        self.users
            .find(id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("user {id}")))
    }

    pub async fn list(&self, _actor: &Actor, page: Page) -> DomainResult<Vec<User>> {
        self.users.list(page).await
    }

    /// الإدارة بتشوف كل المستخدمين — بما فيهم المحظورين.
    pub async fn list_all(&self, actor: &Actor, page: Page) -> DomainResult<Vec<User>> {
        actor.require_admin()?;
        self.users.list(page).await
    }

    pub async fn count(&self) -> DomainResult<u64> {
        self.users.count().await
    }
}
