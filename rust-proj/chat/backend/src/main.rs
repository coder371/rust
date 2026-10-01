//! خادم شات: REST للحسابات والغرف، و WebSocket للرسائل الحيّة.

mod auth;
mod config;
mod error;
mod models;
mod rooms;
mod state;
mod ws;

use axum::{
    http::{header, HeaderValue, Method},
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{config::Config, state::AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "chat_backend=info,tower_http=warn".into()),
        )
        .init();

    let config = Config::from_env()?;
    let bind_addr = config.bind_addr.clone();

    let cors = build_cors(&config)?;
    let state = AppState::new(config);

    let api = Router::new()
        .route("/auth/register", post(auth::routes::register))
        .route("/auth/login", post(auth::routes::login))
        .route("/auth/me", get(auth::routes::me))
        .route("/rooms", get(rooms::list_rooms).post(rooms::create_room))
        .route("/rooms/{room_id}/messages", get(rooms::room_messages));

    let app = Router::new()
        .route("/", get(index))
        .route("/health", get(|| async { "ok" }))
        .nest("/api", api)
        .route("/ws", get(ws::ws_handler))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    tracing::info!("listening on http://{bind_addr}");

    axum::serve(listener, app).await?;

    Ok(())
}

/// رد على الجذر بيوضّح إن ده الـ API مش واجهة التطبيق.
///
/// من غيره الزيارة بالمتصفح بتدّي 404 فاضية، واللي بيخلّي حد يفتكر
/// إن الخادم واقع وهو شغال تمام.
async fn index() -> Json<serde_json::Value> {
    Json(json!({
        "service": "chat-backend",
        "note": "ده الـ API. واجهة التطبيق بتشتغل على منفذ تاني (5173 في التطوير).",
        "endpoints": [
            "GET  /health",
            "POST /api/auth/register",
            "POST /api/auth/login",
            "GET  /api/auth/me",
            "GET  /api/rooms",
            "POST /api/rooms",
            "GET  /api/rooms/{room_id}/messages",
            "GET  /ws?token=…&room=…"
        ]
    }))
}

/// CORS مقيّد بالأصول المذكورة في الإعدادات.
///
/// مش `Any` عن قصد: الفرونت بيبعت هيدر `Authorization`، والمتصفح بيرفض
/// إرسال بيانات اعتماد لأصل مفتوح للكل.
fn build_cors(config: &Config) -> Result<CorsLayer, Box<dyn std::error::Error>> {
    let origins = config
        .allowed_origins
        .iter()
        .map(|origin| origin.parse::<HeaderValue>())
        .collect::<Result<Vec<_>, _>>()?;

    Ok(CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]))
}
