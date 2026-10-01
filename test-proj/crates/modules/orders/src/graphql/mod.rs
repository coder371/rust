pub mod mutation;
pub mod query;
pub mod types;

pub use mutation::OrdersMutation;
pub use query::OrdersQuery;

use async_graphql::Context;
use qumra_kernel::TenantContext;

/// هوية الطلب تأتي من الـ middleware، لا من مدخلات المستخدم.
pub(crate) fn tenant<'a>(ctx: &'a Context<'_>) -> async_graphql::Result<&'a TenantContext> {
    ctx.data::<TenantContext>()
        .map_err(|_| async_graphql::Error::new("مطلوب رأس x-store-id"))
}
