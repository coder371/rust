use std::sync::RwLock;

use async_trait::async_trait;
use kernel::{DomainError, Page, UserId, user::{User, UserRepo}};

#[derive(Default)]
pub struct InMemoryUserRepo {
    users: RwLock<Vec<User>>,
}

impl InMemoryUserRepo {
    pub fn new() -> Self {
        Self::default()
    }

    /// ريبو مليان بيانات تجربة.
    pub fn seeded() -> Self {
        Self {
            users: RwLock::new(super::seed::users()),
        }
    }

    fn read(&self) -> Result<std::sync::RwLockReadGuard<'_, Vec<User>>, DomainError> {
        self.users
            .read()
            .map_err(|_| DomainError::Internal("user store poisoned".into()))
    }

    fn write(&self) -> Result<std::sync::RwLockWriteGuard<'_, Vec<User>>, DomainError> {
        self.users
            .write()
            .map_err(|_| DomainError::Internal("user store poisoned".into()))
    }
}

#[async_trait]
impl UserRepo for InMemoryUserRepo {
    async fn find(&self, id: &UserId) -> Result<Option<User>, DomainError> {
        Ok(self.read()?.iter().find(|u| &u.id == id).cloned())
    }

    async fn list(&self, page: Page) -> Result<Vec<User>, DomainError> {
        Ok(self
            .read()?
            .iter()
            .skip(page.offset as usize)
            .take(page.limit as usize)
            .cloned()
            .collect())
    }

    async fn insert(&self, user: User) -> Result<User, DomainError> {
        let mut store = self.write()?;
        if store.iter().any(|u| u.email == user.email) {
            return Err(DomainError::Conflict(format!(
                "email already used: {}",
                user.email
            )));
        }
        store.push(user.clone());
        Ok(user)
    }

    async fn set_banned(&self, id: &UserId, banned: bool) -> Result<User, DomainError> {
        let mut store = self.write()?;
        let user = store
            .iter_mut()
            .find(|u| &u.id == id)
            .ok_or_else(|| DomainError::NotFound(format!("user {id}")))?;
        user.banned = banned;
        Ok(user.clone())
    }

    async fn count(&self) -> Result<u64, DomainError> {
        Ok(self.read()?.len() as u64)
    }
}
