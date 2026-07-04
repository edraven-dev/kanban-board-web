use std::sync::Arc;

use crate::application::error::ApplicationError;
use crate::application::reorder::ensure_permutation;
use crate::domain::board::Board;
use crate::domain::ids::{BoardId, ProjectId};
use crate::domain::name::EntityName;
use crate::domain::ports::{BoardRepository, LimitedInsert};
use crate::domain::position::Position;

pub const MAX_BOARDS_PER_PROJECT: i64 = 99;

#[derive(Clone)]
pub struct BoardService {
    boards: Arc<dyn BoardRepository>,
}

impl BoardService {
    pub fn new(boards: Arc<dyn BoardRepository>) -> Self {
        Self { boards }
    }

    pub async fn list(&self, project_id: ProjectId) -> Result<Vec<Board>, ApplicationError> {
        Ok(self.boards.list_by_project(project_id).await?)
    }

    pub async fn create(
        &self,
        project_id: ProjectId,
        name: &str,
    ) -> Result<Board, ApplicationError> {
        let name = EntityName::new(name)?;
        let next = self
            .boards
            .list_by_project(project_id)
            .await?
            .iter()
            .map(|b| b.position.value())
            .max()
            .map_or(0, |max| max + 1);
        let board = Board::new(project_id, name, Position::new(next)?);
        match self
            .boards
            .insert_within_limit(&board, MAX_BOARDS_PER_PROJECT)
            .await?
        {
            LimitedInsert::Created => Ok(board),
            LimitedInsert::ParentMissing => Err(ApplicationError::NotFound),
            LimitedInsert::LimitReached => Err(ApplicationError::LimitExceeded),
        }
    }

    pub async fn update(&self, id: BoardId, name: &str) -> Result<Board, ApplicationError> {
        let name = EntityName::new(name)?;
        self.boards.update(id, name).await?;
        self.boards.get(id).await?.ok_or(ApplicationError::NotFound)
    }

    pub async fn delete(&self, id: BoardId) -> Result<(), ApplicationError> {
        if self.boards.get(id).await?.is_none() {
            return Err(ApplicationError::NotFound);
        }
        self.boards.delete(id).await?;
        Ok(())
    }

    pub async fn reorder(
        &self,
        project_id: ProjectId,
        ordered_ids: Vec<BoardId>,
    ) -> Result<(), ApplicationError> {
        let existing: Vec<BoardId> = self
            .boards
            .list_by_project(project_id)
            .await?
            .into_iter()
            .map(|b| b.id)
            .collect();
        ensure_permutation(&existing, &ordered_ids)?;
        self.boards.reorder(project_id, &ordered_ids).await?;
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
    struct FakeBoardRepo {
        rows: Mutex<Vec<Board>>,
        fail: bool,
    }

    impl FakeBoardRepo {
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
    impl BoardRepository for FakeBoardRepo {
        async fn list_by_project(&self, project_id: ProjectId) -> RepoResult<Vec<Board>> {
            if self.fail {
                return Err(RepositoryError::new("boom"));
            }
            let mut rows: Vec<Board> = self
                .rows
                .lock()
                .unwrap()
                .iter()
                .filter(|b| b.project_id == project_id)
                .cloned()
                .collect();
            rows.sort_by_key(|b| b.position.value());
            Ok(rows)
        }

        async fn get(&self, id: BoardId) -> RepoResult<Option<Board>> {
            Ok(self.rows.lock().unwrap().iter().find(|b| b.id == id).cloned())
        }

        async fn insert_within_limit(&self, board: &Board, max: i64) -> RepoResult<LimitedInsert> {
            let mut rows = self.rows.lock().unwrap();
            let count = rows.iter().filter(|b| b.project_id == board.project_id).count() as i64;
            if count >= max {
                return Ok(LimitedInsert::LimitReached);
            }
            rows.push(board.clone());
            Ok(LimitedInsert::Created)
        }

        async fn update(&self, id: BoardId, name: EntityName) -> RepoResult<()> {
            if let Some(b) = self.rows.lock().unwrap().iter_mut().find(|b| b.id == id) {
                b.name = name;
            }
            Ok(())
        }

        async fn delete(&self, id: BoardId) -> RepoResult<()> {
            self.rows.lock().unwrap().retain(|b| b.id != id);
            Ok(())
        }

        async fn reorder(&self, project_id: ProjectId, ordered_ids: &[BoardId]) -> RepoResult<()> {
            let mut rows = self.rows.lock().unwrap();
            for (index, id) in ordered_ids.iter().enumerate() {
                if let Some(b) = rows.iter_mut().find(|b| b.id == *id && b.project_id == project_id) {
                    b.position = Position::new(index as i32).unwrap();
                }
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn create_appends_at_the_next_position() {
        let service = BoardService::new(FakeBoardRepo::new());
        let project = ProjectId::new();

        let first = service.create(project, "Backlog").await.unwrap();
        let second = service.create(project, "  Doing  ").await.unwrap();

        assert_eq!(first.position.value(), 0);
        assert_eq!(second.position.value(), 1);
        assert_eq!(second.name.as_str(), "Doing");
        assert_eq!(second.project_id, project);
    }

    #[tokio::test]
    async fn create_rejects_a_blank_name() {
        let service = BoardService::new(FakeBoardRepo::new());
        let err = service.create(ProjectId::new(), "  ").await.unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::Domain(DomainError::EmptyName)
        ));
    }

    #[tokio::test]
    async fn create_rejects_the_board_beyond_the_limit() {
        let service = BoardService::new(FakeBoardRepo::new());
        let project = ProjectId::new();
        for i in 0..MAX_BOARDS_PER_PROJECT {
            service.create(project, &format!("B{i}")).await.unwrap();
        }
        let err = service.create(project, "overflow").await.unwrap_err();
        assert!(matches!(err, ApplicationError::LimitExceeded));
    }

    #[tokio::test]
    async fn the_limit_is_per_project() {
        let service = BoardService::new(FakeBoardRepo::new());
        let full = ProjectId::new();
        for i in 0..MAX_BOARDS_PER_PROJECT {
            service.create(full, &format!("B{i}")).await.unwrap();
        }
        // A different project is unaffected by the first one being full.
        assert!(service.create(ProjectId::new(), "fresh").await.is_ok());
    }

    #[tokio::test]
    async fn create_surfaces_repository_errors() {
        let service = BoardService::new(FakeBoardRepo::failing());
        let err = service.create(ProjectId::new(), "x").await.unwrap_err();
        assert!(matches!(err, ApplicationError::Repository(_)));
    }

    #[tokio::test]
    async fn update_changes_an_existing_board() {
        let service = BoardService::new(FakeBoardRepo::new());
        let project = ProjectId::new();
        let board = service.create(project, "Old").await.unwrap();

        let updated = service.update(board.id, "New").await.unwrap();
        assert_eq!(updated.name.as_str(), "New");
    }

    #[tokio::test]
    async fn update_of_a_missing_board_is_not_found() {
        let service = BoardService::new(FakeBoardRepo::new());
        let err = service.update(BoardId::new(), "New").await.unwrap_err();
        assert!(matches!(err, ApplicationError::NotFound));
    }

    #[tokio::test]
    async fn delete_removes_a_board_and_missing_is_not_found() {
        let service = BoardService::new(FakeBoardRepo::new());
        let project = ProjectId::new();
        let board = service.create(project, "Bye").await.unwrap();

        service.delete(board.id).await.unwrap();
        assert!(service.list(project).await.unwrap().is_empty());
        assert!(matches!(
            service.delete(board.id).await.unwrap_err(),
            ApplicationError::NotFound
        ));
    }

    #[tokio::test]
    async fn reorder_rewrites_positions_for_a_valid_permutation() {
        let service = BoardService::new(FakeBoardRepo::new());
        let project = ProjectId::new();
        let a = service.create(project, "A").await.unwrap();
        let b = service.create(project, "B").await.unwrap();
        let c = service.create(project, "C").await.unwrap();

        service.reorder(project, vec![c.id, a.id, b.id]).await.unwrap();

        let ordered: Vec<BoardId> = service.list(project).await.unwrap().iter().map(|b| b.id).collect();
        assert_eq!(ordered, [c.id, a.id, b.id]);
    }

    #[tokio::test]
    async fn reorder_rejects_a_non_permutation() {
        let service = BoardService::new(FakeBoardRepo::new());
        let project = ProjectId::new();
        let a = service.create(project, "A").await.unwrap();

        let err = service.reorder(project, vec![a.id, BoardId::new()]).await.unwrap_err();
        assert!(matches!(err, ApplicationError::Unprocessable(_)));
    }
}
