use axum::Router;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

use crate::infrastructure::config::AppEnv;

#[derive(OpenApi)]
#[openapi(info(title = "Kanban API", version = "0.1.0"))]
struct ApiDoc;

pub fn router(app_env: AppEnv) -> Router {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api", OpenApiRouter::new().routes(routes!(health)))
        .split_for_parts();

    let mut router = router.merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", api));

    if app_env.is_development() {
        router = router.layer(CorsLayer::permissive());
    }

    router
}

#[utoipa::path(get, path = "/health", responses((status = 200, description = "Service is healthy", body = str)))]
async fn health() -> &'static str {
    "ok"
}
