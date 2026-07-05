mod common;

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::response::Response;
use backend::adapters::outbound::persistence::pg_board_repo::PgBoardRepo;
use backend::adapters::outbound::persistence::pg_project_repo::PgProjectRepo;
use backend::app_state::AppState;
use backend::domain::board::Board;
use backend::domain::description::Description;
use backend::domain::ids::BoardId;
use backend::domain::name::EntityName;
use backend::domain::ports::{BoardRepository, ProjectRepository};
use backend::domain::position::Position;
use backend::domain::project::Project;
use backend::domain::title::Title;
use backend::infrastructure::config::AppEnv;
use backend::router;
use common::db_test;
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

async fn seed_board(pool: &sqlx::PgPool) -> BoardId {
    let project = Project::new(EntityName::new("P").unwrap(), Position::new(0).unwrap());
    PgProjectRepo::new(pool.clone())
        .insert(&project)
        .await
        .unwrap();
    let board = Board::new(
        project.id,
        EntityName::new("B").unwrap(),
        Position::new(0).unwrap(),
    );
    PgBoardRepo::new(pool.clone())
        .insert_within_limit(&board, 99)
        .await
        .unwrap();
    board.id
}

fn app(pool: &sqlx::PgPool) -> Router {
    router(AppEnv::Development, AppState::new(pool.clone()))
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .method(Method::GET)
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

async fn read_json(res: Response) -> Value {
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

db_test! {
    async fn full_nests_columns_and_cards_ordered_by_position(pool: PgPool) {
        let board = seed_board(&pool).await;
        let repo = PgBoardRepo::new(pool.clone());

        let mut aggregate = repo.load(board).await.unwrap().unwrap();
        let todo = aggregate.add_column(EntityName::new("To Do").unwrap()).unwrap().id;
        let doing = aggregate.add_column(EntityName::new("Doing").unwrap()).unwrap().id;
        aggregate.add_column(EntityName::new("Done").unwrap()).unwrap();
        aggregate.add_card(todo, Title::new("A1").unwrap(), Description::default()).unwrap();
        aggregate.add_card(todo, Title::new("A2").unwrap(), Description::default()).unwrap();
        aggregate.add_card(doing, Title::new("C1").unwrap(), Description::default()).unwrap();
        repo.save(&aggregate).await.unwrap();

        let res = app(&pool)
            .oneshot(get(&format!("/api/boards/{}/full", board.as_uuid())))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = read_json(res).await;

        assert_eq!(body["id"], board.as_uuid().to_string());
        assert_eq!(body["name"], "B");

        let cols = body["columns"].as_array().unwrap();
        let names: Vec<&str> = cols.iter().map(|c| c["name"].as_str().unwrap()).collect();
        assert_eq!(names, ["To Do", "Doing", "Done"]);

        let todo_cards: Vec<&str> =
            cols[0]["cards"].as_array().unwrap().iter().map(|c| c["title"].as_str().unwrap()).collect();
        assert_eq!(todo_cards, ["A1", "A2"]);
        assert_eq!(cols[1]["cards"][0]["title"], "C1");
        assert_eq!(cols[2]["cards"], json!([]));
    }
}

db_test! {
    async fn full_of_a_board_without_columns_has_an_empty_array(pool: PgPool) {
        let board = seed_board(&pool).await;
        let res = app(&pool)
            .oneshot(get(&format!("/api/boards/{}/full", board.as_uuid())))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(read_json(res).await["columns"], json!([]));
    }
}

db_test! {
    async fn full_of_a_missing_board_is_404(pool: PgPool) {
        let res = app(&pool)
            .oneshot(get(&format!("/api/boards/{}/full", Uuid::now_v7())))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }
}
