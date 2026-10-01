mod config;
mod data;
mod pipeline;
mod templates;

use std::sync::atomic::Ordering;

use axum::{
    Router,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
};
use serde_json::json;

use crate::config::Config;

async fn page(config: &'static Config) -> impl IntoResponse {
    match pipeline::render_storefront_page(config).await {
        Ok(html) => Html(html).into_response(),
        Err(error) => {
            // أخطاء القوالب متداخلة — من غير المشي على السلسلة
            // بنشوف "فشل include" وبس، مش السبب الحقيقي.
            let mut detail = format!("{error:#}");
            let mut source = std::error::Error::source(&error);
            while let Some(inner) = source {
                detail.push_str(&format!("\n  ← {inner}"));
                source = inner.source();
            }
            (StatusCode::INTERNAL_SERVER_ERROR, detail).into_response()
        }
    }
}

async fn stats(config: &'static Config) -> impl IntoResponse {
    pipeline::MONGO_OPS.store(0, Ordering::Relaxed);
    pipeline::REDIS_OPS.store(0, Ordering::Relaxed);

    let started = std::time::Instant::now();
    let html = pipeline::render_storefront_page(config).await.unwrap_or_default();
    let ms = started.elapsed().as_secs_f64() * 1000.0;

    axum::Json(json!({
        "mode": config.label(),
        "ms": (ms * 10.0).round() / 10.0,
        "htmlBytes": html.len(),
        "mongoOps": pipeline::MONGO_OPS.load(Ordering::Relaxed),
        "redisOps": pipeline::REDIS_OPS.load(Ordering::Relaxed),
    }))
    .into_response()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // عدد الخيوط صريح عشان المقارنة تبقى عادلة: نسخة نود الواحدة
    // بتشتغل على نواة واحدة، فلازم نقدر نحدّ راست بنفس العدد.
    let workers: usize = std::env::var("WORKERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1));

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(workers)
        .enable_all()
        .build()?
        .block_on(run(workers))
}

async fn run(workers: usize) -> Result<(), Box<dyn std::error::Error>> {
    let config: &'static Config = Box::leak(Box::new(Config::from_env()));

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/stats", get(move || stats(config)))
        .route("/", get(move || page(config)));

    let addr = format!("127.0.0.1:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    println!("storefront-sim-rs [{}] {} خيط on http://{addr}", config.label(), workers);

    axum::serve(listener, app).await?;
    Ok(())
}
