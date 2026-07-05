use chrono::{DateTime, Utc};

use crate::domain::description::Description;
use crate::domain::ids::{CardId, ColumnId};
use crate::domain::position::Position;
use crate::domain::title::Title;

/// A card within the Board aggregate. Mutated only by the `Board` root; the
/// domain-visible fields let the root re-sequence and edit it, while outside the
/// domain it is read-only through its accessors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub(in crate::domain) id: CardId,
    pub(in crate::domain) column_id: ColumnId,
    pub(in crate::domain) title: Title,
    pub(in crate::domain) description: Description,
    pub(in crate::domain) position: Position,
    pub(in crate::domain) created_at: DateTime<Utc>,
    pub(in crate::domain) updated_at: DateTime<Utc>,
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

    /// Rebuilds a card from its persisted state (no new id or timestamps).
    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        id: CardId,
        column_id: ColumnId,
        title: Title,
        description: Description,
        position: Position,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            column_id,
            title,
            description,
            position,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> CardId {
        self.id
    }

    pub fn column_id(&self) -> ColumnId {
        self.column_id
    }

    pub fn title(&self) -> &Title {
        &self.title
    }

    pub fn description(&self) -> &Description {
        &self.description
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
