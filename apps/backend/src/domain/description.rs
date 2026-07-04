use crate::domain::error::DomainError;

pub const MAX_DESCRIPTION_LEN: usize = 10_000;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Description(String);

impl Description {
    pub fn new(raw: impl Into<String>) -> Result<Self, DomainError> {
        let text = raw.into();
        let len = text.chars().count();
        if len > MAX_DESCRIPTION_LEN {
            return Err(DomainError::DescriptionTooLong {
                max: MAX_DESCRIPTION_LEN,
                actual: len,
            });
        }
        Ok(Self(text))
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
    fn accepts_an_empty_description() {
        assert_eq!(Description::new("").unwrap().as_str(), "");
        assert_eq!(Description::default().as_str(), "");
    }

    #[test]
    fn preserves_content_and_whitespace() {
        let text = "  line one\n  line two  ";
        assert_eq!(Description::new(text).unwrap().into_string(), text);
    }

    #[test]
    fn accepts_a_description_at_the_maximum_length() {
        let text = "x".repeat(MAX_DESCRIPTION_LEN);
        assert!(Description::new(text).is_ok());
    }

    #[test]
    fn rejects_a_description_over_the_maximum_length() {
        let err = Description::new("x".repeat(MAX_DESCRIPTION_LEN + 1)).unwrap_err();
        assert_eq!(
            err,
            DomainError::DescriptionTooLong {
                max: MAX_DESCRIPTION_LEN,
                actual: MAX_DESCRIPTION_LEN + 1,
            },
        );
        assert_eq!(
            err.to_string(),
            "description must be at most 10000 characters, got 10001",
        );
    }
}
