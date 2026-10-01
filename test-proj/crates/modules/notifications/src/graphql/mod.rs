pub mod query;
pub use query::NotificationsQuery;

use async_graphql::Context;
use qumra_kernel::TenantContext;

pub(crate) fn tenant<'a>(ctx: &'a Context<'_>) -> async_graphql::Result<&'a TenantContext> {
    ctx.data::<TenantContext>()
        .map_err(|_| async_graphql::Error::new("مطلوب رأس x-store-id"))
}
