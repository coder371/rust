use async_graphql::{ID, SimpleObject};
use gw_shared::MoneyView;

/// شكل المستخدم في بوابة الإدارة — كل الحقول ظاهرة.
#[derive(SimpleObject)]
#[graphql(name = "User")]
pub struct User {
    pub id: ID,
    pub name: String,
    pub email: String,
    pub salary: MoneyView,
    pub internal_notes: String,
    pub banned: bool,
}

impl From<kernel::user::User> for User {
    fn from(u: kernel::user::User) -> Self {
        Self {
            id: ID(u.id.to_string()),
            name: u.name,
            email: u.email,
            salary: u.salary.into(),
            internal_notes: u.internal_notes,
            banned: u.banned,
        }
    }
}
