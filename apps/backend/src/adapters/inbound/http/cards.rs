use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::adapters::inbound::http::dto::{
    CardResponse, CreateCardRequest, MoveCardRequest, UpdateCardRequest,
};
use crate::app_state::AppState;
use crate::application::error::ApplicationError;
use crate::domain::ids::{CardId, ColumnId};

#[utoipa::path(
    get, path = "/columns/{id}/cards", tag = "cards",
    params(("id" = Uuid, Path, description = "Column id")),
    responses((status = 200, body = Vec<CardResponse>))
)]
pub async fn list(
    State(state): State<AppState>,
    Path(column_id): Path<Uuid>,
) -> Result<Json<Vec<CardResponse>>, ApplicationError> {
    let cards = state
        .boards
        .list_cards(ColumnId::from_uuid(column_id))
        .await?;
    Ok(Json(cards.into_iter().map(CardResponse::from).collect()))
}

#[utoipa::path(
    post, path = "/columns/{id}/cards", tag = "cards",
    params(("id" = Uuid, Path, description = "Column id")),
    request_body = CreateCardRequest,
    responses((status = 201, body = CardResponse), (status = 400), (status = 404))
)]
pub async fn create(
    State(state): State<AppState>,
    Path(column_id): Path<Uuid>,
    Json(body): Json<CreateCardRequest>,
) -> Result<(StatusCode, Json<CardResponse>), ApplicationError> {
    let card = state
        .boards
        .create_card(
            ColumnId::from_uuid(column_id),
            &body.title,
            body.description.as_deref(),
        )
        .await?;
    Ok((StatusCode::CREATED, Json(card.into())))
}

#[utoipa::path(
    get, path = "/cards/{id}", tag = "cards",
    params(("id" = Uuid, Path, description = "Card id")),
    responses((status = 200, body = CardResponse), (status = 404))
)]
pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<CardResponse>, ApplicationError> {
    let card = state.boards.get_card(CardId::from_uuid(id)).await?;
    Ok(Json(card.into()))
}

#[utoipa::path(
    patch, path = "/cards/{id}", tag = "cards",
    params(("id" = Uuid, Path, description = "Card id")),
    request_body = UpdateCardRequest,
    responses((status = 200, body = CardResponse), (status = 400), (status = 404))
)]
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateCardRequest>,
) -> Result<Json<CardResponse>, ApplicationError> {
    let card = state
        .boards
        .update_card(
            CardId::from_uuid(id),
            body.title.as_deref(),
            body.description.as_deref(),
        )
        .await?;
    Ok(Json(card.into()))
}

#[utoipa::path(
    delete, path = "/cards/{id}", tag = "cards",
    params(("id" = Uuid, Path, description = "Card id")),
    responses((status = 204), (status = 404))
)]
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApplicationError> {
    state.boards.delete_card(CardId::from_uuid(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put, path = "/cards/{id}/move", tag = "cards",
    params(("id" = Uuid, Path, description = "Card id")),
    request_body = MoveCardRequest,
    responses((status = 204), (status = 400), (status = 404))
)]
pub async fn move_card(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<MoveCardRequest>,
) -> Result<StatusCode, ApplicationError> {
    state
        .boards
        .move_card(
            CardId::from_uuid(id),
            ColumnId::from_uuid(body.column_id),
            body.position,
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
