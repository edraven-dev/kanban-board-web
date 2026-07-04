use crate::domain::error::DomainError;

pub const MAX_TITLE_LEN: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Title(String);

impl Title {
    pub fn new(raw: impl Into<String>) -> Result<Self, DomainError> {
        let trimmed = raw.into().trim().to_owned();
        if trimmed.is_empty() {
            return Err(DomainError::EmptyTitle);
        }
        let len = trimmed.chars().count();
        if len > MAX_TITLE_LEN {
            return Err(DomainError::TitleTooLong {
                max: MAX_TITLE_LEN,
                actual: len,
            });
        }
        Ok(Self(trimmed))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_and_trims_a_valid_title() {
        let title = Title::new("  Ship the release  ").unwrap();
        assert_eq!(title.as_str(), "Ship the release");
        assert_eq!(title.into_string(), "Ship the release");
    }

    #[test]
    fn rejects_an_empty_title() {
        assert_eq!(Title::new("   ").unwrap_err(), DomainError::EmptyTitle);
        assert_eq!(Title::new("").unwrap_err().to_string(), "title must not be empty");
    }

    #[test]
    fn accepts_a_title_at_the_maximum_length() {
        let title = "a".repeat(MAX_TITLE_LEN);
        assert_eq!(Title::new(title.as_str()).unwrap().as_str(), title);
    }

    #[test]
    fn rejects_a_title_over_the_maximum_length() {
        let raw = "a".repeat(MAX_TITLE_LEN + 1);
        let err = Title::new(raw.as_str()).unwrap_err();
        assert_eq!(
            err,
            DomainError::TitleTooLong {
                max: MAX_TITLE_LEN,
                actual: MAX_TITLE_LEN + 1,
            },
        );
        assert_eq!(err.to_string(), "title must be at most 200 characters, got 201");
    }
}
