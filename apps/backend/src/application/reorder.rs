use std::collections::HashSet;
use std::hash::Hash;

use crate::application::error::ApplicationError;

pub fn ensure_permutation<Id>(existing: &[Id], provided: &[Id]) -> Result<(), ApplicationError>
where
    Id: Eq + Hash + Copy,
{
    let provided_set: HashSet<Id> = provided.iter().copied().collect();
    let existing_set: HashSet<Id> = existing.iter().copied().collect();
    if provided.len() != provided_set.len() || provided_set != existing_set {
        return Err(ApplicationError::Unprocessable(
            "orderedIds must be a permutation of the current ids".to_owned(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_reordering_of_the_same_ids() {
        assert!(ensure_permutation(&[1, 2, 3], &[3, 1, 2]).is_ok());
    }

    #[test]
    fn accepts_the_empty_case() {
        assert!(ensure_permutation::<i32>(&[], &[]).is_ok());
    }

    #[test]
    fn rejects_a_missing_or_unknown_id() {
        assert!(matches!(
            ensure_permutation(&[1, 2], &[1, 9]),
            Err(ApplicationError::Unprocessable(_))
        ));
    }

    #[test]
    fn rejects_duplicates() {
        assert!(matches!(
            ensure_permutation(&[1, 2], &[1, 1]),
            Err(ApplicationError::Unprocessable(_))
        ));
    }
}
