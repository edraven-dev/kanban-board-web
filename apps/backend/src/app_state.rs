use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn holds_a_cloneable_pool() {
        // `connect_lazy` validates the URL without opening a connection.
        let pool = PgPool::connect_lazy("postgres://kanban:kanban@localhost:5432/kanban").unwrap();
        let state = AppState::new(pool);
        let clone = state.clone();
        assert_eq!(state.pool.size(), clone.pool.size());
    }
}
