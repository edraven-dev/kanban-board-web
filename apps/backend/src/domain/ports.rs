use async_trait::async_trait;
use thiserror::Error;

use crate::domain::board::Board;
use crate::domain::card::Card;
use crate::domain::column::Column;
use crate::domain::description::Description;
use crate::domain::ids::{BoardId, CardId, ColumnId, ProjectId};
use crate::domain::name::EntityName;
use crate::domain::project::Project;
use crate::domain::title::Title;

#[derive(Debug, Error)]
#[error("repository error: {message}")]
pub struct RepositoryError {
    message: String,
}

impl RepositoryError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

pub type RepoResult<T> = Result<T, RepositoryError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitedInsert {
    Created,
    ParentMissing,
    LimitReached,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertOutcome {
    Inserted,
    ParentMissing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveOutcome {
    Moved,
    CardMissing,
    TargetMissing,
}

#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn list(&self) -> RepoResult<Vec<Project>>;
    async fn get(&self, id: ProjectId) -> RepoResult<Option<Project>>;
    async fn insert(&self, project: &Project) -> RepoResult<()>;
    async fn update(&self, id: ProjectId, name: EntityName) -> RepoResult<()>;
    async fn delete(&self, id: ProjectId) -> RepoResult<()>;
    async fn reorder(&self, ordered_ids: &[ProjectId]) -> RepoResult<()>;
}

#[async_trait]
pub trait BoardRepository: Send + Sync {
    async fn list_by_project(&self, project_id: ProjectId) -> RepoResult<Vec<Board>>;
    async fn get(&self, id: BoardId) -> RepoResult<Option<Board>>;
    async fn insert_within_limit(&self, board: &Board, max: i64) -> RepoResult<LimitedInsert>;
    async fn update(&self, id: BoardId, name: EntityName) -> RepoResult<()>;
    async fn delete(&self, id: BoardId) -> RepoResult<()>;
    async fn reorder(&self, project_id: ProjectId, ordered_ids: &[BoardId]) -> RepoResult<()>;
}

#[async_trait]
pub trait ColumnRepository: Send + Sync {
    async fn list_by_board(&self, board_id: BoardId) -> RepoResult<Vec<Column>>;
    async fn get(&self, id: ColumnId) -> RepoResult<Option<Column>>;
    async fn insert_within_limit(&self, column: &Column, max: i64) -> RepoResult<LimitedInsert>;
    async fn update(&self, id: ColumnId, name: EntityName) -> RepoResult<()>;
    async fn delete(&self, id: ColumnId) -> RepoResult<()>;
    async fn reorder(&self, board_id: BoardId, ordered_ids: &[ColumnId]) -> RepoResult<()>;
}

#[async_trait]
pub trait CardRepository: Send + Sync {
    async fn list_by_column(&self, column_id: ColumnId) -> RepoResult<Vec<Card>>;
    async fn list_by_columns(&self, column_ids: &[ColumnId]) -> RepoResult<Vec<Card>>;
    async fn get(&self, id: CardId) -> RepoResult<Option<Card>>;
    async fn insert(&self, card: &Card) -> RepoResult<InsertOutcome>;
    async fn update(&self, id: CardId, title: Title, description: Description) -> RepoResult<()>;
    async fn delete(&self, id: CardId) -> RepoResult<()>;
    async fn move_card(
        &self,
        id: CardId,
        target_column: ColumnId,
        position: i32,
    ) -> RepoResult<MoveOutcome>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_error_carries_its_message() {
        let err = RepositoryError::new("connection reset");
        assert_eq!(err.to_string(), "repository error: connection reset");
    }

    #[test]
    fn ports_are_object_safe() {
        fn assert_dyn<T: ?Sized>() {}
        assert_dyn::<dyn ProjectRepository>();
        assert_dyn::<dyn BoardRepository>();
        assert_dyn::<dyn ColumnRepository>();
        assert_dyn::<dyn CardRepository>();
    }
}
