use std::sync::Arc;

use sqlx::PgPool;

use crate::adapters::outbound::persistence::pg_board_repo::PgBoardRepo;
use crate::adapters::outbound::persistence::pg_column_repo::PgColumnRepo;
use crate::adapters::outbound::persistence::pg_project_repo::PgProjectRepo;
use crate::application::board_service::BoardService;
use crate::application::column_service::ColumnService;
use crate::application::project_service::ProjectService;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub projects: ProjectService,
    pub boards: BoardService,
    pub columns: ColumnService,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        let projects = ProjectService::new(Arc::new(PgProjectRepo::new(pool.clone())));
        let boards = BoardService::new(Arc::new(PgBoardRepo::new(pool.clone())));
        let columns = ColumnService::new(Arc::new(PgColumnRepo::new(pool.clone())));
        Self {
            pool,
            projects,
            boards,
            columns,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn wires_services_from_a_cloneable_pool() {
        // `connect_lazy` validates the URL without opening a connection.
        let pool = PgPool::connect_lazy("postgres://kanban:kanban@localhost:5432/kanban").unwrap();
        let state = AppState::new(pool);
        let clone = state.clone();
        assert_eq!(state.pool.size(), clone.pool.size());
    }
}
