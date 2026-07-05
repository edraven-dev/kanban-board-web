use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::adapters::inbound::http::dto::{
    CreateProjectRequest, ProjectResponse, ReorderRequest, UpdateProjectRequest,
};
use crate::app_state::AppState;
use crate::application::error::ApplicationError;
use crate::domain::ids::ProjectId;

#[utoipa::path(
    get, path = "/projects", tag = "projects",
    responses((status = 200, body = Vec<ProjectResponse>))
)]
pub async fn list(
    State(state): State<AppState>,
) -> Result<Json<Vec<ProjectResponse>>, ApplicationError> {
    let projects = state.projects.list().await?;
    Ok(Json(
        projects.into_iter().map(ProjectResponse::from).collect(),
    ))
}

#[utoipa::path(
    post, path = "/projects", tag = "projects",
    request_body = CreateProjectRequest,
    responses((status = 201, body = ProjectResponse), (status = 400))
)]
pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ProjectResponse>), ApplicationError> {
    let project = state.projects.create(&body.name).await?;
    Ok((StatusCode::CREATED, Json(project.into())))
}

#[utoipa::path(
    patch, path = "/projects/{id}", tag = "projects",
    params(("id" = Uuid, Path, description = "Project id")),
    request_body = UpdateProjectRequest,
    responses((status = 200, body = ProjectResponse), (status = 400), (status = 404))
)]
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateProjectRequest>,
) -> Result<Json<ProjectResponse>, ApplicationError> {
    let project = state
        .projects
        .update(ProjectId::from_uuid(id), &body.name)
        .await?;
    Ok(Json(project.into()))
}

#[utoipa::path(
    delete, path = "/projects/{id}", tag = "projects",
    params(("id" = Uuid, Path, description = "Project id")),
    responses((status = 204), (status = 404))
)]
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApplicationError> {
    state.projects.delete(ProjectId::from_uuid(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put, path = "/projects/reorder", tag = "projects",
    request_body = ReorderRequest,
    responses((status = 204), (status = 422))
)]
pub async fn reorder(
    State(state): State<AppState>,
    Json(body): Json<ReorderRequest>,
) -> Result<StatusCode, ApplicationError> {
    let ids = body
        .ordered_ids
        .into_iter()
        .map(ProjectId::from_uuid)
        .collect();
    state.projects.reorder(ids).await?;
    Ok(StatusCode::NO_CONTENT)
}
