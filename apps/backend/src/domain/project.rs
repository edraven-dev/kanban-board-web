use chrono::{DateTime, Utc};

use crate::domain::ids::ProjectId;
use crate::domain::name::EntityName;
use crate::domain::position::Position;

/// Project aggregate root. Read-only outside the domain; mutate through its methods.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub(in crate::domain) id: ProjectId,
    pub(in crate::domain) name: EntityName,
    pub(in crate::domain) position: Position,
    pub(in crate::domain) created_at: DateTime<Utc>,
    pub(in crate::domain) updated_at: DateTime<Utc>,
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

    /// Rebuilds a project from its persisted state (no new id or timestamps).
    pub fn from_parts(
        id: ProjectId,
        name: EntityName,
        position: Position,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            position,
            created_at,
            updated_at,
        }
    }

    pub fn rename(&mut self, name: EntityName) {
        self.name = name;
        self.updated_at = Utc::now();
    }

    pub fn reposition(&mut self, position: Position) {
        self.position = position;
        self.updated_at = Utc::now();
    }

    pub fn id(&self) -> ProjectId {
        self.id
    }

    pub fn name(&self) -> &EntityName {
        &self.name
    }

    pub fn position(&self) -> Position {
        self.position
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
