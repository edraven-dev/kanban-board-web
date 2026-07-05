use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::board::{Board, BoardSummary};
use crate::domain::card::Card;
use crate::domain::column::Column;
use crate::domain::project::Project;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProjectResponse {
    pub id: Uuid,
    pub name: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Project> for ProjectResponse {
    fn from(project: Project) -> Self {
        Self {
            id: project.id().as_uuid(),
            name: project.name().as_str().to_owned(),
            position: project.position().value(),
            created_at: project.created_at(),
            updated_at: project.updated_at(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateProjectRequest {
    pub name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProjectRequest {
    pub name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReorderRequest {
    pub ordered_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BoardResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<BoardSummary> for BoardResponse {
    fn from(board: BoardSummary) -> Self {
        Self {
            id: board.id().as_uuid(),
            project_id: board.project_id().as_uuid(),
            name: board.name().as_str().to_owned(),
            position: board.position().value(),
            created_at: board.created_at(),
            updated_at: board.updated_at(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateBoardRequest {
    pub name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateBoardRequest {
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ColumnResponse {
    pub id: Uuid,
    pub board_id: Uuid,
    pub name: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Column> for ColumnResponse {
    fn from(column: Column) -> Self {
        Self {
            id: column.id().as_uuid(),
            board_id: column.board_id().as_uuid(),
            name: column.name().as_str().to_owned(),
            position: column.position().value(),
            created_at: column.created_at(),
            updated_at: column.updated_at(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateColumnRequest {
    pub name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateColumnRequest {
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CardResponse {
    pub id: Uuid,
    pub column_id: Uuid,
    pub title: String,
    pub description: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Card> for CardResponse {
    fn from(card: Card) -> Self {
        Self {
            id: card.id().as_uuid(),
            column_id: card.column_id().as_uuid(),
            title: card.title().as_str().to_owned(),
            description: card.description().as_str().to_owned(),
            position: card.position().value(),
            created_at: card.created_at(),
            updated_at: card.updated_at(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCardRequest {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateCardRequest {
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MoveCardRequest {
    pub column_id: Uuid,
    pub position: i32,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BoardFullResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub columns: Vec<ColumnFullResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ColumnFullResponse {
    pub id: Uuid,
    pub board_id: Uuid,
    pub name: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub cards: Vec<CardResponse>,
}

impl From<Board> for BoardFullResponse {
    fn from(board: Board) -> Self {
        Self {
            id: board.id().as_uuid(),
            project_id: board.project_id().as_uuid(),
            name: board.name().as_str().to_owned(),
            position: board.position().value(),
            created_at: board.created_at(),
            updated_at: board.updated_at(),
            columns: board
                .into_columns()
                .into_iter()
                .map(ColumnFullResponse::from)
                .collect(),
        }
    }
}

impl From<Column> for ColumnFullResponse {
    fn from(column: Column) -> Self {
        Self {
            id: column.id().as_uuid(),
            board_id: column.board_id().as_uuid(),
            name: column.name().as_str().to_owned(),
            position: column.position().value(),
            created_at: column.created_at(),
            updated_at: column.updated_at(),
            cards: column
                .into_cards()
                .into_iter()
                .map(CardResponse::from)
                .collect(),
        }
    }
}
