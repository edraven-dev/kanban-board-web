use chrono::{DateTime, Utc};

use crate::domain::ids::{BoardId, ColumnId};
use crate::domain::name::EntityName;
use crate::domain::position::Position;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub id: ColumnId,
    pub board_id: BoardId,
    pub name: EntityName,
    pub position: Position,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Column {
    pub fn new(board_id: BoardId, name: EntityName, position: Position) -> Self {
        let now = Utc::now();
        Self {
            id: ColumnId::new(),
            board_id,
            name,
            position,
            created_at: now,
            updated_at: now,
        }
    }
}
