pub mod pg_board_repo;
pub mod pg_project_repo;
pub mod rows;

use crate::domain::ports::RepositoryError;

impl From<sqlx::Error> for RepositoryError {
    fn from(error: sqlx::Error) -> Self {
        Self::new(error.to_string())
    }
}
