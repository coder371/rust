mod config;
mod eventbus;
mod wiring;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let cfg = config::Config::from_env();
    let (hub, app) = wiring::build(&cfg);

    let listener = match tokio::net::TcpListener::bind(cfg.addr()).await {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("❌ failed to bind {}: {err}", cfg.addr());
            return;
        }
    };

    println!("🚀 http://localhost:{}", cfg.port);
    println!("   ❷ services: {}", hub.registry().names().join(", "));
    for mount in [gw_admin::MOUNT, gw_public::MOUNT, gw_partner::MOUNT] {
        println!("   ❸ {mount}/graphql   (playground: {mount}/playground)");
    }
    println!("   ❸ {}/users   {}/health", gw_rest::MOUNT, gw_rest::MOUNT);

    if let Err(err) = axum::serve(listener, app).await {
        eprintln!("❌ server error: {err}");
    }
}
