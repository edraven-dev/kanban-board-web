mod common;

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use axum::response::Response;
use backend::adapters::outbound::persistence::pg_project_repo::PgProjectRepo;
use backend::app_state::AppState;
use backend::domain::ids::ProjectId;
use backend::domain::name::EntityName;
use backend::domain::ports::ProjectRepository;
use backend::domain::position::Position;
use backend::domain::project::Project;
use backend::infrastructure::config::AppEnv;
use backend::router;
use common::db_test;
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

fn project(name: &str, position: i32) -> Project {
    Project::new(
        EntityName::new(name).unwrap(),
        Position::new(position).unwrap(),
    )
}

// ---------------------------------------------------------------------------
// Repository tests (sqlx against Postgres)
// ---------------------------------------------------------------------------

db_test! {
    async fn insert_and_get_round_trip(pool: PgPool) {
        let repo = PgProjectRepo::new(pool);
        let p = project("Roadmap", 0);
        repo.insert(&p).await.unwrap();

        let fetched = repo.get(p.id).await.unwrap().unwrap();
        assert_eq!(fetched.id, p.id);
        assert_eq!(fetched.name.as_str(), "Roadmap");
        assert_eq!(fetched.position.value(), 0);
    }
}

db_test! {
    async fn get_of_a_missing_project_is_none(pool: PgPool) {
        let repo = PgProjectRepo::new(pool);
        assert!(repo.get(ProjectId::new()).await.unwrap().is_none());
    }
}

db_test! {
    async fn list_orders_by_position(pool: PgPool) {
        let repo = PgProjectRepo::new(pool);
        for p in [&project("A", 2), &project("B", 0), &project("C", 1)] {
            repo.insert(p).await.unwrap();
        }
        let names: Vec<String> = repo
            .list()
            .await
            .unwrap()
            .iter()
            .map(|p| p.name.as_str().to_owned())
            .collect();
        assert_eq!(names, ["B", "C", "A"]);
    }
}

db_test! {
    async fn update_changes_the_name(pool: PgPool) {
        let repo = PgProjectRepo::new(pool);
        let p = project("Old", 0);
        repo.insert(&p).await.unwrap();
        repo.update(p.id, EntityName::new("New").unwrap()).await.unwrap();
        assert_eq!(repo.get(p.id).await.unwrap().unwrap().name.as_str(), "New");
    }
}

db_test! {
    async fn delete_removes_the_row(pool: PgPool) {
        let repo = PgProjectRepo::new(pool);
        let p = project("Gone", 0);
        repo.insert(&p).await.unwrap();
        repo.delete(p.id).await.unwrap();
        assert!(repo.get(p.id).await.unwrap().is_none());
    }
}

db_test! {
    async fn reorder_rewrites_positions_transactionally(pool: PgPool) {
        let repo = PgProjectRepo::new(pool);
        let (a, b, c) = (project("A", 0), project("B", 1), project("C", 2));
        for p in [&a, &b, &c] {
            repo.insert(p).await.unwrap();
        }
        repo.reorder(&[c.id, a.id, b.id]).await.unwrap();

        let ordered: Vec<ProjectId> = repo.list().await.unwrap().iter().map(|p| p.id).collect();
        assert_eq!(ordered, [c.id, a.id, b.id]);
    }
}

db_test! {
    async fn a_driver_error_becomes_a_repository_error(pool: PgPool) {
        let repo = PgProjectRepo::new(pool);
        let p = project("Dup", 0);
        repo.insert(&p).await.unwrap();
        // Re-inserting the same id violates the primary key.
        let err = repo.insert(&p).await.unwrap_err();
        assert!(err.to_string().contains("repository error"));
    }
}

db_test! {
    async fn a_row_violating_the_name_rule_fails_to_map(pool: PgPool) {
        sqlx::query("INSERT INTO projects (id, name, position) VALUES ($1, '', 0)")
            .bind(Uuid::now_v7())
            .execute(&pool)
            .await
            .unwrap();
        let repo = PgProjectRepo::new(pool);
        assert!(repo.list().await.is_err());
    }
}

db_test! {
    async fn a_row_violating_the_position_rule_fails_to_map(pool: PgPool) {
        let id = Uuid::now_v7();
        sqlx::query("INSERT INTO projects (id, name, position) VALUES ($1, 'ok', -1)")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
        let repo = PgProjectRepo::new(pool);
        assert!(repo.get(ProjectId::from_uuid(id)).await.is_err());
    }
}

// ---------------------------------------------------------------------------
// HTTP handler tests (full stack via oneshot)
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

async fn create(pool: &sqlx::PgPool, name: &str) -> Value {
    let res = app(pool)
        .oneshot(json_request(Method::POST, "/api/projects", json!({ "name": name })))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    read_json(res).await
}

db_test! {
    async fn post_creates_a_project(pool: PgPool) {
        let body = create(&pool, "Launch").await;
        assert_eq!(body["name"], "Launch");
        assert_eq!(body["position"], 0);
        assert!(body["id"].is_string());
        assert!(body["createdAt"].is_string());
        assert!(body["updatedAt"].is_string());
    }
}

db_test! {
    async fn post_with_a_blank_name_is_400(pool: PgPool) {
        let res = app(&pool)
            .oneshot(json_request(Method::POST, "/api/projects", json!({ "name": "   " })))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        assert_eq!(read_json(res).await["error"]["code"], "validation");
    }
}

db_test! {
    async fn get_lists_projects_in_order(pool: PgPool) {
        create(&pool, "First").await;
        create(&pool, "Second").await;

        let res = app(&pool).oneshot(empty_request(Method::GET, "/api/projects")).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = read_json(res).await;
        assert_eq!(body.as_array().unwrap().len(), 2);
        assert_eq!(body[0]["name"], "First");
        assert_eq!(body[1]["name"], "Second");
    }
}

db_test! {
    async fn patch_updates_a_project(pool: PgPool) {
        let id = create(&pool, "Old").await["id"].as_str().unwrap().to_owned();
        let res = app(&pool)
            .oneshot(json_request(Method::PATCH, &format!("/api/projects/{id}"), json!({ "name": "New" })))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(read_json(res).await["name"], "New");
    }
}

db_test! {
    async fn patch_of_a_missing_project_is_404(pool: PgPool) {
        let res = app(&pool)
            .oneshot(json_request(
                Method::PATCH,
                &format!("/api/projects/{}", Uuid::now_v7()),
                json!({ "name": "X" }),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
        assert_eq!(read_json(res).await["error"]["code"], "not_found");
    }
}

db_test! {
    async fn delete_removes_a_project_and_is_404_the_second_time(pool: PgPool) {
        let id = create(&pool, "Bye").await["id"].as_str().unwrap().to_owned();

        let res = app(&pool).oneshot(empty_request(Method::DELETE, &format!("/api/projects/{id}"))).await.unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let again = app(&pool).oneshot(empty_request(Method::DELETE, &format!("/api/projects/{id}"))).await.unwrap();
        assert_eq!(again.status(), StatusCode::NOT_FOUND);
    }
}

db_test! {
    async fn put_reorder_changes_the_order(pool: PgPool) {
        let a = create(&pool, "A").await["id"].as_str().unwrap().to_owned();
        let b = create(&pool, "B").await["id"].as_str().unwrap().to_owned();

        let res = app(&pool)
            .oneshot(json_request(Method::PUT, "/api/projects/reorder", json!({ "orderedIds": [b, a] })))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let list = read_json(app(&pool).oneshot(empty_request(Method::GET, "/api/projects")).await.unwrap()).await;
        assert_eq!(list[0]["name"], "B");
        assert_eq!(list[1]["name"], "A");
    }
}

db_test! {
    async fn put_reorder_with_a_bad_id_set_is_422(pool: PgPool) {
        create(&pool, "A").await;
        let res = app(&pool)
            .oneshot(json_request(
                Method::PUT,
                "/api/projects/reorder",
                json!({ "orderedIds": [Uuid::now_v7().to_string()] }),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(read_json(res).await["error"]["code"], "unprocessable");
    }
}
