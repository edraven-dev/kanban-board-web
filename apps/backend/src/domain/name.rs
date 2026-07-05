use crate::domain::error::DomainError;

pub const MAX_NAME_LEN: usize = 120;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityName(String);

impl EntityName {
    pub fn new(raw: impl Into<String>) -> Result<Self, DomainError> {
        let trimmed = raw.into().trim().to_owned();
        if trimmed.is_empty() {
            return Err(DomainError::EmptyName);
        }
        let len = trimmed.chars().count();
        if len > MAX_NAME_LEN {
            return Err(DomainError::NameTooLong {
                max: MAX_NAME_LEN,
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
    fn accepts_and_trims_a_valid_name() {
        let name = EntityName::new("  Backlog  ").unwrap();
        assert_eq!(name.as_str(), "Backlog");
        assert_eq!(name.into_string(), "Backlog");
    }

    #[test]
    fn rejects_an_empty_name() {
        assert_eq!(EntityName::new("").unwrap_err(), DomainError::EmptyName);
    }

    #[test]
    fn rejects_a_whitespace_only_name() {
        assert_eq!(
            EntityName::new("   \t\n ").unwrap_err(),
            DomainError::EmptyName
        );
        assert_eq!(
            EntityName::new("").unwrap_err().to_string(),
            "name must not be empty"
        );
    }

    #[test]
    fn accepts_a_name_at_the_maximum_length() {
        let name = "a".repeat(MAX_NAME_LEN);
        assert_eq!(EntityName::new(name.as_str()).unwrap().as_str(), name);
    }

    #[test]
    fn rejects_a_name_over_the_maximum_length() {
        let raw = "a".repeat(MAX_NAME_LEN + 1);
        let err = EntityName::new(raw.as_str()).unwrap_err();
        assert_eq!(
            err,
            DomainError::NameTooLong {
                max: MAX_NAME_LEN,
                actual: MAX_NAME_LEN + 1,
            },
        );
        assert_eq!(
            err.to_string(),
            "name must be at most 120 characters, got 121"
        );
    }

    #[test]
    fn counts_length_in_characters_not_bytes() {
        let name = "é".repeat(MAX_NAME_LEN);
        assert!(EntityName::new(name.as_str()).is_ok());
    }
}
