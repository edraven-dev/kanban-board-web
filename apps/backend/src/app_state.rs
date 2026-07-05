use std::sync::Arc;

use sqlx::PgPool;

use crate::adapters::outbound::persistence::pg_board_repo::PgBoardRepo;
use crate::adapters::outbound::persistence::pg_project_repo::PgProjectRepo;
use crate::application::board_service::BoardService;
use crate::application::project_directory::ProjectDirectoryService;
use crate::application::project_service::ProjectService;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub projects: ProjectService,
    pub boards: BoardService,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        let project_repo = Arc::new(PgProjectRepo::new(pool.clone()));
        let board_repo = Arc::new(PgBoardRepo::new(pool.clone()));

        let directory = Arc::new(ProjectDirectoryService::new(project_repo.clone()));

        let projects = ProjectService::new(project_repo);
        let boards = BoardService::new(board_repo, directory);
        Self {
            pool,
            projects,
            boards,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn wires_services_from_a_cloneable_pool() {
        let pool = PgPool::connect_lazy("postgres://kanban:kanban@localhost:5432/kanban").unwrap();
        let state = AppState::new(pool);
        let clone = state.clone();
        assert_eq!(state.pool.size(), clone.pool.size());
    }
}
