use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use backend::app_state::AppState;
use backend::infrastructure::config::AppEnv;
use backend::router;
use sqlx::PgPool;
use tower::ServiceExt;

fn test_state() -> AppState {
    AppState::new(PgPool::connect_lazy("postgres://kanban:kanban@localhost:5432/kanban").unwrap())
}

fn get(uri: &str, origin: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().method(Method::GET).uri(uri);
    if let Some(origin) = origin {
        builder = builder.header(header::ORIGIN, origin);
    }
    builder.body(Body::empty()).unwrap()
}

#[tokio::test]
async fn health_returns_ok_json_under_the_api_prefix() {
    let res = router(AppEnv::Development, test_state())
        .oneshot(get("/api/health", None))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn root_health_is_not_found() {
    let res = router(AppEnv::Development, test_state())
        .oneshot(get("/health", None))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn development_applies_cors() {
    let res = router(AppEnv::Development, test_state())
        .oneshot(get("/api/health", Some("http://localhost:3000")))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(
        res.headers()
            .contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN),
        "development must apply a CORS layer"
    );
}

#[tokio::test]
async fn openapi_spec_is_served_and_documents_health() {
    let res = router(AppEnv::Development, test_state())
        .oneshot(get("/api/openapi.json", None))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let spec: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(
        spec["paths"]["/api/health"].is_object(),
        "OpenAPI spec must document /api/health"
    );
    assert!(
        spec["paths"]["/api/projects"].is_object(),
        "OpenAPI spec must document /api/projects"
    );
}

#[tokio::test]
async fn production_emits_no_cors_header() {
    let res = router(AppEnv::Production, test_state())
        .oneshot(get("/api/health", Some("http://localhost:3000")))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(
        res.headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_none(),
        "production must not emit CORS headers"
    );
}
