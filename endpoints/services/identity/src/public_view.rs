use kernel::{Actor, DomainError, DomainResult, Page, UserId, user::User};

/// القراءات اللي بتخرج للعامة. القاعدة هنا: المحظور مايظهرش أصلًا.
impl super::IdentityService {
    pub async fn browse(&self, _actor: &Actor, page: Page) -> DomainResult<Vec<User>> {
        let users = self.users.list(page).await?;
        Ok(users.into_iter().filter(|u| !u.banned).collect())
    }

    pub async fn public_profile(&self, _actor: &Actor, id: &UserId) -> DomainResult<User> {
        match self.users.find(id).await? {
            // المحظور بيرجع NotFound مش Forbidden — مش بنسرّب إنه موجود
            Some(user) if !user.banned => Ok(user),
            _ => Err(DomainError::NotFound(format!("user {id}"))),
        }
    }
}
