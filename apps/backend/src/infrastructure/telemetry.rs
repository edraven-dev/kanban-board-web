use tracing_subscriber::EnvFilter;

use crate::infrastructure::config::AppEnv;

pub fn init_tracing(app_env: AppEnv) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let builder = tracing_subscriber::fmt().with_env_filter(filter);
    match app_env {
        AppEnv::Development => builder.init(),
        AppEnv::Production => builder.json().flatten_event(true).init(),
    }
}
