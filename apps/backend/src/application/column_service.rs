use std::sync::Arc;

use crate::application::error::ApplicationError;
use crate::application::reorder::ensure_permutation;
use crate::domain::column::Column;
use crate::domain::ids::{BoardId, ColumnId};
use crate::domain::name::EntityName;
use crate::domain::ports::{ColumnRepository, LimitedInsert};
use crate::domain::position::Position;

pub const MAX_COLUMNS_PER_BOARD: i64 = 99;

#[derive(Clone)]
pub struct ColumnService {
    columns: Arc<dyn ColumnRepository>,
}

impl ColumnService {
    pub fn new(columns: Arc<dyn ColumnRepository>) -> Self {
        Self { columns }
    }

    pub async fn list(&self, board_id: BoardId) -> Result<Vec<Column>, ApplicationError> {
        Ok(self.columns.list_by_board(board_id).await?)
    }

    pub async fn create(&self, board_id: BoardId, name: &str) -> Result<Column, ApplicationError> {
        let name = EntityName::new(name)?;
        let next = self
            .columns
            .list_by_board(board_id)
            .await?
            .iter()
            .map(|c| c.position.value())
            .max()
            .map_or(0, |max| max + 1);
        let column = Column::new(board_id, name, Position::new(next)?);
        match self
            .columns
            .insert_within_limit(&column, MAX_COLUMNS_PER_BOARD)
            .await?
        {
            LimitedInsert::Created => Ok(column),
            LimitedInsert::ParentMissing => Err(ApplicationError::NotFound),
            LimitedInsert::LimitReached => Err(ApplicationError::LimitExceeded),
        }
    }

    pub async fn update(&self, id: ColumnId, name: &str) -> Result<Column, ApplicationError> {
        let name = EntityName::new(name)?;
        self.columns.update(id, name).await?;
        self.columns.get(id).await?.ok_or(ApplicationError::NotFound)
    }

    pub async fn delete(&self, id: ColumnId) -> Result<(), ApplicationError> {
        if self.columns.get(id).await?.is_none() {
            return Err(ApplicationError::NotFound);
        }
        self.columns.delete(id).await?;
        Ok(())
    }

    pub async fn reorder(
        &self,
        board_id: BoardId,
        ordered_ids: Vec<ColumnId>,
    ) -> Result<(), ApplicationError> {
        let existing: Vec<ColumnId> = self
            .columns
            .list_by_board(board_id)
            .await?
            .into_iter()
            .map(|c| c.id)
            .collect();
        ensure_permutation(&existing, &ordered_ids)?;
        self.columns.reorder(board_id, &ordered_ids).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::error::DomainError;
    use crate::domain::ports::{RepoResult, RepositoryError};
    use async_trait::async_trait;
    use std::sync::Mutex;

    #[derive(Default)]
    struct FakeColumnRepo {
        rows: Mutex<Vec<Column>>,
        fail: bool,
    }

    impl FakeColumnRepo {
        fn new() -> Arc<Self> {
            Arc::new(Self::default())
        }

        fn failing() -> Arc<Self> {
            Arc::new(Self {
                rows: Mutex::new(Vec::new()),
                fail: true,
            })
        }
    }

    #[async_trait]
    impl ColumnRepository for FakeColumnRepo {
        async fn list_by_board(&self, board_id: BoardId) -> RepoResult<Vec<Column>> {
            if self.fail {
                return Err(RepositoryError::new("boom"));
            }
            let mut rows: Vec<Column> = self
                .rows
                .lock()
                .unwrap()
                .iter()
                .filter(|c| c.board_id == board_id)
                .cloned()
                .collect();
            rows.sort_by_key(|c| c.position.value());
            Ok(rows)
        }

        async fn get(&self, id: ColumnId) -> RepoResult<Option<Column>> {
            Ok(self.rows.lock().unwrap().iter().find(|c| c.id == id).cloned())
        }

        async fn insert_within_limit(&self, column: &Column, max: i64) -> RepoResult<LimitedInsert> {
            let mut rows = self.rows.lock().unwrap();
            let count = rows.iter().filter(|c| c.board_id == column.board_id).count() as i64;
            if count >= max {
                return Ok(LimitedInsert::LimitReached);
            }
            rows.push(column.clone());
            Ok(LimitedInsert::Created)
        }

        async fn update(&self, id: ColumnId, name: EntityName) -> RepoResult<()> {
            if let Some(c) = self.rows.lock().unwrap().iter_mut().find(|c| c.id == id) {
                c.name = name;
            }
            Ok(())
        }

        async fn delete(&self, id: ColumnId) -> RepoResult<()> {
            self.rows.lock().unwrap().retain(|c| c.id != id);
            Ok(())
        }

        async fn reorder(&self, board_id: BoardId, ordered_ids: &[ColumnId]) -> RepoResult<()> {
            let mut rows = self.rows.lock().unwrap();
            for (index, id) in ordered_ids.iter().enumerate() {
                if let Some(c) = rows.iter_mut().find(|c| c.id == *id && c.board_id == board_id) {
                    c.position = Position::new(index as i32).unwrap();
                }
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn create_appends_at_the_next_position() {
        let service = ColumnService::new(FakeColumnRepo::new());
        let board = BoardId::new();

        let first = service.create(board, "To Do").await.unwrap();
        let second = service.create(board, "  Doing  ").await.unwrap();

        assert_eq!(first.position.value(), 0);
        assert_eq!(second.position.value(), 1);
        assert_eq!(second.name.as_str(), "Doing");
        assert_eq!(second.board_id, board);
    }

    #[tokio::test]
    async fn create_rejects_a_blank_name() {
        let service = ColumnService::new(FakeColumnRepo::new());
        let err = service.create(BoardId::new(), "  ").await.unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::Domain(DomainError::EmptyName)
        ));
    }

    #[tokio::test]
    async fn create_rejects_the_column_beyond_the_limit() {
        let service = ColumnService::new(FakeColumnRepo::new());
        let board = BoardId::new();
        for i in 0..MAX_COLUMNS_PER_BOARD {
            service.create(board, &format!("C{i}")).await.unwrap();
        }
        let err = service.create(board, "overflow").await.unwrap_err();
        assert!(matches!(err, ApplicationError::LimitExceeded));
    }

    #[tokio::test]
    async fn the_limit_is_per_board() {
        let service = ColumnService::new(FakeColumnRepo::new());
        let full = BoardId::new();
        for i in 0..MAX_COLUMNS_PER_BOARD {
            service.create(full, &format!("C{i}")).await.unwrap();
        }
        assert!(service.create(BoardId::new(), "fresh").await.is_ok());
    }

    #[tokio::test]
    async fn create_surfaces_repository_errors() {
        let service = ColumnService::new(FakeColumnRepo::failing());
        let err = service.create(BoardId::new(), "x").await.unwrap_err();
        assert!(matches!(err, ApplicationError::Repository(_)));
    }

    #[tokio::test]
    async fn update_changes_an_existing_column() {
        let service = ColumnService::new(FakeColumnRepo::new());
        let column = service.create(BoardId::new(), "Old").await.unwrap();

        let updated = service.update(column.id, "New").await.unwrap();
        assert_eq!(updated.name.as_str(), "New");
    }

    #[tokio::test]
    async fn update_of_a_missing_column_is_not_found() {
        let service = ColumnService::new(FakeColumnRepo::new());
        let err = service.update(ColumnId::new(), "New").await.unwrap_err();
        assert!(matches!(err, ApplicationError::NotFound));
    }

    #[tokio::test]
    async fn delete_removes_a_column_and_missing_is_not_found() {
        let service = ColumnService::new(FakeColumnRepo::new());
        let board = BoardId::new();
        let column = service.create(board, "Bye").await.unwrap();

        service.delete(column.id).await.unwrap();
        assert!(service.list(board).await.unwrap().is_empty());
        assert!(matches!(
            service.delete(column.id).await.unwrap_err(),
            ApplicationError::NotFound
        ));
    }

    #[tokio::test]
    async fn reorder_rewrites_positions_for_a_valid_permutation() {
        let service = ColumnService::new(FakeColumnRepo::new());
        let board = BoardId::new();
        let a = service.create(board, "A").await.unwrap();
        let b = service.create(board, "B").await.unwrap();
        let c = service.create(board, "C").await.unwrap();

        service.reorder(board, vec![c.id, a.id, b.id]).await.unwrap();

        let ordered: Vec<ColumnId> = service.list(board).await.unwrap().iter().map(|c| c.id).collect();
        assert_eq!(ordered, [c.id, a.id, b.id]);
    }

    #[tokio::test]
    async fn reorder_rejects_a_non_permutation() {
        let service = ColumnService::new(FakeColumnRepo::new());
        let board = BoardId::new();
        let a = service.create(board, "A").await.unwrap();

        let err = service.reorder(board, vec![a.id, ColumnId::new()]).await.unwrap_err();
        assert!(matches!(err, ApplicationError::Unprocessable(_)));
    }
}
