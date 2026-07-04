use chrono::{DateTime, Utc};

use crate::domain::ids::ProjectId;
use crate::domain::name::EntityName;
use crate::domain::position::Position;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub id: ProjectId,
    pub name: EntityName,
    pub position: Position,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Project {
    pub fn new(name: EntityName, position: Position) -> Self {
        let now = Utc::now();
        Self {
            id: ProjectId::new(),
            name,
            position,
            created_at: now,
            updated_at: now,
        }
    }
}
