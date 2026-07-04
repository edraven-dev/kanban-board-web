mod common;

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use axum::response::Response;
use backend::adapters::outbound::persistence::pg_board_repo::PgBoardRepo;
use backend::adapters::outbound::persistence::pg_card_repo::PgCardRepo;
use backend::adapters::outbound::persistence::pg_column_repo::PgColumnRepo;
use backend::adapters::outbound::persistence::pg_project_repo::PgProjectRepo;
use backend::app_state::AppState;
use backend::domain::board::Board;
use backend::domain::card::Card;
use backend::domain::column::Column;
use backend::domain::description::Description;
use backend::domain::ids::{CardId, ColumnId};
use backend::domain::name::EntityName;
use backend::domain::ports::{
    BoardRepository, CardRepository, ColumnRepository, InsertOutcome, MoveOutcome,
    ProjectRepository,
};
use backend::domain::position::Position;
use backend::domain::project::Project;
use backend::domain::title::Title;
use backend::infrastructure::config::AppEnv;
use backend::router;
use common::db_test;
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

async fn seed_column(pool: &sqlx::PgPool) -> ColumnId {
    let project = Project::new(EntityName::new("P").unwrap(), Position::new(0).unwrap());
    PgProjectRepo::new(pool.clone()).insert(&project).await.unwrap();
    let board = Board::new(project.id, EntityName::new("B").unwrap(), Position::new(0).unwrap());
    PgBoardRepo::new(pool.clone())
        .insert_within_limit(&board, 99)
        .await
        .unwrap();
    let column = Column::new(board.id, EntityName::new("C").unwrap(), Position::new(0).unwrap());
    PgColumnRepo::new(pool.clone())
        .insert_within_limit(&column, 99)
        .await
        .unwrap();
    column.id
}

fn card(column: ColumnId, title: &str, description: &str, position: i32) -> Card {
    Card::new(
        column,
        Title::new(title).unwrap(),
        Description::new(description).unwrap(),
        Position::new(position).unwrap(),
    )
}

async fn titles(repo: &PgCardRepo, column: ColumnId) -> Vec<String> {
    repo.list_by_column(column)
        .await
        .unwrap()
        .iter()
        .map(|c| c.title.as_str().to_owned())
        .collect()
}

// ---------------------------------------------------------------------------
// Repository tests
// ---------------------------------------------------------------------------

db_test! {
    async fn insert_creates_and_round_trips(pool: PgPool) {
        let column = seed_column(&pool).await;
        let repo = PgCardRepo::new(pool);
        let c = card(column, "Ship it", "with details", 0);

        assert_eq!(repo.insert(&c).await.unwrap(), InsertOutcome::Inserted);
        let fetched = repo.get(c.id).await.unwrap().unwrap();
        assert_eq!(fetched.title.as_str(), "Ship it");
        assert_eq!(fetched.description.as_str(), "with details");
        assert_eq!(fetched.column_id, column);
    }
}

db_test! {
    async fn insert_under_a_missing_column_is_parent_missing(pool: PgPool) {
        let repo = PgCardRepo::new(pool);
        let c = card(ColumnId::new(), "orphan", "", 0);
        assert_eq!(repo.insert(&c).await.unwrap(), InsertOutcome::ParentMissing);
    }
}

db_test! {
    async fn list_is_scoped_to_the_column_and_ordered(pool: PgPool) {
        let repo = PgCardRepo::new(pool.clone());
        let a = seed_column(&pool).await;
        let b = seed_column(&pool).await;
        repo.insert(&card(a, "A2", "", 2)).await.unwrap();
        repo.insert(&card(a, "A0", "", 0)).await.unwrap();
        repo.insert(&card(b, "B0", "", 0)).await.unwrap();

        assert_eq!(titles(&repo, a).await, ["A0", "A2"]);
    }
}

db_test! {
    async fn get_of_a_missing_card_is_none(pool: PgPool) {
        let repo = PgCardRepo::new(pool);
        assert!(repo.get(CardId::new()).await.unwrap().is_none());
    }
}

db_test! {
    async fn update_changes_fields_and_preserves_created_at(pool: PgPool) {
        let column = seed_column(&pool).await;
        let repo = PgCardRepo::new(pool);
        let c = card(column, "Old", "old body", 0);
        repo.insert(&c).await.unwrap();
        let created_at = repo.get(c.id).await.unwrap().unwrap().created_at;

        repo.update(c.id, Title::new("New").unwrap(), Description::new("new body").unwrap())
            .await
            .unwrap();

        let fetched = repo.get(c.id).await.unwrap().unwrap();
        assert_eq!(fetched.title.as_str(), "New");
        assert_eq!(fetched.description.as_str(), "new body");
        assert_eq!(fetched.created_at, created_at);
        assert!(fetched.updated_at >= created_at);
    }
}

db_test! {
    async fn delete_removes_the_row(pool: PgPool) {
        let column = seed_column(&pool).await;
        let repo = PgCardRepo::new(pool);
        let c = card(column, "Gone", "", 0);
        repo.insert(&c).await.unwrap();
        repo.delete(c.id).await.unwrap();
        assert!(repo.get(c.id).await.unwrap().is_none());
    }
}

db_test! {
    async fn move_within_a_column_rewrites_positions(pool: PgPool) {
        let column = seed_column(&pool).await;
        let repo = PgCardRepo::new(pool);
        let (a, b, c) = (card(column, "A", "", 0), card(column, "B", "", 1), card(column, "C", "", 2));
        for x in [&a, &b, &c] {
            repo.insert(x).await.unwrap();
        }

        assert_eq!(repo.move_card(a.id, column, 2).await.unwrap(), MoveOutcome::Moved);
        assert_eq!(titles(&repo, column).await, ["B", "C", "A"]);

        let positions: Vec<i32> = repo
            .list_by_column(column)
            .await
            .unwrap()
            .iter()
            .map(|c| c.position.value())
            .collect();
        assert_eq!(positions, [0, 1, 2]);
    }
}

db_test! {
    async fn move_across_columns_moves_and_reseqs_both_sides(pool: PgPool) {
        let src = seed_column(&pool).await;
        let dst = seed_column(&pool).await;
        let repo = PgCardRepo::new(pool);
        let a = card(src, "A", "", 0);
        repo.insert(&a).await.unwrap();
        repo.insert(&card(dst, "X", "", 0)).await.unwrap();
        repo.insert(&card(dst, "Y", "", 1)).await.unwrap();

        assert_eq!(repo.move_card(a.id, dst, 1).await.unwrap(), MoveOutcome::Moved);

        assert!(titles(&repo, src).await.is_empty());
        assert_eq!(titles(&repo, dst).await, ["X", "A", "Y"]);
        assert_eq!(repo.get(a.id).await.unwrap().unwrap().column_id, dst);
    }
}

db_test! {
    async fn move_of_a_missing_card_is_card_missing(pool: PgPool) {
        let column = seed_column(&pool).await;
        let repo = PgCardRepo::new(pool);
        assert_eq!(
            repo.move_card(CardId::new(), column, 0).await.unwrap(),
            MoveOutcome::CardMissing
        );
    }
}

db_test! {
    async fn move_to_a_missing_target_is_target_missing(pool: PgPool) {
        let column = seed_column(&pool).await;
        let repo = PgCardRepo::new(pool);
        let c = card(column, "A", "", 0);
        repo.insert(&c).await.unwrap();
        assert_eq!(
            repo.move_card(c.id, ColumnId::new(), 0).await.unwrap(),
            MoveOutcome::TargetMissing
        );
    }
}

db_test! {
    async fn deleting_a_column_cascades_to_its_cards(pool: PgPool) {
        let column = seed_column(&pool).await;
        let repo = PgCardRepo::new(pool.clone());
        repo.insert(&card(column, "A", "", 0)).await.unwrap();

        PgColumnRepo::new(pool.clone()).delete(column).await.unwrap();
        assert!(repo.list_by_column(column).await.unwrap().is_empty());
    }
}

db_test! {
    async fn a_driver_error_becomes_a_repository_error(pool: PgPool) {
        let column = seed_column(&pool).await;
        let repo = PgCardRepo::new(pool);
        let c = card(column, "Dup", "", 0);
        repo.insert(&c).await.unwrap();
        let err = repo.insert(&c).await.unwrap_err();
        assert!(err.to_string().contains("repository error"));
    }
}

db_test! {
    async fn a_row_violating_the_title_rule_fails_to_map(pool: PgPool) {
        let column = seed_column(&pool).await;
        sqlx::query("INSERT INTO cards (id, column_id, title, position) VALUES ($1, $2, '', 0)")
            .bind(Uuid::now_v7())
            .bind(column.as_uuid())
            .execute(&pool)
            .await
            .unwrap();
        assert!(PgCardRepo::new(pool).list_by_column(column).await.is_err());
    }
}

// ---------------------------------------------------------------------------
// HTTP handler tests
// ---------------------------------------------------------------------------

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
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
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
