use kernel::{Actor, DomainError, DomainResult, UserId, user::User};

impl super::IdentityService {
    pub async fn set_banned(&self, actor: &Actor, id: &UserId, banned: bool) -> DomainResult<User> {
        actor.require_admin()?;

        let user = self
            .users
            .find(id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("user {id}")))?;

        if user.banned == banned {
            return Err(DomainError::Conflict(format!(
                "user {id} is already banned={banned}"
            )));
        }

        self.users.set_banned(id, banned).await
    }
}
