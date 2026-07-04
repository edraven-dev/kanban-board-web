use std::sync::Arc;

use sqlx::PgPool;

use crate::adapters::outbound::persistence::pg_board_repo::PgBoardRepo;
use crate::adapters::outbound::persistence::pg_card_repo::PgCardRepo;
use crate::adapters::outbound::persistence::pg_column_repo::PgColumnRepo;
use crate::adapters::outbound::persistence::pg_project_repo::PgProjectRepo;
use crate::application::board_service::BoardService;
use crate::application::board_view_service::BoardViewService;
use crate::application::card_service::CardService;
use crate::application::column_service::ColumnService;
use crate::application::project_service::ProjectService;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub projects: ProjectService,
    pub boards: BoardService,
    pub columns: ColumnService,
    pub cards: CardService,
    pub board_view: BoardViewService,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        let board_repo = Arc::new(PgBoardRepo::new(pool.clone()));
        let column_repo = Arc::new(PgColumnRepo::new(pool.clone()));
        let card_repo = Arc::new(PgCardRepo::new(pool.clone()));

        let projects = ProjectService::new(Arc::new(PgProjectRepo::new(pool.clone())));
        let boards = BoardService::new(board_repo.clone());
        let columns = ColumnService::new(column_repo.clone());
        let cards = CardService::new(card_repo.clone());
        let board_view = BoardViewService::new(board_repo, column_repo, card_repo);
        Self {
            pool,
            projects,
            boards,
            columns,
            cards,
            board_view,
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
