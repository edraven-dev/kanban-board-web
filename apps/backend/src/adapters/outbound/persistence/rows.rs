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

pub struct ProjectRow {
    pub id: Uuid,
    pub name: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<ProjectRow> for Project {
    type Error = RepositoryError;

    fn try_from(row: ProjectRow) -> Result<Self, Self::Error> {
        Ok(Project::from_parts(
            ProjectId::from_uuid(row.id),
            EntityName::new(row.name)?,
            Position::new(row.position)?,
            row.created_at,
            row.updated_at,
        ))
    }
}

pub struct BoardRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<BoardRow> for BoardSummary {
    type Error = RepositoryError;

    fn try_from(row: BoardRow) -> Result<Self, Self::Error> {
        Ok(BoardSummary::from_parts(
            BoardId::from_uuid(row.id),
            ProjectId::from_uuid(row.project_id),
            EntityName::new(row.name)?,
            Position::new(row.position)?,
            row.created_at,
            row.updated_at,
        ))
    }
}

pub struct ColumnRow {
    pub id: Uuid,
    pub board_id: Uuid,
    pub name: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<ColumnRow> for Column {
    type Error = RepositoryError;

    fn try_from(row: ColumnRow) -> Result<Self, Self::Error> {
        Ok(Column::from_parts(
            ColumnId::from_uuid(row.id),
            BoardId::from_uuid(row.board_id),
            EntityName::new(row.name)?,
            Position::new(row.position)?,
            row.created_at,
            row.updated_at,
            Vec::new(),
        ))
    }
}

pub struct CardRow {
    pub id: Uuid,
    pub column_id: Uuid,
    pub title: String,
    pub description: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<CardRow> for Card {
    type Error = RepositoryError;

    fn try_from(row: CardRow) -> Result<Self, Self::Error> {
        Ok(Card::from_parts(
            CardId::from_uuid(row.id),
            ColumnId::from_uuid(row.column_id),
            Title::new(row.title)?,
            Description::new(row.description)?,
            Position::new(row.position)?,
            row.created_at,
            row.updated_at,
        ))
    }
}
