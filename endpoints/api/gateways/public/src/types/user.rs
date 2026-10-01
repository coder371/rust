use async_graphql::{ID, SimpleObject};

/// شكل المستخدم في البوابة العامة — الاسم والـ id بس.
/// مفيش راتب ولا ملاحظات داخلية ولا حتى إيميل.
#[derive(SimpleObject)]
#[graphql(name = "User")]
pub struct User {
    pub id: ID,
    pub name: String,
}

impl From<kernel::user::User> for User {
    fn from(u: kernel::user::User) -> Self {
        Self {
            id: ID(u.id.to_string()),
            name: u.name,
        }
    }
}
