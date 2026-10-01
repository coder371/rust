use async_graphql::MergedObject;

mod browse_users;
mod hello;
mod user_profile;

#[derive(MergedObject, Default)]
pub struct UserQueries(browse_users::BrowseUsersQuery, user_profile::UserProfileQuery);

#[derive(MergedObject, Default)]
#[graphql(name = "Query")]
pub struct PublicQuery(hello::HelloQuery, UserQueries);
