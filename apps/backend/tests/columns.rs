mod common;

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use axum::response::Response;
use backend::adapters::outbound::persistence::pg_board_repo::PgBoardRepo;
use backend::adapters::outbound::persistence::pg_project_repo::PgProjectRepo;
use backend::app_state::AppState;
use backend::domain::board::Board;
use backend::domain::ids::BoardId;
use backend::domain::name::EntityName;
use backend::domain::ports::{BoardRepository, ProjectRepository};
use backend::domain::position::Position;
use backend::domain::project::Project;
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

async fn seed_columns(pool: &sqlx::PgPool, board: BoardId, count: usize) {
    let repo = PgBoardRepo::new(pool.clone());
    let mut aggregate = repo.load(board).await.unwrap().unwrap();
    for i in 0..count {
        aggregate
            .add_column(EntityName::new(format!("C{i}")).unwrap())
            .unwrap();
    }
    repo.save(&aggregate).await.unwrap();
}

fn app(pool: &sqlx::PgPool) -> Router {
    router(AppEnv::Development, AppState::new(pool.clone()))
}

fn json_request(method: Method, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn empty_request(method: Method, uri: &str) -> Request<Body> {
    Request::builder()
        .method(method)
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

async fn create_column(pool: &sqlx::PgPool, board: BoardId, name: &str) -> Value {
    let res = app(pool)
        .oneshot(json_request(
            Method::POST,
            &format!("/api/boards/{}/columns", board.as_uuid()),
            json!({ "name": name }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    read_json(res).await
}

db_test! {
    async fn post_creates_a_column(pool: PgPool) {
        let board = seed_board(&pool).await;
        let body = create_column(&pool, board, "To Do").await;
        assert_eq!(body["name"], "To Do");
        assert_eq!(body["position"], 0);
        assert_eq!(body["boardId"], board.as_uuid().to_string());
    }
}

db_test! {
    async fn post_under_a_missing_board_is_404(pool: PgPool) {
        let res = app(&pool)
            .oneshot(json_request(
                Method::POST,
                &format!("/api/boards/{}/columns", Uuid::now_v7()),
                json!({ "name": "x" }),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }
}

db_test! {
    async fn post_with_a_blank_name_is_400(pool: PgPool) {
        let board = seed_board(&pool).await;
        let res = app(&pool)
            .oneshot(json_request(
                Method::POST,
                &format!("/api/boards/{}/columns", board.as_uuid()),
                json!({ "name": "  " }),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }
}

db_test! {
    async fn post_beyond_the_limit_is_409(pool: PgPool) {
        let board = seed_board(&pool).await;
        seed_columns(&pool, board, 99).await;
        let res = app(&pool)
            .oneshot(json_request(
                Method::POST,
                &format!("/api/boards/{}/columns", board.as_uuid()),
                json!({ "name": "overflow" }),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::CONFLICT);
        assert_eq!(read_json(res).await["error"]["code"], "limit_exceeded");
    }
}

db_test! {
    async fn get_lists_columns_for_the_board(pool: PgPool) {
        let board = seed_board(&pool).await;
        create_column(&pool, board, "First").await;
        create_column(&pool, board, "Second").await;

        let res = app(&pool)
            .oneshot(empty_request(Method::GET, &format!("/api/boards/{}/columns", board.as_uuid())))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = read_json(res).await;
        assert_eq!(body.as_array().unwrap().len(), 2);
        assert_eq!(body[0]["name"], "First");
    }
}

db_test! {
    async fn patch_updates_a_column(pool: PgPool) {
        let board = seed_board(&pool).await;
        let id = create_column(&pool, board, "Old").await["id"].as_str().unwrap().to_owned();

        let res = app(&pool)
            .oneshot(json_request(Method::PATCH, &format!("/api/columns/{id}"), json!({ "name": "New" })))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(read_json(res).await["name"], "New");
    }
}

db_test! {
    async fn patch_of_a_missing_column_is_404(pool: PgPool) {
        let res = app(&pool)
            .oneshot(json_request(Method::PATCH, &format!("/api/columns/{}", Uuid::now_v7()), json!({ "name": "X" })))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }
}

db_test! {
    async fn delete_removes_a_column_then_is_404(pool: PgPool) {
        let board = seed_board(&pool).await;
        let id = create_column(&pool, board, "Bye").await["id"].as_str().unwrap().to_owned();

        let res = app(&pool).oneshot(empty_request(Method::DELETE, &format!("/api/columns/{id}"))).await.unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
        let again = app(&pool).oneshot(empty_request(Method::DELETE, &format!("/api/columns/{id}"))).await.unwrap();
        assert_eq!(again.status(), StatusCode::NOT_FOUND);
    }
}

db_test! {
    async fn put_reorder_changes_the_order(pool: PgPool) {
        let board = seed_board(&pool).await;
        let a = create_column(&pool, board, "A").await["id"].as_str().unwrap().to_owned();
        let b = create_column(&pool, board, "B").await["id"].as_str().unwrap().to_owned();

        let res = app(&pool)
            .oneshot(json_request(
                Method::PUT,
                &format!("/api/boards/{}/columns/reorder", board.as_uuid()),
                json!({ "orderedIds": [b, a] }),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let list = read_json(
            app(&pool)
                .oneshot(empty_request(Method::GET, &format!("/api/boards/{}/columns", board.as_uuid())))
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(list[0]["name"], "B");
    }
}

db_test! {
    async fn put_reorder_with_a_bad_id_set_is_422(pool: PgPool) {
        let board = seed_board(&pool).await;
        create_column(&pool, board, "A").await;
        let res = app(&pool)
            .oneshot(json_request(
                Method::PUT,
                &format!("/api/boards/{}/columns/reorder", board.as_uuid()),
                json!({ "orderedIds": [Uuid::now_v7().to_string()] }),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
