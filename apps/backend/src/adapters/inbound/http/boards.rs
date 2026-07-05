use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::adapters::inbound::http::dto::{
    BoardFullResponse, BoardResponse, CreateBoardRequest, ReorderRequest, UpdateBoardRequest,
};
use crate::app_state::AppState;
use crate::application::error::ApplicationError;
use crate::domain::ids::{BoardId, ProjectId};

#[utoipa::path(
    get, path = "/projects/{id}/boards", tag = "boards",
    params(("id" = Uuid, Path, description = "Project id")),
    responses((status = 200, body = Vec<BoardResponse>))
)]
pub async fn list(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<BoardResponse>>, ApplicationError> {
    let boards = state.boards.list(ProjectId::from_uuid(project_id)).await?;
    Ok(Json(boards.into_iter().map(BoardResponse::from).collect()))
}

#[utoipa::path(
    post, path = "/projects/{id}/boards", tag = "boards",
    params(("id" = Uuid, Path, description = "Project id")),
    request_body = CreateBoardRequest,
    responses(
        (status = 201, body = BoardResponse),
        (status = 400), (status = 404), (status = 409)
    )
)]
pub async fn create(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(body): Json<CreateBoardRequest>,
) -> Result<(StatusCode, Json<BoardResponse>), ApplicationError> {
    let board = state
        .boards
        .create(ProjectId::from_uuid(project_id), &body.name)
        .await?;
    Ok((StatusCode::CREATED, Json(board.into())))
}

#[utoipa::path(
    get, path = "/boards/{id}/full", tag = "boards",
    params(("id" = Uuid, Path, description = "Board id")),
    responses((status = 200, body = BoardFullResponse), (status = 404))
)]
pub async fn full(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<BoardFullResponse>, ApplicationError> {
    let board = state
        .boards
        .get_full(BoardId::from_uuid(id))
        .await?
        .ok_or(ApplicationError::NotFound)?;
    Ok(Json(board.into()))
}

#[utoipa::path(
    patch, path = "/boards/{id}", tag = "boards",
    params(("id" = Uuid, Path, description = "Board id")),
    request_body = UpdateBoardRequest,
    responses((status = 200, body = BoardResponse), (status = 400), (status = 404))
)]
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateBoardRequest>,
) -> Result<Json<BoardResponse>, ApplicationError> {
    let board = state
        .boards
        .update(BoardId::from_uuid(id), &body.name)
        .await?;
    Ok(Json(board.into()))
}

#[utoipa::path(
    delete, path = "/boards/{id}", tag = "boards",
    params(("id" = Uuid, Path, description = "Board id")),
    responses((status = 204), (status = 404))
)]
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApplicationError> {
    state.boards.delete(BoardId::from_uuid(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put, path = "/projects/{id}/boards/reorder", tag = "boards",
    params(("id" = Uuid, Path, description = "Project id")),
    request_body = ReorderRequest,
    responses((status = 204), (status = 422))
)]
pub async fn reorder(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(body): Json<ReorderRequest>,
) -> Result<StatusCode, ApplicationError> {
    let ids = body
        .ordered_ids
        .into_iter()
        .map(BoardId::from_uuid)
        .collect();
    state
        .boards
        .reorder(ProjectId::from_uuid(project_id), ids)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
