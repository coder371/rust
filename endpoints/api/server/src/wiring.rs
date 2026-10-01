//! ❸ نقطة التركيب: 3 خطوات بالترتيب — كور، خدمات، بوابات.
//!
//! أي بوابة هنا مش شايفة `store` أصلًا، وأي خدمة مش شايفة البوابات.

use std::sync::Arc;

use axum::Router;
use kernel::{order::OrderRepo, user::UserRepo};
use services::ServiceHub;
use store::memory::{InMemoryOrderRepo, InMemoryUserRepo};
use services::AuthService;

use crate::config::Config;

pub fn build(cfg: &Config) -> (Arc<ServiceHub>, Router) {
    // ❶ الكور — بدّل السطرين دول بـ Mongo وخلاص
    let user_repo: Arc<dyn UserRepo> = Arc::new(InMemoryUserRepo::seeded());
    let order_repo: Arc<dyn OrderRepo> = Arc::new(InMemoryOrderRepo::seeded());

    // ❷ البيزنس لوجيك — هَب واحد بيشغّل كل الخدمات جنب بعض
    let hub = Arc::new(ServiceHub::new(
        user_repo,
        order_repo,
        AuthService::new(
            cfg.admin_token.clone(),
            cfg.partner_token.clone(),
            cfg.customer_token.clone(),
        ),
    ));

    // ❸ البوابات — كل واحدة على مسارها، وكلها بتاخد نفس الهَب
    let router = Router::new()
        .nest(
            gw_admin::MOUNT,
            gw_admin::router(gw_admin::AdminCtx { hub: hub.clone() }),
        )
        .nest(
            gw_public::MOUNT,
            gw_public::router(gw_public::PublicCtx { hub: hub.clone() }),
        )
        .nest(
            gw_partner::MOUNT,
            gw_partner::router(gw_partner::PartnerCtx { hub: hub.clone() }),
        )
        .nest(
            gw_rest::MOUNT,
            gw_rest::router(gw_rest::RestCtx { hub: hub.clone() }),
        );

    (hub, router)
}
