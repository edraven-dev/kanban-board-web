use std::collections::HashMap;
use std::sync::Arc;

use crate::application::error::ApplicationError;
use crate::domain::board::Board;
use crate::domain::card::Card;
use crate::domain::column::Column;
use crate::domain::ids::BoardId;
use crate::domain::ports::{BoardRepository, CardRepository, ColumnRepository};

pub struct BoardView {
    pub board: Board,
    pub columns: Vec<ColumnView>,
}

pub struct ColumnView {
    pub column: Column,
    pub cards: Vec<Card>,
}

#[derive(Clone)]
pub struct BoardViewService {
    boards: Arc<dyn BoardRepository>,
    columns: Arc<dyn ColumnRepository>,
    cards: Arc<dyn CardRepository>,
}

impl BoardViewService {
    pub fn new(
        boards: Arc<dyn BoardRepository>,
        columns: Arc<dyn ColumnRepository>,
        cards: Arc<dyn CardRepository>,
    ) -> Self {
        Self {
            boards,
            columns,
            cards,
        }
    }

    pub async fn get_full(&self, board_id: BoardId) -> Result<Option<BoardView>, ApplicationError> {
        let Some(board) = self.boards.get(board_id).await? else {
            return Ok(None);
        };
        let columns = self.columns.list_by_board(board_id).await?;
        let column_ids: Vec<_> = columns.iter().map(|c| c.id).collect();
        let cards = self.cards.list_by_columns(&column_ids).await?;

        let mut by_column: HashMap<_, Vec<Card>> = HashMap::new();
        for card in cards {
            by_column.entry(card.column_id).or_default().push(card);
        }

        let columns = columns
            .into_iter()
            .map(|column| {
                let cards = by_column.remove(&column.id).unwrap_or_default();
                ColumnView { column, cards }
            })
            .collect();

        Ok(Some(BoardView { board, columns }))
    }
}
