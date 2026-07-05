use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::board::BoardSummary;
use crate::domain::card::Card;
use crate::domain::column::Column;
use crate::domain::description::Description;
use crate::domain::ids::{BoardId, CardId, ColumnId, ProjectId};
use crate::domain::name::EntityName;
use crate::domain::ports::RepositoryError;
use crate::domain::position::Position;
use crate::domain::project::Project;
use crate::domain::title::Title;

pub struct ProjectRecord {
    pub id: Uuid,
    pub name: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<ProjectRecord> for Project {
    type Error = RepositoryError;

    fn try_from(record: ProjectRecord) -> Result<Self, Self::Error> {
        Ok(Project {
            id: ProjectId::from_uuid(record.id),
            name: EntityName::new(record.name).map_err(|e| RepositoryError::new(e.to_string()))?,
            position: Position::new(record.position)
                .map_err(|e| RepositoryError::new(e.to_string()))?,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }
}

pub struct BoardRecord {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<BoardRecord> for BoardSummary {
    type Error = RepositoryError;

    fn try_from(record: BoardRecord) -> Result<Self, Self::Error> {
        Ok(BoardSummary {
            id: BoardId::from_uuid(record.id),
            project_id: ProjectId::from_uuid(record.project_id),
            name: EntityName::new(record.name).map_err(|e| RepositoryError::new(e.to_string()))?,
            position: Position::new(record.position)
                .map_err(|e| RepositoryError::new(e.to_string()))?,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }
}

pub struct ColumnRecord {
    pub id: Uuid,
    pub board_id: Uuid,
    pub name: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<ColumnRecord> for Column {
    type Error = RepositoryError;

    fn try_from(record: ColumnRecord) -> Result<Self, Self::Error> {
        Ok(Column {
            id: ColumnId::from_uuid(record.id),
            board_id: BoardId::from_uuid(record.board_id),
            name: EntityName::new(record.name).map_err(|e| RepositoryError::new(e.to_string()))?,
            position: Position::new(record.position)
                .map_err(|e| RepositoryError::new(e.to_string()))?,
            created_at: record.created_at,
            updated_at: record.updated_at,
            cards: Vec::new(),
        })
    }
}

pub struct CardRecord {
    pub id: Uuid,
    pub column_id: Uuid,
    pub title: String,
    pub description: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<CardRecord> for Card {
    type Error = RepositoryError;

    fn try_from(record: CardRecord) -> Result<Self, Self::Error> {
        Ok(Card {
            id: CardId::from_uuid(record.id),
            column_id: ColumnId::from_uuid(record.column_id),
            title: Title::new(record.title).map_err(|e| RepositoryError::new(e.to_string()))?,
            description: Description::new(record.description)
                .map_err(|e| RepositoryError::new(e.to_string()))?,
            position: Position::new(record.position)
                .map_err(|e| RepositoryError::new(e.to_string()))?,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }
}
