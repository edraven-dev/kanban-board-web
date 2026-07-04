use backend::app_state::AppState;
use backend::infrastructure::config::Config;
use backend::infrastructure::telemetry::init_tracing;
use backend::infrastructure::db;
use backend::router;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let config = Config::from_env().expect("failed to load configuration");
    init_tracing(config.app_env);

    let pool = db::connect(&config)
        .await
        .expect("failed to connect to the database");
    // Wired into the router in B3+.
    let _state = AppState::new(pool);

    let app = router(config.app_env);

    let listener = TcpListener::bind(("0.0.0.0", config.port))
        .await
        .expect("failed to bind listener");
    tracing::info!(port = config.port, env = ?config.app_env, "backend listening");
    axum::serve(listener, app).await.expect("server crashed");
}
