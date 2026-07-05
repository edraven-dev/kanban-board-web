mod common;

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use axum::response::Response;
use backend::adapters::outbound::persistence::pg_board_repo::PgBoardRepo;
use backend::adapters::outbound::persistence::pg_project_repo::PgProjectRepo;
use backend::app_state::AppState;
use backend::domain::board::Board;
use backend::domain::ids::ColumnId;
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

async fn seed_column(pool: &sqlx::PgPool) -> ColumnId {
    let project = Project::new(EntityName::new("P").unwrap(), Position::new(0).unwrap());
    PgProjectRepo::new(pool.clone())
        .insert(&project)
        .await
        .unwrap();
    let board = Board::new(
        project.id(),
        EntityName::new("B").unwrap(),
        Position::new(0).unwrap(),
    );
    let repo = PgBoardRepo::new(pool.clone());
    repo.insert_within_limit(&board, 99).await.unwrap();
    let mut aggregate = repo.load(board.id()).await.unwrap().unwrap();
    let column_id = aggregate
        .add_column(EntityName::new("C").unwrap())
        .unwrap()
        .id();
    repo.save(&aggregate).await.unwrap();
    column_id
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

async fn create_card(pool: &sqlx::PgPool, column: ColumnId, body: Value) -> Value {
    let res = app(pool)
        .oneshot(json_request(
            Method::POST,
            &format!("/api/columns/{}/cards", column.as_uuid()),
            body,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    read_json(res).await
}

db_test! {
    async fn post_creates_a_card_with_a_description(pool: PgPool) {
        let column = seed_column(&pool).await;
        let body = create_card(&pool, column, json!({ "title": "Ship it", "description": "body" })).await;
        assert_eq!(body["title"], "Ship it");
        assert_eq!(body["description"], "body");
        assert_eq!(body["position"], 0);
        assert_eq!(body["columnId"], column.as_uuid().to_string());
    }
}

db_test! {
    async fn post_defaults_description_to_empty(pool: PgPool) {
        let column = seed_column(&pool).await;
        let body = create_card(&pool, column, json!({ "title": "No body" })).await;
        assert_eq!(body["description"], "");
    }
}

db_test! {
    async fn post_under_a_missing_column_is_404(pool: PgPool) {
        let res = app(&pool)
            .oneshot(json_request(
                Method::POST,
                &format!("/api/columns/{}/cards", Uuid::now_v7()),
                json!({ "title": "x" }),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }
}

db_test! {
    async fn post_with_a_blank_title_is_400(pool: PgPool) {
        let column = seed_column(&pool).await;
        let res = app(&pool)
            .oneshot(json_request(
                Method::POST,
                &format!("/api/columns/{}/cards", column.as_uuid()),
                json!({ "title": "  " }),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }
}

db_test! {
    async fn get_lists_cards_for_the_column(pool: PgPool) {
        let column = seed_column(&pool).await;
        create_card(&pool, column, json!({ "title": "First" })).await;
        create_card(&pool, column, json!({ "title": "Second" })).await;

        let res = app(&pool)
            .oneshot(empty_request(Method::GET, &format!("/api/columns/{}/cards", column.as_uuid())))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = read_json(res).await;
        assert_eq!(body.as_array().unwrap().len(), 2);
        assert_eq!(body[0]["title"], "First");
    }
}

db_test! {
    async fn get_one_returns_the_card_and_missing_is_404(pool: PgPool) {
        let column = seed_column(&pool).await;
        let id = create_card(&pool, column, json!({ "title": "Hi" })).await["id"]
            .as_str()
            .unwrap()
            .to_owned();

        let res = app(&pool).oneshot(empty_request(Method::GET, &format!("/api/cards/{id}"))).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(read_json(res).await["title"], "Hi");

        let missing = app(&pool)
            .oneshot(empty_request(Method::GET, &format!("/api/cards/{}", Uuid::now_v7())))
            .await
            .unwrap();
        assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    }
}

db_test! {
    async fn patch_updates_only_the_provided_fields(pool: PgPool) {
        let column = seed_column(&pool).await;
        let id = create_card(&pool, column, json!({ "title": "Old", "description": "keep" })).await["id"]
            .as_str()
            .unwrap()
            .to_owned();

        let res = app(&pool)
            .oneshot(json_request(Method::PATCH, &format!("/api/cards/{id}"), json!({ "title": "New" })))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = read_json(res).await;
        assert_eq!(body["title"], "New");
        assert_eq!(body["description"], "keep");
    }
}

db_test! {
    async fn patch_of_a_missing_card_is_404(pool: PgPool) {
        let res = app(&pool)
            .oneshot(json_request(Method::PATCH, &format!("/api/cards/{}", Uuid::now_v7()), json!({ "title": "X" })))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }
}

db_test! {
    async fn patch_with_a_blank_title_is_400(pool: PgPool) {
        let column = seed_column(&pool).await;
        let id = create_card(&pool, column, json!({ "title": "Old" })).await["id"]
            .as_str()
            .unwrap()
            .to_owned();
        let res = app(&pool)
            .oneshot(json_request(Method::PATCH, &format!("/api/cards/{id}"), json!({ "title": "" })))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }
}

db_test! {
    async fn delete_removes_a_card_then_is_404(pool: PgPool) {
        let column = seed_column(&pool).await;
        let id = create_card(&pool, column, json!({ "title": "Bye" })).await["id"]
            .as_str()
            .unwrap()
            .to_owned();

        let res = app(&pool).oneshot(empty_request(Method::DELETE, &format!("/api/cards/{id}"))).await.unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
        let again = app(&pool).oneshot(empty_request(Method::DELETE, &format!("/api/cards/{id}"))).await.unwrap();
        assert_eq!(again.status(), StatusCode::NOT_FOUND);
    }
}

db_test! {
    async fn put_move_reorders_within_a_column(pool: PgPool) {
        let column = seed_column(&pool).await;
        let a = create_card(&pool, column, json!({ "title": "A" })).await["id"].as_str().unwrap().to_owned();
        create_card(&pool, column, json!({ "title": "B" })).await;

        let res = app(&pool)
            .oneshot(json_request(
                Method::PUT,
                &format!("/api/cards/{a}/move"),
                json!({ "columnId": column.as_uuid().to_string(), "position": 1 }),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let list = read_json(
            app(&pool)
                .oneshot(empty_request(Method::GET, &format!("/api/columns/{}/cards", column.as_uuid())))
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(list[0]["title"], "B");
        assert_eq!(list[1]["title"], "A");
    }
}

db_test! {
    async fn put_move_to_a_missing_target_is_404(pool: PgPool) {
        let column = seed_column(&pool).await;
        let a = create_card(&pool, column, json!({ "title": "A" })).await["id"].as_str().unwrap().to_owned();

        let res = app(&pool)
            .oneshot(json_request(
                Method::PUT,
                &format!("/api/cards/{a}/move"),
                json!({ "columnId": Uuid::now_v7().to_string(), "position": 0 }),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }
}

db_test! {
    async fn put_move_with_a_negative_position_is_400(pool: PgPool) {
        let column = seed_column(&pool).await;
        let a = create_card(&pool, column, json!({ "title": "A" })).await["id"].as_str().unwrap().to_owned();

        let res = app(&pool)
            .oneshot(json_request(
                Method::PUT,
                &format!("/api/cards/{a}/move"),
                json!({ "columnId": column.as_uuid().to_string(), "position": -1 }),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }
}
