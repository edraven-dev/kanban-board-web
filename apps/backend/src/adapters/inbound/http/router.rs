use axum::Json;
use axum::Router;
use serde::Serialize;
use tower_http::cors::CorsLayer;
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

use crate::adapters::inbound::http::{boards, projects};
use crate::app_state::AppState;
use crate::infrastructure::config::AppEnv;

#[derive(OpenApi)]
#[openapi(info(title = "Kanban API", version = "0.1.0"))]
struct ApiDoc;

pub fn router(app_env: AppEnv, state: AppState) -> Router {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api", api_routes())
        .split_for_parts();

    let mut router = router
        .merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", api))
        .with_state(state);

    if app_env.is_development() {
        router = router.layer(CorsLayer::permissive());
    }

    router
}

fn api_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(health))
        .routes(routes!(projects::list, projects::create))
        .routes(routes!(projects::reorder))
        .routes(routes!(projects::update, projects::delete))
        .routes(routes!(boards::list, boards::create))
        .routes(routes!(boards::reorder))
        .routes(routes!(boards::update, boards::delete))
}

#[derive(Serialize, ToSchema)]
struct Health {
    status: &'static str,
}

#[utoipa::path(get, path = "/health", responses((status = 200, body = Health)))]
async fn health() -> Json<Health> {
    Json(Health { status: "ok" })
}
