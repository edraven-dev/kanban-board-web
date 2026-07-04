mod common;

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use axum::response::Response;
use backend::adapters::outbound::persistence::pg_board_repo::PgBoardRepo;
use backend::adapters::outbound::persistence::pg_column_repo::PgColumnRepo;
use backend::adapters::outbound::persistence::pg_project_repo::PgProjectRepo;
use backend::app_state::AppState;
use backend::domain::board::Board;
use backend::domain::column::Column;
use backend::domain::ids::{BoardId, ColumnId};
use backend::domain::name::EntityName;
use backend::domain::ports::{BoardRepository, ColumnRepository, LimitedInsert, ProjectRepository};
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
    PgProjectRepo::new(pool.clone()).insert(&project).await.unwrap();
    let board = Board::new(project.id, EntityName::new("B").unwrap(), Position::new(0).unwrap());
    PgBoardRepo::new(pool.clone())
        .insert_within_limit(&board, 99)
        .await
        .unwrap();
    board.id
}

fn column(board: BoardId, name: &str, position: i32) -> Column {
    Column::new(
        board,
        EntityName::new(name).unwrap(),
        Position::new(position).unwrap(),
    )
}

// ---------------------------------------------------------------------------
// Repository tests
// ---------------------------------------------------------------------------

db_test! {
    async fn insert_within_limit_creates_and_round_trips(pool: PgPool) {
        let board = seed_board(&pool).await;
        let repo = PgColumnRepo::new(pool);
        let c = column(board, "To Do", 0);

        assert_eq!(repo.insert_within_limit(&c, 99).await.unwrap(), LimitedInsert::Created);
        let fetched = repo.get(c.id).await.unwrap().unwrap();
        assert_eq!(fetched.name.as_str(), "To Do");
        assert_eq!(fetched.board_id, board);
    }
}

db_test! {
    async fn insert_under_a_missing_board_is_parent_missing(pool: PgPool) {
        let repo = PgColumnRepo::new(pool);
        let c = column(BoardId::new(), "orphan", 0);
        assert_eq!(
            repo.insert_within_limit(&c, 99).await.unwrap(),
            LimitedInsert::ParentMissing
        );
    }
}

db_test! {
    async fn insert_within_limit_enforces_the_max_in_a_transaction(pool: PgPool) {
        let board = seed_board(&pool).await;
        let repo = PgColumnRepo::new(pool);

        assert_eq!(repo.insert_within_limit(&column(board, "A", 0), 2).await.unwrap(), LimitedInsert::Created);
        assert_eq!(repo.insert_within_limit(&column(board, "B", 1), 2).await.unwrap(), LimitedInsert::Created);
        assert_eq!(repo.insert_within_limit(&column(board, "C", 2), 2).await.unwrap(), LimitedInsert::LimitReached);
        assert_eq!(repo.list_by_board(board).await.unwrap().len(), 2);
    }
}

db_test! {
    async fn list_is_scoped_to_the_board_and_ordered(pool: PgPool) {
        let repo = PgColumnRepo::new(pool.clone());
        let a = seed_board(&pool).await;
        let b = seed_board(&pool).await;
        repo.insert_within_limit(&column(a, "A2", 2), 99).await.unwrap();
        repo.insert_within_limit(&column(a, "A0", 0), 99).await.unwrap();
        repo.insert_within_limit(&column(b, "B0", 0), 99).await.unwrap();

        let names: Vec<String> = repo
            .list_by_board(a)
            .await
            .unwrap()
            .iter()
            .map(|x| x.name.as_str().to_owned())
            .collect();
        assert_eq!(names, ["A0", "A2"]);
    }
}

db_test! {
    async fn get_of_a_missing_column_is_none(pool: PgPool) {
        let repo = PgColumnRepo::new(pool);
        assert!(repo.get(ColumnId::new()).await.unwrap().is_none());
    }
}

db_test! {
    async fn update_changes_the_name(pool: PgPool) {
        let board = seed_board(&pool).await;
        let repo = PgColumnRepo::new(pool);
        let c = column(board, "Old", 0);
        repo.insert_within_limit(&c, 99).await.unwrap();
        repo.update(c.id, EntityName::new("New").unwrap()).await.unwrap();
        assert_eq!(repo.get(c.id).await.unwrap().unwrap().name.as_str(), "New");
    }
}

db_test! {
    async fn delete_removes_the_row(pool: PgPool) {
        let board = seed_board(&pool).await;
        let repo = PgColumnRepo::new(pool);
        let c = column(board, "Gone", 0);
        repo.insert_within_limit(&c, 99).await.unwrap();
        repo.delete(c.id).await.unwrap();
        assert!(repo.get(c.id).await.unwrap().is_none());
    }
}

db_test! {
    async fn reorder_rewrites_positions_transactionally(pool: PgPool) {
        let board = seed_board(&pool).await;
        let repo = PgColumnRepo::new(pool);
        let (a, b, c) = (column(board, "A", 0), column(board, "B", 1), column(board, "C", 2));
        for x in [&a, &b, &c] {
            repo.insert_within_limit(x, 99).await.unwrap();
        }
        repo.reorder(board, &[c.id, a.id, b.id]).await.unwrap();

        let ordered: Vec<ColumnId> = repo.list_by_board(board).await.unwrap().iter().map(|x| x.id).collect();
        assert_eq!(ordered, [c.id, a.id, b.id]);
    }
}

db_test! {
    async fn deleting_a_board_cascades_to_its_columns(pool: PgPool) {
        let board = seed_board(&pool).await;
        let columns = PgColumnRepo::new(pool.clone());
        columns.insert_within_limit(&column(board, "A", 0), 99).await.unwrap();

        PgBoardRepo::new(pool.clone()).delete(board).await.unwrap();
        assert!(columns.list_by_board(board).await.unwrap().is_empty());
    }
}

db_test! {
    async fn a_driver_error_becomes_a_repository_error(pool: PgPool) {
        let board = seed_board(&pool).await;
        let repo = PgColumnRepo::new(pool);
        let c = column(board, "Dup", 0);
        repo.insert_within_limit(&c, 99).await.unwrap();
        let err = repo.insert_within_limit(&c, 99).await.unwrap_err();
        assert!(err.to_string().contains("repository error"));
    }
}

db_test! {
    async fn a_row_violating_the_name_rule_fails_to_map(pool: PgPool) {
        let board = seed_board(&pool).await;
        sqlx::query("INSERT INTO columns (id, board_id, name, position) VALUES ($1, $2, '', 0)")
            .bind(Uuid::now_v7())
            .bind(board.as_uuid())
            .execute(&pool)
            .await
            .unwrap();
        assert!(PgColumnRepo::new(pool).list_by_board(board).await.is_err());
    }
}

db_test! {
    async fn a_row_violating_the_position_rule_fails_to_map(pool: PgPool) {
        let board = seed_board(&pool).await;
        let id = Uuid::now_v7();
        sqlx::query("INSERT INTO columns (id, board_id, name, position) VALUES ($1, $2, 'ok', -1)")
            .bind(id)
            .bind(board.as_uuid())
            .execute(&pool)
            .await
            .unwrap();
        assert!(PgColumnRepo::new(pool).get(ColumnId::from_uuid(id)).await.is_err());
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
        let repo = PgColumnRepo::new(pool.clone());
        for i in 0..99 {
            let c = column(board, &format!("C{i}"), i);
            assert_eq!(repo.insert_within_limit(&c, 99).await.unwrap(), LimitedInsert::Created);
        }
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
