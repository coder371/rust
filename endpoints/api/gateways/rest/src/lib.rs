//! بوابة REST — بتمشي بنفس قواعد بوابات الجراف:
//! اللوجيك في `services/`، موديلات عرض خاصة بيها، وميدلوير خاص بيها.
//! بتتركّب في `api/server` تحت `/rest`.

mod context;
mod dto;
mod error;
mod routes;

use axum::{
    Router,
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::{Next, from_fn_with_state},
    response::Response,
    routing::get,
};
use services::ServiceHub;
use std::sync::Arc;

pub use context::RestCtx;

pub const MOUNT: &str = "/rest";

pub fn router(ctx: RestCtx) -> Router {
    let hub = ctx.hub.clone();

    Router::new()
        .route("/health", get(routes::health::handler))
        .route("/users", get(routes::list_users::handler).post(routes::create_user::handler))
        .route("/users/{id}", get(routes::user_by_id::handler))
        .route("/accounts/{id}", get(routes::account_overview::handler))
        .with_state(ctx)
        .layer(from_fn_with_state(hub, rest_auth))
}

/// التوكن اختياري: الزائر يقرأ، والأدمن بس اللي يكتب (الشرط جوّه اللوجيك).
async fn rest_auth(State(hub): State<Arc<ServiceHub>>, mut req: Request, next: Next) -> Response {
    let actor = {
        let header = req
            .headers()
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok());
        hub.auth().authenticate_optional(header)
    };

    req.extensions_mut().insert(actor);
    next.run(req).await
}
