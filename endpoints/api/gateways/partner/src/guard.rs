use async_graphql::{Context, Guard, Result};
use kernel::Actor;

pub struct PartnerGuard;

impl Guard for PartnerGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        let actor = ctx.data::<Actor>()?;
        if actor.is_partner() {
            Ok(())
        } else {
            Err("partner role required".into())
        }
    }
}
