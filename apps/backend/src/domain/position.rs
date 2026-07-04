use crate::domain::error::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position(i32);

impl Position {
    pub fn new(value: i32) -> Result<Self, DomainError> {
        if value < 0 {
            return Err(DomainError::NegativePosition(value));
        }
        Ok(Self(value))
    }

    pub fn value(self) -> i32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_zero_and_positive_values() {
        assert_eq!(Position::new(0).unwrap().value(), 0);
        assert_eq!(Position::new(42).unwrap().value(), 42);
    }

    #[test]
    fn orders_by_value() {
        assert!(Position::new(1).unwrap() < Position::new(2).unwrap());
    }

    #[test]
    fn rejects_negative_values() {
        let err = Position::new(-1).unwrap_err();
        assert_eq!(err, DomainError::NegativePosition(-1));
        assert_eq!(err.to_string(), "position must not be negative, got -1");
    }
}
