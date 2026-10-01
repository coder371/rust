//! بوابة الإدارة: سكيما + راوتر + الميدلوير بتاعها.
//! بتتركّب في `api/server` تحت `/admin`.

mod context;
mod guard;
mod mutations;
mod queries;
mod types;

use async_graphql::{EmptySubscription, Schema, SchemaBuilder, http::GraphiQLSource};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    Extension, Router,
    extract::{Request, State},
    http::{StatusCode, header::AUTHORIZATION},
    middleware::{Next, from_fn_with_state},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use kernel::Actor;
use services::ServiceHub;
use std::sync::Arc;

pub use context::AdminCtx;

pub type AdminSchema = Schema<queries::AdminQuery, mutations::AdminMutation, EmptySubscription>;

pub const MOUNT: &str = "/admin";

fn schema_builder() -> SchemaBuilder<queries::AdminQuery, mutations::AdminMutation, EmptySubscription>
{
    Schema::build(
        queries::AdminQuery::default(),
        mutations::AdminMutation::default(),
        EmptySubscription,
    )
}

pub fn build_schema(ctx: AdminCtx) -> AdminSchema {
    schema_builder().data(ctx).finish()
}

/// SDL من غير أي حقن بيانات — بيستعمله تست السناب‑شوت.
pub fn schema_sdl() -> String {
    schema_builder().finish().sdl()
}

pub fn router(ctx: AdminCtx) -> Router {
    let hub = ctx.hub.clone();
    let schema = build_schema(ctx);

    // الراوتس المحميّة بتوكن الأدمن
    let guarded = Router::new()
        .route("/graphql", post(graphql_handler))
        .with_state(schema.clone())
        .layer(from_fn_with_state(hub, admin_auth));

    Router::new()
        .route("/playground", get(playground))
        .route("/schema.graphql", get(sdl))
        .with_state(schema)
        .merge(guarded)
}

async fn graphql_handler(
    State(schema): State<AdminSchema>,
    Extension(actor): Extension<Actor>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    // الـ Actor بيتحقن مع كل ريكوست، مش مع السكيما
    schema.execute(req.into_inner().data(actor)).await.into()
}

async fn playground() -> impl IntoResponse {
    Html(
        GraphiQLSource::build()
            .endpoint(&format!("{MOUNT}/graphql"))
            .finish(),
    )
}

async fn sdl(State(schema): State<AdminSchema>) -> String {
    schema.sdl()
}

/// ميدلوير بوابة الإدارة: توكن أدمن إجباري.
async fn admin_auth(
    State(hub): State<Arc<ServiceHub>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let actor = hub.auth().authenticate(header).map_err(|_| StatusCode::UNAUTHORIZED)?;
    if !actor.is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }

    req.extensions_mut().insert(actor);
    Ok(next.run(req).await)
}
