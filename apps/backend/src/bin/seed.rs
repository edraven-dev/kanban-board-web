use backend::adapters::outbound::persistence::pg_board_repo::PgBoardRepo;
use backend::adapters::outbound::persistence::pg_project_repo::PgProjectRepo;
use backend::application::seed::seed;
use backend::infrastructure::config::Config;
use backend::infrastructure::db;

/// Dev-only seed: populates a demo project so the UI has data to show. Never
/// runs under `APP_ENV=production`.
#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let config = Config::from_env().expect("failed to load configuration");
    if !config.app_env.is_development() {
        eprintln!("refusing to seed: APP_ENV must be `development`");
        std::process::exit(1);
    }

    let pool = db::connect(&config)
        .await
        .expect("failed to connect to the database");
    let projects = PgProjectRepo::new(pool.clone());
    let boards = PgBoardRepo::new(pool.clone());

    let summary = seed(&projects, &boards).await.expect("seed failed");
    println!(
        "seeded {} project, {} boards, {} columns, {} cards",
        summary.projects, summary.boards, summary.columns, summary.cards,
    );
}
