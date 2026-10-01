use async_graphql::{Context, Guard, Result};
use kernel::Actor;

/// حماية على مستوى الحقل — طبقة تانية بعد الميدلوير.
pub struct AdminGuard;

impl Guard for AdminGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        let actor = ctx.data::<Actor>()?;
        if actor.is_admin() {
            Ok(())
        } else {
            Err("admin role required".into())
        }
    }
}
