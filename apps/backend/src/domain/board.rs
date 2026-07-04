use chrono::{DateTime, Utc};

use crate::domain::ids::{BoardId, ProjectId};
use crate::domain::name::EntityName;
use crate::domain::position::Position;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    pub id: BoardId,
    pub project_id: ProjectId,
    pub name: EntityName,
    pub position: Position,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Board {
    pub fn new(project_id: ProjectId, name: EntityName, position: Position) -> Self {
        let now = Utc::now();
        Self {
            id: BoardId::new(),
            project_id,
            name,
            position,
            created_at: now,
            updated_at: now,
        }
    }
}
