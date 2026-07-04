use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error("name must not be empty")]
    EmptyName,
    #[error("name must be at most {max} characters, got {actual}")]
    NameTooLong { max: usize, actual: usize },
    #[error("title must not be empty")]
    EmptyTitle,
    #[error("title must be at most {max} characters, got {actual}")]
    TitleTooLong { max: usize, actual: usize },
    #[error("description must be at most {max} characters, got {actual}")]
    DescriptionTooLong { max: usize, actual: usize },
    #[error("position must not be negative, got {0}")]
    NegativePosition(i32),
}
