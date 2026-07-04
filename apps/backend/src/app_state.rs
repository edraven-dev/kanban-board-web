use std::sync::Arc;

use sqlx::PgPool;

use crate::adapters::outbound::persistence::pg_project_repo::PgProjectRepo;
use crate::application::project_service::ProjectService;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub projects: ProjectService,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        let projects = ProjectService::new(Arc::new(PgProjectRepo::new(pool.clone())));
        Self { pool, projects }
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
