use kernel::{
    Actor, DomainError, DomainResult, UserId,
    user::{NewUser, User},
};

impl super::IdentityService {
    pub async fn create(&self, actor: &Actor, input: NewUser) -> DomainResult<User> {
        actor.require_admin()?;

        if input.name.trim().is_empty() {
            return Err(DomainError::Invalid("name is required".into()));
        }
        if !input.email.contains('@') {
            return Err(DomainError::Invalid("email is not valid".into()));
        }

        let user = User {
            id: UserId::new(format!("u{}", rand::random::<u32>())),
            name: input.name,
            email: input.email,
            salary: input.salary,
            internal_notes: String::new(),
            banned: false,
        };

        self.users.insert(user).await
    }
}
