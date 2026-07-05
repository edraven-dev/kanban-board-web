use std::collections::HashSet;

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::domain::card::Card;
use crate::domain::column::Column;
use crate::domain::description::Description;
use crate::domain::ids::{BoardId, CardId, ColumnId, ProjectId};
use crate::domain::name::EntityName;
use crate::domain::position::Position;
use crate::domain::title::Title;

pub const MAX_COLUMNS_PER_BOARD: usize = 99;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum BoardError {
    #[error("column not found in this board")]
    ColumnNotFound,
    #[error("card not found in this board")]
    CardNotFound,
    #[error("target column not found in this board")]
    TargetColumnNotFound,
    #[error("column limit reached")]
    ColumnLimitReached,
    #[error("orderedIds must be a permutation of the current column ids")]
    NotAPermutation,
}

/// A board's own fields without its columns/cards — the read model for lists and reorder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardSummary {
    pub(in crate::domain) id: BoardId,
    pub(in crate::domain) project_id: ProjectId,
    pub(in crate::domain) name: EntityName,
    pub(in crate::domain) position: Position,
    pub(in crate::domain) created_at: DateTime<Utc>,
    pub(in crate::domain) updated_at: DateTime<Utc>,
}

impl BoardSummary {
    /// Rebuilds a board summary from its persisted row.
    pub fn from_parts(
        id: BoardId,
        project_id: ProjectId,
        name: EntityName,
        position: Position,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            project_id,
            name,
            position,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> BoardId {
        self.id
    }

    pub fn project_id(&self) -> ProjectId {
        self.project_id
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

/// Aggregate root owning its ordered columns → cards; all mutations enforce the
/// invariants here (≤ 99 columns, contiguous positions, in-board card moves).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    pub(in crate::domain) id: BoardId,
    pub(in crate::domain) project_id: ProjectId,
    pub(in crate::domain) name: EntityName,
    pub(in crate::domain) position: Position,
    pub(in crate::domain) created_at: DateTime<Utc>,
    pub(in crate::domain) updated_at: DateTime<Utc>,
    pub(in crate::domain) columns: Vec<Column>,
}

impl Board {
    /// A fresh board. `position` is the requested placement, but
    /// `BoardRepository::insert_within_limit` reassigns it atomically at insert
    /// so concurrent creates in a project can't collide.
    pub fn new(project_id: ProjectId, name: EntityName, position: Position) -> Self {
        let now = Utc::now();
        Self {
            id: BoardId::new(),
            project_id,
            name,
            position,
            created_at: now,
            updated_at: now,
            columns: Vec::new(),
        }
    }

    /// Rebuilds a board aggregate (with its columns and cards) from persisted state.
    pub fn from_parts(
        id: BoardId,
        project_id: ProjectId,
        name: EntityName,
        position: Position,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        columns: Vec<Column>,
    ) -> Self {
        Self {
            id,
            project_id,
            name,
            position,
            created_at,
            updated_at,
            columns,
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

    pub fn id(&self) -> BoardId {
        self.id
    }

    pub fn project_id(&self) -> ProjectId {
        self.project_id
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

    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    pub fn into_columns(self) -> Vec<Column> {
        self.columns
    }

    pub fn to_summary(&self) -> BoardSummary {
        BoardSummary {
            id: self.id,
            project_id: self.project_id,
            name: self.name.clone(),
            position: self.position,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    pub fn column(&self, id: ColumnId) -> Option<&Column> {
        self.columns.iter().find(|c| c.id == id)
    }

    pub fn card(&self, id: CardId) -> Option<&Card> {
        self.columns
            .iter()
            .flat_map(|c| c.cards.iter())
            .find(|c| c.id == id)
    }

    pub fn add_column(&mut self, name: EntityName) -> Result<&Column, BoardError> {
        if self.columns.len() >= MAX_COLUMNS_PER_BOARD {
            return Err(BoardError::ColumnLimitReached);
        }
        let position = Position::new(self.columns.len() as i32).expect("length is non-negative");
        self.columns.push(Column::new(self.id, name, position));
        Ok(self.columns.last().expect("just pushed"))
    }

    pub fn rename_column(&mut self, id: ColumnId, name: EntityName) -> Result<&Column, BoardError> {
        let column = self
            .columns
            .iter_mut()
            .find(|c| c.id == id)
            .ok_or(BoardError::ColumnNotFound)?;
        column.name = name;
        column.updated_at = Utc::now();
        Ok(column)
    }

    pub fn remove_column(&mut self, id: ColumnId) -> Result<(), BoardError> {
        let index = self
            .columns
            .iter()
            .position(|c| c.id == id)
            .ok_or(BoardError::ColumnNotFound)?;
        self.columns.remove(index);
        reindex(&mut self.columns, |c| &mut c.position);
        Ok(())
    }

    pub fn reorder_columns(&mut self, ordered: &[ColumnId]) -> Result<(), BoardError> {
        let existing: Vec<ColumnId> = self.columns.iter().map(|c| c.id).collect();
        if !is_permutation(&existing, ordered) {
            return Err(BoardError::NotAPermutation);
        }
        let mut taken: Vec<Column> = std::mem::take(&mut self.columns);
        self.columns = ordered
            .iter()
            .map(|id| {
                let index = taken.iter().position(|c| c.id == *id).expect("permutation");
                taken.remove(index)
            })
            .collect();
        reindex(&mut self.columns, |c| &mut c.position);
        Ok(())
    }

    pub fn add_card(
        &mut self,
        column_id: ColumnId,
        title: Title,
        description: Description,
    ) -> Result<&Card, BoardError> {
        let column = self
            .columns
            .iter_mut()
            .find(|c| c.id == column_id)
            .ok_or(BoardError::ColumnNotFound)?;
        let position = Position::new(column.cards.len() as i32).expect("length is non-negative");
        column
            .cards
            .push(Card::new(column_id, title, description, position));
        Ok(column.cards.last().expect("just pushed"))
    }

    pub fn update_card(
        &mut self,
        id: CardId,
        title: Option<Title>,
        description: Option<Description>,
    ) -> Result<&Card, BoardError> {
        let card = self
            .columns
            .iter_mut()
            .flat_map(|c| c.cards.iter_mut())
            .find(|c| c.id == id)
            .ok_or(BoardError::CardNotFound)?;
        if let Some(title) = title {
            card.title = title;
        }
        if let Some(description) = description {
            card.description = description;
        }
        card.updated_at = Utc::now();
        Ok(card)
    }

    pub fn remove_card(&mut self, id: CardId) -> Result<(), BoardError> {
        for column in &mut self.columns {
            if let Some(index) = column.cards.iter().position(|c| c.id == id) {
                column.cards.remove(index);
                reindex(&mut column.cards, |c| &mut c.position);
                return Ok(());
            }
        }
        Err(BoardError::CardNotFound)
    }

    /// Clamps `position`, re-sequences both columns; the target must be in this board.
    pub fn move_card(
        &mut self,
        id: CardId,
        target_column: ColumnId,
        position: i32,
    ) -> Result<(), BoardError> {
        let source = self
            .columns
            .iter()
            .position(|c| c.cards.iter().any(|card| card.id == id))
            .ok_or(BoardError::CardNotFound)?;
        let target = self
            .columns
            .iter()
            .position(|c| c.id == target_column)
            .ok_or(BoardError::TargetColumnNotFound)?;

        let card_index = self.columns[source]
            .cards
            .iter()
            .position(|c| c.id == id)
            .expect("located above");
        let mut card = self.columns[source].cards.remove(card_index);
        reindex(&mut self.columns[source].cards, |c| &mut c.position);

        let clamped = position.clamp(0, self.columns[target].cards.len() as i32) as usize;
        card.column_id = target_column;
        card.updated_at = Utc::now();
        self.columns[target].cards.insert(clamped, card);
        reindex(&mut self.columns[target].cards, |c| &mut c.position);
        Ok(())
    }
}

fn reindex<T>(items: &mut [T], position: impl Fn(&mut T) -> &mut Position) {
    for (index, item) in items.iter_mut().enumerate() {
        *position(item) = Position::new(index as i32).expect("index is non-negative");
    }
}

fn is_permutation<Id: Eq + std::hash::Hash + Copy>(existing: &[Id], provided: &[Id]) -> bool {
    let provided_set: HashSet<Id> = provided.iter().copied().collect();
    let existing_set: HashSet<Id> = existing.iter().copied().collect();
    provided.len() == provided_set.len() && provided_set == existing_set
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board() -> Board {
        Board::new(
            ProjectId::new(),
            EntityName::new("B").unwrap(),
            Position::new(0).unwrap(),
        )
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).unwrap()
    }

    fn title(value: &str) -> Title {
        Title::new(value).unwrap()
    }

    fn column_ids(board: &Board) -> Vec<ColumnId> {
        board.columns.iter().map(|c| c.id).collect()
    }

    fn card_titles(board: &Board, column: ColumnId) -> Vec<String> {
        board
            .column(column)
            .unwrap()
            .cards
            .iter()
            .map(|c| c.title.as_str().to_owned())
            .collect()
    }

    #[test]
    fn add_column_appends_at_contiguous_positions() {
        let mut board = board();
        let first = board.add_column(name("To Do")).unwrap().position.value();
        let second = board.add_column(name("Doing")).unwrap().position.value();
        assert_eq!((first, second), (0, 1));
        assert_eq!(board.columns[1].board_id, board.id);
    }

    #[test]
    fn add_column_rejects_the_hundredth_column() {
        let mut board = board();
        for i in 0..MAX_COLUMNS_PER_BOARD {
            board.add_column(name(&format!("C{i}"))).unwrap();
        }
        assert_eq!(board.columns.len(), 99);
        assert_eq!(
            board.add_column(name("overflow")),
            Err(BoardError::ColumnLimitReached)
        );
    }

    #[test]
    fn rename_column_updates_the_name_or_reports_missing() {
        let mut board = board();
        let id = board.add_column(name("Old")).unwrap().id;
        assert_eq!(
            board.rename_column(id, name("New")).unwrap().name.as_str(),
            "New"
        );
        assert_eq!(
            board.rename_column(ColumnId::new(), name("X")),
            Err(BoardError::ColumnNotFound)
        );
    }

    #[test]
    fn remove_column_reindexes_the_remainder() {
        let mut board = board();
        let a = board.add_column(name("A")).unwrap().id;
        board.add_column(name("B")).unwrap();
        board.add_column(name("C")).unwrap();
        board.remove_column(a).unwrap();
        let positions: Vec<i32> = board.columns.iter().map(|c| c.position.value()).collect();
        assert_eq!(positions, [0, 1]);
        assert_eq!(board.remove_column(a), Err(BoardError::ColumnNotFound));
    }

    #[test]
    fn reorder_columns_permutes_and_reindexes() {
        let mut board = board();
        let a = board.add_column(name("A")).unwrap().id;
        let b = board.add_column(name("B")).unwrap().id;
        let c = board.add_column(name("C")).unwrap().id;
        board.reorder_columns(&[c, a, b]).unwrap();
        assert_eq!(column_ids(&board), [c, a, b]);
        assert_eq!(
            board
                .columns
                .iter()
                .map(|c| c.position.value())
                .collect::<Vec<_>>(),
            [0, 1, 2]
        );
    }

    #[test]
    fn reorder_columns_rejects_a_non_permutation() {
        let mut board = board();
        let a = board.add_column(name("A")).unwrap().id;
        assert_eq!(
            board.reorder_columns(&[a, ColumnId::new()]),
            Err(BoardError::NotAPermutation)
        );
    }

    #[test]
    fn add_card_appends_and_update_applies_only_provided_fields() {
        let mut board = board();
        let column = board.add_column(name("To Do")).unwrap().id;
        let card = board
            .add_card(column, title("A"), Description::default())
            .unwrap();
        let id = card.id;
        assert_eq!(card.position.value(), 0);

        board
            .update_card(id, None, Some(Description::new("body").unwrap()))
            .unwrap();
        let card = board.card(id).unwrap();
        assert_eq!(card.title.as_str(), "A");
        assert_eq!(card.description.as_str(), "body");

        board.update_card(id, Some(title("New")), None).unwrap();
        let card = board.card(id).unwrap();
        assert_eq!(card.title.as_str(), "New");
        assert_eq!(card.description.as_str(), "body");
    }

    #[test]
    fn add_card_and_update_card_report_missing() {
        let mut board = board();
        assert_eq!(
            board.add_card(ColumnId::new(), title("x"), Description::default()),
            Err(BoardError::ColumnNotFound)
        );
        assert_eq!(
            board.update_card(CardId::new(), None, None),
            Err(BoardError::CardNotFound)
        );
    }

    #[test]
    fn remove_card_reindexes_its_column() {
        let mut board = board();
        let column = board.add_column(name("To Do")).unwrap().id;
        let a = board
            .add_card(column, title("A"), Description::default())
            .unwrap()
            .id;
        board
            .add_card(column, title("B"), Description::default())
            .unwrap();
        board.remove_card(a).unwrap();
        assert_eq!(card_titles(&board, column), ["B"]);
        assert_eq!(board.column(column).unwrap().cards[0].position.value(), 0);
        assert_eq!(board.remove_card(a), Err(BoardError::CardNotFound));
    }

    #[test]
    fn move_card_within_a_column_reorders() {
        let mut board = board();
        let column = board.add_column(name("To Do")).unwrap().id;
        let a = board
            .add_card(column, title("A"), Description::default())
            .unwrap()
            .id;
        board
            .add_card(column, title("B"), Description::default())
            .unwrap();
        board
            .add_card(column, title("C"), Description::default())
            .unwrap();

        board.move_card(a, column, 2).unwrap();
        assert_eq!(card_titles(&board, column), ["B", "C", "A"]);
        let positions: Vec<i32> = board
            .column(column)
            .unwrap()
            .cards
            .iter()
            .map(|c| c.position.value())
            .collect();
        assert_eq!(positions, [0, 1, 2]);
    }

    #[test]
    fn move_card_across_columns_reseqs_both_and_clamps() {
        let mut board = board();
        let src = board.add_column(name("Src")).unwrap().id;
        let dst = board.add_column(name("Dst")).unwrap().id;
        let a = board
            .add_card(src, title("A"), Description::default())
            .unwrap()
            .id;
        board
            .add_card(dst, title("X"), Description::default())
            .unwrap();
        board
            .add_card(dst, title("Y"), Description::default())
            .unwrap();

        // A position past the end clamps to append.
        board.move_card(a, dst, 99).unwrap();
        assert!(card_titles(&board, src).is_empty());
        assert_eq!(card_titles(&board, dst), ["X", "Y", "A"]);
        assert_eq!(board.card(a).unwrap().column_id, dst);
    }

    #[test]
    fn move_card_reports_missing_card_and_target() {
        let mut board = board();
        let column = board.add_column(name("To Do")).unwrap().id;
        let a = board
            .add_card(column, title("A"), Description::default())
            .unwrap()
            .id;
        assert_eq!(
            board.move_card(CardId::new(), column, 0),
            Err(BoardError::CardNotFound)
        );
        assert_eq!(
            board.move_card(a, ColumnId::new(), 0),
            Err(BoardError::TargetColumnNotFound)
        );
    }
}
