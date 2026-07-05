mod common;

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use axum::response::Response;
use backend::adapters::outbound::persistence::pg_board_repo::PgBoardRepo;
use backend::adapters::outbound::persistence::pg_project_repo::PgProjectRepo;
use backend::app_state::AppState;
use backend::domain::board::Board;
use backend::domain::ids::{BoardId, ProjectId};
use backend::domain::name::EntityName;
use backend::domain::ports::{BoardRepository, LimitedInsert, ProjectRepository};
use backend::domain::position::Position;
use backend::domain::project::Project;
use backend::infrastructure::config::AppEnv;
use backend::router;
use common::db_test;
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

async fn seed_project(pool: &sqlx::PgPool) -> ProjectId {
    let repo = PgProjectRepo::new(pool.clone());
    let p = Project::new(
        EntityName::new("Parent").unwrap(),
        Position::new(0).unwrap(),
    );
    repo.insert(&p).await.unwrap();
    p.id()
}

fn board(project: ProjectId, name: &str, position: i32) -> Board {
    Board::new(
        project,
        EntityName::new(name).unwrap(),
        Position::new(position).unwrap(),
    )
}

db_test! {
    async fn insert_within_limit_creates_and_round_trips(pool: PgPool) {
        let project = seed_project(&pool).await;
        let repo = PgBoardRepo::new(pool);
        let b = board(project, "Backlog", 0);

        assert_eq!(repo.insert_within_limit(&b, 99).await.unwrap(), LimitedInsert::Created);
        let fetched = repo.summary(b.id()).await.unwrap().unwrap();
        assert_eq!(fetched.name().as_str(), "Backlog");
        assert_eq!(fetched.project_id(), project);
        // Position is assigned by the repo (first board in the project → 0).
        assert_eq!(fetched.position().value(), 0);
    }
}

db_test! {
    async fn insert_assigns_the_next_position_after_a_gap(pool: PgPool) {
        let project = seed_project(&pool).await;
        let repo = PgBoardRepo::new(pool);
        let first = board(project, "A", 0);
        let second = board(project, "B", 0);
        repo.insert_within_limit(&first, 99).await.unwrap();
        repo.insert_within_limit(&second, 99).await.unwrap();
        repo.delete(first.id()).await.unwrap(); // leaves a gap; only position 1 remains

        let third = board(project, "C", 0);
        repo.insert_within_limit(&third, 99).await.unwrap();
        // Uses max(position)+1, so 2 — a count-based guess (1) would collide with B.
        assert_eq!(
            repo.summary(third.id()).await.unwrap().unwrap().position().value(),
            2
        );
    }
}

db_test! {
    // Existence is the directory's job now; the FK is only a backstop, so an orphan errors.
    async fn insert_under_a_missing_project_violates_the_foreign_key(pool: PgPool) {
        let repo = PgBoardRepo::new(pool);
        let b = board(ProjectId::new(), "orphan", 0);
        let err = repo.insert_within_limit(&b, 99).await.unwrap_err();
        assert!(err.to_string().contains("repository error"));
    }
}

db_test! {
    async fn insert_within_limit_enforces_the_max_in_a_transaction(pool: PgPool) {
        let project = seed_project(&pool).await;
        let repo = PgBoardRepo::new(pool);

        assert_eq!(repo.insert_within_limit(&board(project, "A", 0), 2).await.unwrap(), LimitedInsert::Created);
        assert_eq!(repo.insert_within_limit(&board(project, "B", 1), 2).await.unwrap(), LimitedInsert::Created);
        assert_eq!(repo.insert_within_limit(&board(project, "C", 2), 2).await.unwrap(), LimitedInsert::LimitReached);
        assert_eq!(repo.list_by_project(project).await.unwrap().len(), 2);
    }
}

db_test! {
    async fn list_is_scoped_to_the_project_and_ordered(pool: PgPool) {
        let repo = PgBoardRepo::new(pool.clone());
        let a = seed_project(&pool).await;
        let b = seed_project(&pool).await;
        // insert_within_limit appends, so insertion order is position order.
        repo.insert_within_limit(&board(a, "A0", 0), 99).await.unwrap();
        repo.insert_within_limit(&board(a, "A1", 0), 99).await.unwrap();
        repo.insert_within_limit(&board(b, "B0", 0), 99).await.unwrap();

        let names: Vec<String> = repo
            .list_by_project(a)
            .await
            .unwrap()
            .iter()
            .map(|x| x.name().as_str().to_owned())
            .collect();
        assert_eq!(names, ["A0", "A1"]);
    }
}

db_test! {
    async fn get_of_a_missing_board_is_none(pool: PgPool) {
        let repo = PgBoardRepo::new(pool);
        assert!(repo.summary(BoardId::new()).await.unwrap().is_none());
    }
}

db_test! {
    async fn update_changes_the_name(pool: PgPool) {
        let project = seed_project(&pool).await;
        let repo = PgBoardRepo::new(pool);
        let b = board(project, "Old", 0);
        repo.insert_within_limit(&b, 99).await.unwrap();
        repo.update(b.id(), EntityName::new("New").unwrap()).await.unwrap();
        assert_eq!(repo.summary(b.id()).await.unwrap().unwrap().name().as_str(), "New");
    }
}

db_test! {
    async fn delete_removes_the_row(pool: PgPool) {
        let project = seed_project(&pool).await;
        let repo = PgBoardRepo::new(pool);
        let b = board(project, "Gone", 0);
        repo.insert_within_limit(&b, 99).await.unwrap();
        repo.delete(b.id()).await.unwrap();
        assert!(repo.summary(b.id()).await.unwrap().is_none());
    }
}

db_test! {
    async fn reorder_rewrites_positions_transactionally(pool: PgPool) {
        let project = seed_project(&pool).await;
        let repo = PgBoardRepo::new(pool);
        let (a, b, c) = (board(project, "A", 0), board(project, "B", 1), board(project, "C", 2));
        for x in [&a, &b, &c] {
            repo.insert_within_limit(x, 99).await.unwrap();
        }
        repo.reorder(project, &[c.id(), a.id(), b.id()]).await.unwrap();

        let ordered: Vec<BoardId> = repo.list_by_project(project).await.unwrap().iter().map(|x| x.id()).collect();
        assert_eq!(ordered, [c.id(), a.id(), b.id()]);
    }
}

db_test! {
    async fn deleting_a_project_cascades_to_its_boards(pool: PgPool) {
        let project = seed_project(&pool).await;
        let boards = PgBoardRepo::new(pool.clone());
        boards.insert_within_limit(&board(project, "A", 0), 99).await.unwrap();

        PgProjectRepo::new(pool.clone()).delete(project).await.unwrap();
        assert!(boards.list_by_project(project).await.unwrap().is_empty());
    }
}

db_test! {
    async fn a_driver_error_becomes_a_repository_error(pool: PgPool) {
        let project = seed_project(&pool).await;
        let repo = PgBoardRepo::new(pool);
        let b = board(project, "Dup", 0);
        repo.insert_within_limit(&b, 99).await.unwrap();
        let err = repo.insert_within_limit(&b, 99).await.unwrap_err();
        assert!(err.to_string().contains("repository error"));
    }
}

db_test! {
    async fn a_row_violating_the_name_rule_fails_to_map(pool: PgPool) {
        let project = seed_project(&pool).await;
        sqlx::query("INSERT INTO boards (id, project_id, name, position) VALUES ($1, $2, '', 0)")
            .bind(Uuid::now_v7())
            .bind(project.as_uuid())
            .execute(&pool)
            .await
            .unwrap();
        assert!(PgBoardRepo::new(pool).list_by_project(project).await.is_err());
    }
}

db_test! {
    async fn a_row_violating_the_position_rule_fails_to_map(pool: PgPool) {
        let project = seed_project(&pool).await;
        let id = Uuid::now_v7();
        sqlx::query("INSERT INTO boards (id, project_id, name, position) VALUES ($1, $2, 'ok', -1)")
            .bind(id)
            .bind(project.as_uuid())
            .execute(&pool)
            .await
            .unwrap();
        assert!(PgBoardRepo::new(pool).summary(BoardId::from_uuid(id)).await.is_err());
    }
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

async fn create_board(pool: &sqlx::PgPool, project: ProjectId, name: &str) -> Value {
    let res = app(pool)
        .oneshot(json_request(
            Method::POST,
            &format!("/api/projects/{}/boards", project.as_uuid()),
            json!({ "name": name }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    read_json(res).await
}

db_test! {
    async fn post_creates_a_board(pool: PgPool) {
        let project = seed_project(&pool).await;
        let body = create_board(&pool, project, "Backlog").await;
        assert_eq!(body["name"], "Backlog");
        assert_eq!(body["position"], 0);
        assert_eq!(body["projectId"], project.as_uuid().to_string());
    }
}

db_test! {
    async fn post_under_a_missing_project_is_404(pool: PgPool) {
        let res = app(&pool)
            .oneshot(json_request(
                Method::POST,
                &format!("/api/projects/{}/boards", Uuid::now_v7()),
                json!({ "name": "x" }),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
        assert_eq!(read_json(res).await["error"]["code"], "not_found");
    }
}

db_test! {
    async fn post_with_a_blank_name_is_400(pool: PgPool) {
        let project = seed_project(&pool).await;
        let res = app(&pool)
            .oneshot(json_request(
                Method::POST,
                &format!("/api/projects/{}/boards", project.as_uuid()),
                json!({ "name": "  " }),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }
}

db_test! {
    async fn post_beyond_the_limit_is_409(pool: PgPool) {
        let project = seed_project(&pool).await;
        let repo = PgBoardRepo::new(pool.clone());
        for i in 0..99 {
            let b = board(project, &format!("B{i}"), i);
            assert_eq!(repo.insert_within_limit(&b, 99).await.unwrap(), LimitedInsert::Created);
        }
        let res = app(&pool)
            .oneshot(json_request(
                Method::POST,
                &format!("/api/projects/{}/boards", project.as_uuid()),
                json!({ "name": "overflow" }),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::CONFLICT);
        assert_eq!(read_json(res).await["error"]["code"], "limit_exceeded");
    }
}

db_test! {
    async fn get_lists_boards_for_the_project(pool: PgPool) {
        let project = seed_project(&pool).await;
        create_board(&pool, project, "First").await;
        create_board(&pool, project, "Second").await;

        let res = app(&pool)
            .oneshot(empty_request(Method::GET, &format!("/api/projects/{}/boards", project.as_uuid())))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = read_json(res).await;
        assert_eq!(body.as_array().unwrap().len(), 2);
        assert_eq!(body[0]["name"], "First");
    }
}

db_test! {
    async fn patch_updates_a_board(pool: PgPool) {
        let project = seed_project(&pool).await;
        let id = create_board(&pool, project, "Old").await["id"].as_str().unwrap().to_owned();

        let res = app(&pool)
            .oneshot(json_request(Method::PATCH, &format!("/api/boards/{id}"), json!({ "name": "New" })))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(read_json(res).await["name"], "New");
    }
}

db_test! {
    async fn patch_of_a_missing_board_is_404(pool: PgPool) {
        let res = app(&pool)
            .oneshot(json_request(Method::PATCH, &format!("/api/boards/{}", Uuid::now_v7()), json!({ "name": "X" })))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }
}

db_test! {
    async fn delete_removes_a_board_then_is_404(pool: PgPool) {
        let project = seed_project(&pool).await;
        let id = create_board(&pool, project, "Bye").await["id"].as_str().unwrap().to_owned();

        let res = app(&pool).oneshot(empty_request(Method::DELETE, &format!("/api/boards/{id}"))).await.unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
        let again = app(&pool).oneshot(empty_request(Method::DELETE, &format!("/api/boards/{id}"))).await.unwrap();
        assert_eq!(again.status(), StatusCode::NOT_FOUND);
    }
}

db_test! {
    async fn put_reorder_changes_the_order(pool: PgPool) {
        let project = seed_project(&pool).await;
        let a = create_board(&pool, project, "A").await["id"].as_str().unwrap().to_owned();
        let b = create_board(&pool, project, "B").await["id"].as_str().unwrap().to_owned();

        let res = app(&pool)
            .oneshot(json_request(
                Method::PUT,
                &format!("/api/projects/{}/boards/reorder", project.as_uuid()),
                json!({ "orderedIds": [b, a] }),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let list = read_json(
            app(&pool)
                .oneshot(empty_request(Method::GET, &format!("/api/projects/{}/boards", project.as_uuid())))
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(list[0]["name"], "B");
    }
}

db_test! {
    async fn put_reorder_with_a_bad_id_set_is_422(pool: PgPool) {
        let project = seed_project(&pool).await;
        create_board(&pool, project, "A").await;
        let res = app(&pool)
            .oneshot(json_request(
                Method::PUT,
                &format!("/api/projects/{}/boards/reorder", project.as_uuid()),
                json!({ "orderedIds": [Uuid::now_v7().to_string()] }),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
