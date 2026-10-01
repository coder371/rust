//! بوابة الشركاء: توكن على شكل `Bearer <partner_token>:<tenant>`.
//! بتتركّب في `api/server` تحت `/partner`.

mod context;
mod guard;
mod queries;
mod types;

use async_graphql::{EmptyMutation, EmptySubscription, Schema, SchemaBuilder, http::GraphiQLSource};
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

pub use context::PartnerCtx;

pub type PartnerSchema = Schema<queries::PartnerQuery, EmptyMutation, EmptySubscription>;

pub const MOUNT: &str = "/partner";

fn schema_builder() -> SchemaBuilder<queries::PartnerQuery, EmptyMutation, EmptySubscription> {
    Schema::build(queries::PartnerQuery::default(), EmptyMutation, EmptySubscription)
        .limit_depth(10)
}

pub fn build_schema(ctx: PartnerCtx) -> PartnerSchema {
    schema_builder().data(ctx).finish()
}

/// SDL من غير أي حقن بيانات — بيستعمله تست السناب‑شوت.
pub fn schema_sdl() -> String {
    schema_builder().finish().sdl()
}

pub fn router(ctx: PartnerCtx) -> Router {
    let hub = ctx.hub.clone();
    let schema = build_schema(ctx);

    let guarded = Router::new()
        .route("/graphql", post(graphql_handler))
        .with_state(schema.clone())
        .layer(from_fn_with_state(hub, partner_auth));

    Router::new()
        .route("/playground", get(playground))
        .route("/schema.graphql", get(sdl))
        .with_state(schema)
        .merge(guarded)
}

async fn graphql_handler(
    State(schema): State<PartnerSchema>,
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

async fn sdl(State(schema): State<PartnerSchema>) -> String {
    schema.sdl()
}

async fn partner_auth(
    State(hub): State<Arc<ServiceHub>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let actor = hub.auth().authenticate(header).map_err(|_| StatusCode::UNAUTHORIZED)?;
    if !actor.is_partner() {
        return Err(StatusCode::FORBIDDEN);
    }

    req.extensions_mut().insert(actor);
    Ok(next.run(req).await)
}
