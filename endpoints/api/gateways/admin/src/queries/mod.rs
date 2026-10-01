use async_graphql::MergedObject;

mod dashboard;
mod list_users;
mod revenue;
mod user_by_id;

/// لما العدد يكبر، بنجمّع بالـ aggregate بدل تيوبل واحد طويل.
#[derive(MergedObject, Default)]
pub struct UserQueries(user_by_id::UserByIdQuery, list_users::ListUsersQuery);

#[derive(MergedObject, Default)]
pub struct ReportQueries(revenue::RevenueQuery, dashboard::DashboardQuery);

#[derive(MergedObject, Default)]
#[graphql(name = "Query")]
pub struct AdminQuery(UserQueries, ReportQueries);
