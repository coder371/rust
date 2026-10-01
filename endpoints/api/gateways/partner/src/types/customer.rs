use async_graphql::{ID, SimpleObject};

/// الشريك بيشوف بيانات التواصل بس — مفيش راتب ولا ملاحظات.
#[derive(SimpleObject)]
#[graphql(name = "Customer")]
pub struct Customer {
    pub id: ID,
    pub name: String,
    pub email: String,
}

impl From<kernel::user::User> for Customer {
    fn from(u: kernel::user::User) -> Self {
        Self {
            id: ID(u.id.to_string()),
            name: u.name,
            email: u.email,
        }
    }
}
