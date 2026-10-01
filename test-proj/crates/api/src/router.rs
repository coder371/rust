use crate::schema::QumraSchema;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    Router,
    extract::State,
    http::HeaderMap,
    routing::{get, post},
};
use qumra_kernel::{Permission, StoreId, TenantContext};

#[derive(Clone)]
pub struct ApiState {
    pub schema: QumraSchema,
}

/// في الإنتاج: JWT يُفكّ ويُتحقّق منه هنا وتُشتقّ الصلاحيات من الدور.
/// في هذا المثال: رأس x-store-id يكفي لإبراز الفكرة —
/// المهم أن الهوية تأتي من الـ middleware لا من مدخلات الاستعلام.
fn tenant_from_headers(h: &HeaderMap) -> Option<TenantContext> {
    let store = h.get("x-store-id")?.to_str().ok()?;
    Some(TenantContext {
        store_id: StoreId::new(store),
        actor: h
            .get("x-actor")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("owner")
            .to_string(),
        permissions: vec![
            Permission::OrdersRead,
            Permission::OrdersWrite,
            Permission::InventoryRead,
            Permission::InventoryWrite,
            Permission::NotificationsRead,
        ],
    })
}

async fn graphql_handler(
    State(st): State<ApiState>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner();

    // الهوية تُحقن في سياق التنفيذ، فيراها كل resolver بلا تمرير يدوي
    if let Some(t) = tenant_from_headers(&headers) {
        req = req.data(t);
    }

    st.schema.execute(req).await.into()
}

async fn health() -> &'static str {
    "ok"
}

pub fn router(schema: QumraSchema) -> Router {
    Router::new()
        .route("/graphql", post(graphql_handler))
        .route("/health", get(health))
        .with_state(ApiState { schema })
}
