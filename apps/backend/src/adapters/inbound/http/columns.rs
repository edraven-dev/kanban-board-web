use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::adapters::inbound::http::dto::{
    ColumnResponse, CreateColumnRequest, ReorderRequest, UpdateColumnRequest,
};
use crate::app_state::AppState;
use crate::application::error::ApplicationError;
use crate::domain::ids::{BoardId, ColumnId};

#[utoipa::path(
    get, path = "/boards/{id}/columns", tag = "columns",
    params(("id" = Uuid, Path, description = "Board id")),
    responses((status = 200, body = Vec<ColumnResponse>))
)]
pub async fn list(
    State(state): State<AppState>,
    Path(board_id): Path<Uuid>,
) -> Result<Json<Vec<ColumnResponse>>, ApplicationError> {
    let columns = state.columns.list(BoardId::from_uuid(board_id)).await?;
    Ok(Json(columns.into_iter().map(ColumnResponse::from).collect()))
}

#[utoipa::path(
    post, path = "/boards/{id}/columns", tag = "columns",
    params(("id" = Uuid, Path, description = "Board id")),
    request_body = CreateColumnRequest,
    responses(
        (status = 201, body = ColumnResponse),
        (status = 400), (status = 404), (status = 409)
    )
)]
pub async fn create(
    State(state): State<AppState>,
    Path(board_id): Path<Uuid>,
    Json(body): Json<CreateColumnRequest>,
) -> Result<(StatusCode, Json<ColumnResponse>), ApplicationError> {
    let column = state
        .columns
        .create(BoardId::from_uuid(board_id), &body.name)
        .await?;
    Ok((StatusCode::CREATED, Json(column.into())))
}

#[utoipa::path(
    patch, path = "/columns/{id}", tag = "columns",
    params(("id" = Uuid, Path, description = "Column id")),
    request_body = UpdateColumnRequest,
    responses((status = 200, body = ColumnResponse), (status = 400), (status = 404))
)]
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateColumnRequest>,
) -> Result<Json<ColumnResponse>, ApplicationError> {
    let column = state.columns.update(ColumnId::from_uuid(id), &body.name).await?;
    Ok(Json(column.into()))
}

#[utoipa::path(
    delete, path = "/columns/{id}", tag = "columns",
    params(("id" = Uuid, Path, description = "Column id")),
    responses((status = 204), (status = 404))
)]
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApplicationError> {
    state.columns.delete(ColumnId::from_uuid(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put, path = "/boards/{id}/columns/reorder", tag = "columns",
    params(("id" = Uuid, Path, description = "Board id")),
    request_body = ReorderRequest,
    responses((status = 204), (status = 422))
)]
pub async fn reorder(
    State(state): State<AppState>,
    Path(board_id): Path<Uuid>,
    Json(body): Json<ReorderRequest>,
) -> Result<StatusCode, ApplicationError> {
    let ids = body.ordered_ids.into_iter().map(ColumnId::from_uuid).collect();
    state
        .columns
        .reorder(BoardId::from_uuid(board_id), ids)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
