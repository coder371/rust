use async_graphql::MergedObject;

mod ban_user;
mod create_user;

#[derive(MergedObject, Default)]
#[graphql(name = "Mutation")]
pub struct AdminMutation(create_user::CreateUserMutation, ban_user::BanUserMutation);
