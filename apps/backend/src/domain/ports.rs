use async_trait::async_trait;
use thiserror::Error;

use crate::domain::board::{Board, BoardSummary};
use crate::domain::error::DomainError;
use crate::domain::ids::{BoardId, CardId, ColumnId, ProjectId};
use crate::domain::name::EntityName;
use crate::domain::project::Project;

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

impl From<DomainError> for RepositoryError {
    fn from(error: DomainError) -> Self {
        Self::new(error.to_string())
    }
}

pub type RepoResult<T> = Result<T, RepositoryError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitedInsert {
    Created,
    LimitReached,
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

/// Port for the Board aggregate; touches only boards/columns/cards, never `projects`.
#[async_trait]
pub trait BoardRepository: Send + Sync {
    async fn list_by_project(&self, project_id: ProjectId) -> RepoResult<Vec<BoardSummary>>;
    async fn summary(&self, id: BoardId) -> RepoResult<Option<BoardSummary>>;
    async fn load(&self, id: BoardId) -> RepoResult<Option<Board>>;
    async fn load_by_column(&self, column_id: ColumnId) -> RepoResult<Option<Board>>;
    async fn load_by_card(&self, card_id: CardId) -> RepoResult<Option<Board>>;
    async fn insert_within_limit(&self, board: &Board, max: i64) -> RepoResult<LimitedInsert>;
    async fn save(&self, board: &Board) -> RepoResult<()>;
    async fn update(&self, id: BoardId, name: EntityName) -> RepoResult<()>;
    async fn delete(&self, id: BoardId) -> RepoResult<()>;
    async fn reorder(&self, project_id: ProjectId, ordered_ids: &[BoardId]) -> RepoResult<()>;
}

/// Checks a project by id without reading the Project aggregate's tables.
#[async_trait]
pub trait ProjectDirectory: Send + Sync {
    async fn exists(&self, id: ProjectId) -> RepoResult<bool>;
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
        assert_dyn::<dyn ProjectDirectory>();
    }
}
