use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::board::Board;
use crate::domain::ids::{BoardId, ProjectId};
use crate::domain::name::EntityName;
use crate::domain::ports::RepositoryError;
use crate::domain::position::Position;
use crate::domain::project::Project;

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

impl TryFrom<BoardRecord> for Board {
    type Error = RepositoryError;

    fn try_from(record: BoardRecord) -> Result<Self, Self::Error> {
        Ok(Board {
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
