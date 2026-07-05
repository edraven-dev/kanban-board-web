use chrono::{DateTime, Utc};

use crate::domain::card::Card;
use crate::domain::ids::{BoardId, ColumnId};
use crate::domain::name::EntityName;
use crate::domain::position::Position;

/// A column within the Board aggregate; owns its ordered cards. Structural
/// changes go through the `Board` root — outside the domain it is read-only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub(in crate::domain) id: ColumnId,
    pub(in crate::domain) board_id: BoardId,
    pub(in crate::domain) name: EntityName,
    pub(in crate::domain) position: Position,
    pub(in crate::domain) created_at: DateTime<Utc>,
    pub(in crate::domain) updated_at: DateTime<Utc>,
    pub(in crate::domain) cards: Vec<Card>,
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
            cards: Vec::new(),
        }
    }

    /// Rebuilds a column (with its cards) from persisted state.
    pub fn from_parts(
        id: ColumnId,
        board_id: BoardId,
        name: EntityName,
        position: Position,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        cards: Vec<Card>,
    ) -> Self {
        Self {
            id,
            board_id,
            name,
            position,
            created_at,
            updated_at,
            cards,
        }
    }

    pub fn id(&self) -> ColumnId {
        self.id
    }

    pub fn board_id(&self) -> BoardId {
        self.board_id
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

    /// Attaches this column's cards during aggregate reconstitution.
    pub fn with_cards(mut self, cards: Vec<Card>) -> Self {
        self.cards = cards;
        self
    }

    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    pub fn into_cards(self) -> Vec<Card> {
        self.cards
    }
}
