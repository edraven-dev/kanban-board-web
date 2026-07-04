use chrono::{DateTime, Utc};

use crate::domain::description::Description;
use crate::domain::ids::{CardId, ColumnId};
use crate::domain::position::Position;
use crate::domain::title::Title;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub id: CardId,
    pub column_id: ColumnId,
    pub title: Title,
    pub description: Description,
    pub position: Position,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Card {
    pub fn new(
        column_id: ColumnId,
        title: Title,
        description: Description,
        position: Position,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: CardId::new(),
            column_id,
            title,
            description,
            position,
            created_at: now,
            updated_at: now,
        }
    }
}
