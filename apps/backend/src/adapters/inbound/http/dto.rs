use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::application::board_view_service::{BoardView, ColumnView};
use crate::domain::board::Board;
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
            id: project.id.as_uuid(),
            name: project.name.into_string(),
            position: project.position.value(),
            created_at: project.created_at,
            updated_at: project.updated_at,
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

impl From<Board> for BoardResponse {
    fn from(board: Board) -> Self {
        Self {
            id: board.id.as_uuid(),
            project_id: board.project_id.as_uuid(),
            name: board.name.into_string(),
            position: board.position.value(),
            created_at: board.created_at,
            updated_at: board.updated_at,
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
            id: column.id.as_uuid(),
            board_id: column.board_id.as_uuid(),
            name: column.name.into_string(),
            position: column.position.value(),
            created_at: column.created_at,
            updated_at: column.updated_at,
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
            id: card.id.as_uuid(),
            column_id: card.column_id.as_uuid(),
            title: card.title.into_string(),
            description: card.description.into_string(),
            position: card.position.value(),
            created_at: card.created_at,
            updated_at: card.updated_at,
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

impl From<BoardView> for BoardFullResponse {
    fn from(view: BoardView) -> Self {
        Self {
            id: view.board.id.as_uuid(),
            project_id: view.board.project_id.as_uuid(),
            name: view.board.name.into_string(),
            position: view.board.position.value(),
            created_at: view.board.created_at,
            updated_at: view.board.updated_at,
            columns: view.columns.into_iter().map(ColumnFullResponse::from).collect(),
        }
    }
}

impl From<ColumnView> for ColumnFullResponse {
    fn from(view: ColumnView) -> Self {
        Self {
            id: view.column.id.as_uuid(),
            board_id: view.column.board_id.as_uuid(),
            name: view.column.name.into_string(),
            position: view.column.position.value(),
            created_at: view.column.created_at,
            updated_at: view.column.updated_at,
            cards: view.cards.into_iter().map(CardResponse::from).collect(),
        }
    }
}
