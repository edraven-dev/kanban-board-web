use std::sync::Arc;

use crate::application::error::ApplicationError;
use crate::application::reorder::ensure_permutation;
use crate::domain::board::{Board, BoardSummary};
use crate::domain::card::Card;
use crate::domain::column::Column;
use crate::domain::description::Description;
use crate::domain::ids::{BoardId, CardId, ColumnId, ProjectId};
use crate::domain::name::EntityName;
use crate::domain::ports::{BoardRepository, LimitedInsert, ProjectDirectory};
use crate::domain::position::Position;
use crate::domain::title::Title;

pub const MAX_BOARDS_PER_PROJECT: i64 = 99;

/// Board-aggregate use cases; column/card writes load the board, mutate the root, and save.
#[derive(Clone)]
pub struct BoardService {
    boards: Arc<dyn BoardRepository>,
    projects: Arc<dyn ProjectDirectory>,
}

impl BoardService {
    pub fn new(boards: Arc<dyn BoardRepository>, projects: Arc<dyn ProjectDirectory>) -> Self {
        Self { boards, projects }
    }

    pub async fn list(&self, project_id: ProjectId) -> Result<Vec<BoardSummary>, ApplicationError> {
        Ok(self.boards.list_by_project(project_id).await?)
    }

    pub async fn create(
        &self,
        project_id: ProjectId,
        name: &str,
    ) -> Result<BoardSummary, ApplicationError> {
        let name = EntityName::new(name)?;
        if !self.projects.exists(project_id).await? {
            return Err(ApplicationError::NotFound);
        }
        // Position is a placeholder; the repository assigns it atomically at insert.
        let board = Board::new(project_id, name, Position::new(0)?);
        match self
            .boards
            .insert_within_limit(&board, MAX_BOARDS_PER_PROJECT)
            .await?
        {
            // The position is assigned by the repository under its per-project lock,
            // so read it back rather than trusting the in-memory board.
            LimitedInsert::Created => self
                .boards
                .summary(board.id())
                .await?
                .ok_or(ApplicationError::NotFound),
            LimitedInsert::LimitReached => Err(ApplicationError::LimitExceeded),
        }
    }

    pub async fn update(&self, id: BoardId, name: &str) -> Result<BoardSummary, ApplicationError> {
        let name = EntityName::new(name)?;
        self.boards.update(id, name).await?;
        self.boards
            .summary(id)
            .await?
            .ok_or(ApplicationError::NotFound)
    }

    pub async fn delete(&self, id: BoardId) -> Result<(), ApplicationError> {
        if self.boards.summary(id).await?.is_none() {
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
            .map(|b| b.id())
            .collect();
        ensure_permutation(&existing, &ordered_ids)?;
        self.boards.reorder(project_id, &ordered_ids).await?;
        Ok(())
    }

    pub async fn get_full(&self, id: BoardId) -> Result<Option<Board>, ApplicationError> {
        Ok(self.boards.load(id).await?)
    }

    pub async fn list_columns(&self, board_id: BoardId) -> Result<Vec<Column>, ApplicationError> {
        Ok(self
            .boards
            .load(board_id)
            .await?
            .map(Board::into_columns)
            .unwrap_or_default())
    }

    pub async fn create_column(
        &self,
        board_id: BoardId,
        name: &str,
    ) -> Result<Column, ApplicationError> {
        let name = EntityName::new(name)?;
        let mut board = self
            .boards
            .load(board_id)
            .await?
            .ok_or(ApplicationError::NotFound)?;
        let column = board.add_column(name)?.clone();
        self.boards.save(&board).await?;
        Ok(column)
    }

    pub async fn rename_column(
        &self,
        id: ColumnId,
        name: &str,
    ) -> Result<Column, ApplicationError> {
        let name = EntityName::new(name)?;
        let mut board = self
            .boards
            .load_by_column(id)
            .await?
            .ok_or(ApplicationError::NotFound)?;
        let column = board.rename_column(id, name)?.clone();
        self.boards.save(&board).await?;
        Ok(column)
    }

    pub async fn delete_column(&self, id: ColumnId) -> Result<(), ApplicationError> {
        let mut board = self
            .boards
            .load_by_column(id)
            .await?
            .ok_or(ApplicationError::NotFound)?;
        board.remove_column(id)?;
        self.boards.save(&board).await?;
        Ok(())
    }

    pub async fn reorder_columns(
        &self,
        board_id: BoardId,
        ordered_ids: Vec<ColumnId>,
    ) -> Result<(), ApplicationError> {
        let mut board = self
            .boards
            .load(board_id)
            .await?
            .ok_or(ApplicationError::NotFound)?;
        board.reorder_columns(&ordered_ids)?;
        self.boards.save(&board).await?;
        Ok(())
    }

    pub async fn list_cards(&self, column_id: ColumnId) -> Result<Vec<Card>, ApplicationError> {
        let Some(board) = self.boards.load_by_column(column_id).await? else {
            return Ok(Vec::new());
        };
        Ok(board
            .column(column_id)
            .map(|c| c.cards().to_vec())
            .unwrap_or_default())
    }

    pub async fn get_card(&self, id: CardId) -> Result<Card, ApplicationError> {
        let board = self
            .boards
            .load_by_card(id)
            .await?
            .ok_or(ApplicationError::NotFound)?;
        board.card(id).cloned().ok_or(ApplicationError::NotFound)
    }

    pub async fn create_card(
        &self,
        column_id: ColumnId,
        title: &str,
        description: Option<&str>,
    ) -> Result<Card, ApplicationError> {
        let title = Title::new(title)?;
        let description = match description {
            Some(text) => Description::new(text)?,
            None => Description::default(),
        };
        let mut board = self
            .boards
            .load_by_column(column_id)
            .await?
            .ok_or(ApplicationError::NotFound)?;
        let card = board.add_card(column_id, title, description)?.clone();
        self.boards.save(&board).await?;
        Ok(card)
    }

    pub async fn update_card(
        &self,
        id: CardId,
        title: Option<&str>,
        description: Option<&str>,
    ) -> Result<Card, ApplicationError> {
        let title = match title {
            Some(text) => Some(Title::new(text)?),
            None => None,
        };
        let description = match description {
            Some(text) => Some(Description::new(text)?),
            None => None,
        };
        let mut board = self
            .boards
            .load_by_card(id)
            .await?
            .ok_or(ApplicationError::NotFound)?;
        let card = board.update_card(id, title, description)?.clone();
        self.boards.save(&board).await?;
        Ok(card)
    }

    pub async fn delete_card(&self, id: CardId) -> Result<(), ApplicationError> {
        let mut board = self
            .boards
            .load_by_card(id)
            .await?
            .ok_or(ApplicationError::NotFound)?;
        board.remove_card(id)?;
        self.boards.save(&board).await?;
        Ok(())
    }

    pub async fn move_card(
        &self,
        id: CardId,
        target_column: ColumnId,
        position: i32,
    ) -> Result<(), ApplicationError> {
        Position::new(position)?;
        let mut board = self
            .boards
            .load_by_card(id)
            .await?
            .ok_or(ApplicationError::NotFound)?;
        board.move_card(id, target_column, position)?;
        self.boards.save(&board).await?;
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
        boards: Mutex<Vec<Board>>,
        fail: bool,
    }

    impl FakeBoardRepo {
        fn new() -> Arc<Self> {
            Arc::new(Self::default())
        }

        fn failing() -> Arc<Self> {
            Arc::new(Self {
                boards: Mutex::new(Vec::new()),
                fail: true,
            })
        }
    }

    #[async_trait]
    impl BoardRepository for FakeBoardRepo {
        async fn list_by_project(&self, project_id: ProjectId) -> RepoResult<Vec<BoardSummary>> {
            if self.fail {
                return Err(RepositoryError::new("boom"));
            }
            let mut rows: Vec<BoardSummary> = self
                .boards
                .lock()
                .unwrap()
                .iter()
                .filter(|b| b.project_id() == project_id)
                .map(|b| b.to_summary())
                .collect();
            rows.sort_by_key(|b| b.position().value());
            Ok(rows)
        }

        async fn summary(&self, id: BoardId) -> RepoResult<Option<BoardSummary>> {
            Ok(self
                .boards
                .lock()
                .unwrap()
                .iter()
                .find(|b| b.id() == id)
                .map(|b| b.to_summary()))
        }

        async fn load(&self, id: BoardId) -> RepoResult<Option<Board>> {
            if self.fail {
                return Err(RepositoryError::new("boom"));
            }
            Ok(self
                .boards
                .lock()
                .unwrap()
                .iter()
                .find(|b| b.id() == id)
                .cloned())
        }

        async fn load_by_column(&self, column_id: ColumnId) -> RepoResult<Option<Board>> {
            Ok(self
                .boards
                .lock()
                .unwrap()
                .iter()
                .find(|b| b.columns().iter().any(|c| c.id() == column_id))
                .cloned())
        }

        async fn load_by_card(&self, card_id: CardId) -> RepoResult<Option<Board>> {
            Ok(self
                .boards
                .lock()
                .unwrap()
                .iter()
                .find(|b| {
                    b.columns()
                        .iter()
                        .flat_map(|c| c.cards().iter())
                        .any(|c| c.id() == card_id)
                })
                .cloned())
        }

        async fn insert_within_limit(&self, board: &Board, max: i64) -> RepoResult<LimitedInsert> {
            if self.fail {
                return Err(RepositoryError::new("boom"));
            }
            let mut rows = self.boards.lock().unwrap();
            let positions: Vec<i32> = rows
                .iter()
                .filter(|b| b.project_id() == board.project_id())
                .map(|b| b.position().value())
                .collect();
            if positions.len() as i64 >= max {
                return Ok(LimitedInsert::LimitReached);
            }
            let next = positions.iter().max().map_or(0, |m| m + 1);
            let mut stored = board.clone();
            stored.reposition(Position::new(next).unwrap());
            rows.push(stored);
            Ok(LimitedInsert::Created)
        }

        async fn save(&self, board: &Board) -> RepoResult<()> {
            let mut rows = self.boards.lock().unwrap();
            if let Some(slot) = rows.iter_mut().find(|b| b.id() == board.id()) {
                *slot = board.clone();
            }
            Ok(())
        }

        async fn update(&self, id: BoardId, name: EntityName) -> RepoResult<()> {
            if let Some(b) = self
                .boards
                .lock()
                .unwrap()
                .iter_mut()
                .find(|b| b.id() == id)
            {
                b.rename(name);
            }
            Ok(())
        }

        async fn delete(&self, id: BoardId) -> RepoResult<()> {
            self.boards.lock().unwrap().retain(|b| b.id() != id);
            Ok(())
        }

        async fn reorder(&self, project_id: ProjectId, ordered_ids: &[BoardId]) -> RepoResult<()> {
            let mut rows = self.boards.lock().unwrap();
            for (index, id) in ordered_ids.iter().enumerate() {
                if let Some(b) = rows
                    .iter_mut()
                    .find(|b| b.id() == *id && b.project_id() == project_id)
                {
                    b.reposition(Position::new(index as i32).unwrap());
                }
            }
            Ok(())
        }
    }

    struct FakeDirectory {
        present: bool,
    }

    impl FakeDirectory {
        fn present() -> Arc<Self> {
            Arc::new(Self { present: true })
        }
        fn absent() -> Arc<Self> {
            Arc::new(Self { present: false })
        }
    }

    #[async_trait]
    impl ProjectDirectory for FakeDirectory {
        async fn exists(&self, _id: ProjectId) -> RepoResult<bool> {
            Ok(self.present)
        }
    }

    fn service(repo: Arc<FakeBoardRepo>, directory: Arc<FakeDirectory>) -> BoardService {
        BoardService::new(repo, directory)
    }

    async fn board_with_column() -> (BoardService, Arc<FakeBoardRepo>, BoardId, ColumnId) {
        let repo = FakeBoardRepo::new();
        let svc = service(repo.clone(), FakeDirectory::present());
        let project = ProjectId::new();
        let board = svc.create(project, "B").await.unwrap();
        let column = svc.create_column(board.id(), "To Do").await.unwrap();
        (svc, repo, board.id(), column.id())
    }

    #[tokio::test]
    async fn create_appends_and_rejects_blank_name_missing_project_and_limit() {
        let repo = FakeBoardRepo::new();
        let svc = service(repo.clone(), FakeDirectory::present());
        let project = ProjectId::new();

        let first = svc.create(project, "A").await.unwrap();
        let second = svc.create(project, "  B  ").await.unwrap();
        assert_eq!(
            (first.position().value(), second.position().value()),
            (0, 1)
        );
        assert_eq!(second.name().as_str(), "B");

        assert!(matches!(
            svc.create(project, "  ").await.unwrap_err(),
            ApplicationError::Domain(DomainError::EmptyName)
        ));

        let absent = service(FakeBoardRepo::new(), FakeDirectory::absent());
        assert!(matches!(
            absent.create(ProjectId::new(), "X").await.unwrap_err(),
            ApplicationError::NotFound
        ));
    }

    #[tokio::test]
    async fn create_enforces_the_per_project_limit() {
        let svc = service(FakeBoardRepo::new(), FakeDirectory::present());
        let full = ProjectId::new();
        for i in 0..MAX_BOARDS_PER_PROJECT {
            svc.create(full, &format!("B{i}")).await.unwrap();
        }
        assert!(matches!(
            svc.create(full, "overflow").await.unwrap_err(),
            ApplicationError::LimitExceeded
        ));
        assert!(svc.create(ProjectId::new(), "fresh").await.is_ok());
    }

    #[tokio::test]
    async fn create_surfaces_repository_errors() {
        let svc = service(FakeBoardRepo::failing(), FakeDirectory::present());
        assert!(matches!(
            svc.create(ProjectId::new(), "x").await.unwrap_err(),
            ApplicationError::Repository(_)
        ));
    }

    #[tokio::test]
    async fn update_and_delete_and_missing_are_handled() {
        let repo = FakeBoardRepo::new();
        let svc = service(repo.clone(), FakeDirectory::present());
        let board = svc.create(ProjectId::new(), "Old").await.unwrap();

        assert_eq!(
            svc.update(board.id(), "New").await.unwrap().name().as_str(),
            "New"
        );
        assert!(matches!(
            svc.update(BoardId::new(), "X").await.unwrap_err(),
            ApplicationError::NotFound
        ));

        svc.delete(board.id()).await.unwrap();
        assert!(matches!(
            svc.delete(board.id()).await.unwrap_err(),
            ApplicationError::NotFound
        ));
    }

    #[tokio::test]
    async fn reorder_permutes_or_rejects() {
        let svc = service(FakeBoardRepo::new(), FakeDirectory::present());
        let project = ProjectId::new();
        let a = svc.create(project, "A").await.unwrap().id();
        let b = svc.create(project, "B").await.unwrap().id();
        let c = svc.create(project, "C").await.unwrap().id();

        svc.reorder(project, vec![c, a, b]).await.unwrap();
        let ordered: Vec<BoardId> = svc
            .list(project)
            .await
            .unwrap()
            .iter()
            .map(|b| b.id())
            .collect();
        assert_eq!(ordered, [c, a, b]);

        assert!(matches!(
            svc.reorder(project, vec![a, BoardId::new()])
                .await
                .unwrap_err(),
            ApplicationError::Unprocessable(_)
        ));
    }

    #[tokio::test]
    async fn create_column_appends_rejects_missing_board_and_limit() {
        let (svc, _repo, board_id, _col) = board_with_column().await;

        assert!(matches!(
            svc.create_column(BoardId::new(), "x").await.unwrap_err(),
            ApplicationError::NotFound
        ));

        for i in 1..99 {
            svc.create_column(board_id, &format!("C{i}")).await.unwrap();
        }
        assert!(matches!(
            svc.create_column(board_id, "overflow").await.unwrap_err(),
            ApplicationError::LimitExceeded
        ));
    }

    #[tokio::test]
    async fn column_rename_delete_and_reorder_go_through_the_root() {
        let (svc, _repo, board_id, column) = board_with_column().await;
        let second = svc.create_column(board_id, "Second").await.unwrap().id();

        assert_eq!(
            svc.rename_column(column, "Renamed")
                .await
                .unwrap()
                .name()
                .as_str(),
            "Renamed"
        );
        assert!(matches!(
            svc.rename_column(ColumnId::new(), "X").await.unwrap_err(),
            ApplicationError::NotFound
        ));

        svc.reorder_columns(board_id, vec![second, column])
            .await
            .unwrap();
        let full = svc.get_full(board_id).await.unwrap().unwrap();
        assert_eq!(
            full.columns().iter().map(|c| c.id()).collect::<Vec<_>>(),
            [second, column]
        );
        assert!(matches!(
            svc.reorder_columns(board_id, vec![column])
                .await
                .unwrap_err(),
            ApplicationError::Unprocessable(_)
        ));

        svc.delete_column(column).await.unwrap();
        assert_eq!(svc.list_columns(board_id).await.unwrap().len(), 1);
        assert!(matches!(
            svc.delete_column(column).await.unwrap_err(),
            ApplicationError::NotFound
        ));
    }

    #[tokio::test]
    async fn card_lifecycle_and_move_go_through_the_root() {
        let (svc, _repo, board_id, column) = board_with_column().await;
        let other = svc.create_column(board_id, "Done").await.unwrap().id();

        let card = svc.create_card(column, "Task", Some("body")).await.unwrap();
        assert_eq!(card.description().as_str(), "body");
        assert_eq!(
            svc.create_card(column, "Second", None)
                .await
                .unwrap()
                .position()
                .value(),
            1
        );

        let updated = svc.update_card(card.id(), Some("New"), None).await.unwrap();
        assert_eq!(
            (updated.title().as_str(), updated.description().as_str()),
            ("New", "body")
        );

        assert_eq!(
            svc.get_card(card.id()).await.unwrap().title().as_str(),
            "New"
        );

        svc.move_card(card.id(), other, 0).await.unwrap();
        assert_eq!(svc.list_cards(other).await.unwrap()[0].id(), card.id());
        assert_eq!(svc.list_cards(column).await.unwrap().len(), 1);

        svc.delete_card(card.id()).await.unwrap();
        assert!(svc.list_cards(other).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn card_errors_map_to_the_right_status() {
        let (svc, _repo, _board_id, column) = board_with_column().await;

        assert!(matches!(
            svc.create_card(ColumnId::new(), "x", None)
                .await
                .unwrap_err(),
            ApplicationError::NotFound
        ));
        assert!(matches!(
            svc.create_card(column, "  ", None).await.unwrap_err(),
            ApplicationError::Domain(DomainError::EmptyTitle)
        ));
        assert!(matches!(
            svc.get_card(CardId::new()).await.unwrap_err(),
            ApplicationError::NotFound
        ));
        assert!(matches!(
            svc.update_card(CardId::new(), Some("x"), None)
                .await
                .unwrap_err(),
            ApplicationError::NotFound
        ));
        assert!(matches!(
            svc.delete_card(CardId::new()).await.unwrap_err(),
            ApplicationError::NotFound
        ));

        let card = svc.create_card(column, "A", None).await.unwrap();
        assert!(matches!(
            svc.move_card(card.id(), column, -1).await.unwrap_err(),
            ApplicationError::Domain(DomainError::NegativePosition(-1))
        ));
        assert!(matches!(
            svc.move_card(card.id(), ColumnId::new(), 0)
                .await
                .unwrap_err(),
            ApplicationError::NotFound
        ));
    }

    #[tokio::test]
    async fn list_columns_and_cards_are_empty_for_missing_parents() {
        let svc = service(FakeBoardRepo::new(), FakeDirectory::present());
        assert!(svc.list_columns(BoardId::new()).await.unwrap().is_empty());
        assert!(svc.list_cards(ColumnId::new()).await.unwrap().is_empty());
        assert!(svc.get_full(BoardId::new()).await.unwrap().is_none());
    }
}
