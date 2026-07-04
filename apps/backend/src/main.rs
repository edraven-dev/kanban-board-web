use backend::infrastructure::config::{AppEnv, Config};
use backend::router;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let config = Config::from_env().expect("failed to load configuration");
    init_tracing(config.app_env);

    let app = router(config.app_env);

    let listener = TcpListener::bind(("0.0.0.0", config.port))
        .await
        .expect("failed to bind listener");
    tracing::info!(port = config.port, env = ?config.app_env, "backend listening");
    axum::serve(listener, app).await.expect("server crashed");
}

fn init_tracing(app_env: AppEnv) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let builder = tracing_subscriber::fmt().with_env_filter(filter);
    match app_env {
        AppEnv::Development => builder.init(),
        AppEnv::Production => builder.json().flatten_event(true).init(),
    }
}
