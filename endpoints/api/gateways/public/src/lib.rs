//! البوابة العامة: مفتوحة للزوّار، والتوكن اختياري.
//! بتتركّب في `api/server` تحت `/public`.

mod context;
mod queries;
mod types;

use async_graphql::{EmptyMutation, EmptySubscription, Schema, SchemaBuilder, http::GraphiQLSource};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    Extension, Router,
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::{Next, from_fn_with_state},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use kernel::Actor;
use services::ServiceHub;
use std::sync::Arc;

pub use context::PublicCtx;

pub type PublicSchema = Schema<queries::PublicQuery, EmptyMutation, EmptySubscription>;

pub const MOUNT: &str = "/public";

fn schema_builder() -> SchemaBuilder<queries::PublicQuery, EmptyMutation, EmptySubscription> {
    // البوابة العامة مكشوفة للنت — بنحدّ العمق والتعقيد
    Schema::build(queries::PublicQuery::default(), EmptyMutation, EmptySubscription)
        .limit_depth(8)
        .limit_complexity(200)
}

pub fn build_schema(ctx: PublicCtx) -> PublicSchema {
    schema_builder().data(ctx).finish()
}

/// SDL من غير أي حقن بيانات — بيستعمله تست السناب‑شوت.
pub fn schema_sdl() -> String {
    schema_builder().finish().sdl()
}

pub fn router(ctx: PublicCtx) -> Router {
    let hub = ctx.hub.clone();
    let schema = build_schema(ctx);

    Router::new()
        .route("/graphql", post(graphql_handler))
        .route("/playground", get(playground))
        .route("/schema.graphql", get(sdl))
        .with_state(schema)
        .layer(from_fn_with_state(hub, public_auth))
}

async fn graphql_handler(
    State(schema): State<PublicSchema>,
    Extension(actor): Extension<Actor>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner().data(actor)).await.into()
}

async fn playground() -> impl IntoResponse {
    Html(
        GraphiQLSource::build()
            .endpoint(&format!("{MOUNT}/graphql"))
            .finish(),
    )
}

async fn sdl(State(schema): State<PublicSchema>) -> String {
    schema.sdl()
}

/// التوكن هنا اختياري: من غير توكن بتبقى زائر.
async fn public_auth(
    State(hub): State<Arc<ServiceHub>>,
    mut req: Request,
    next: Next,
) -> Response {
    let header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let actor = hub.auth().authenticate_optional(header);
    req.extensions_mut().insert(actor);
    next.run(req).await
}
