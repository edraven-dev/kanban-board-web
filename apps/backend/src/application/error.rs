use thiserror::Error;

use crate::domain::error::DomainError;
use crate::domain::ports::RepositoryError;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error("not found")]
    NotFound,
    #[error("limit exceeded")]
    LimitExceeded,
    #[error("{0}")]
    Unprocessable(String),
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_a_domain_error() {
        let err: ApplicationError = DomainError::EmptyName.into();
        assert!(matches!(err, ApplicationError::Domain(DomainError::EmptyName)));
        assert_eq!(err.to_string(), "name must not be empty");
    }

    #[test]
    fn wraps_a_repository_error() {
        let err: ApplicationError = RepositoryError::new("boom").into();
        assert!(matches!(err, ApplicationError::Repository(_)));
        assert_eq!(err.to_string(), "repository error: boom");
    }

    #[test]
    fn not_found_and_limit_exceeded_have_messages() {
        assert_eq!(ApplicationError::NotFound.to_string(), "not found");
        assert_eq!(ApplicationError::LimitExceeded.to_string(), "limit exceeded");
    }
}
