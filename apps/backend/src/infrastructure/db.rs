use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::infrastructure::config::Config;

pub async fn connect(config: &Config) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
