use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! typed_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }

            pub fn from_uuid(id: Uuid) -> Self {
                Self(id)
            }

            pub fn as_uuid(self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl From<Uuid> for $name {
            fn from(id: Uuid) -> Self {
                Self(id)
            }
        }
    };
}

typed_id!(ProjectId);
typed_id!(BoardId);
typed_id!(ColumnId);
typed_id!(CardId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ids_are_unique_and_time_ordered() {
        let first = ProjectId::new();
        let second = ProjectId::new();
        assert_ne!(first, second);
        assert!(second.as_uuid() > first.as_uuid());
    }

    #[test]
    fn round_trips_through_a_uuid() {
        let raw = Uuid::now_v7();
        let id = BoardId::from_uuid(raw);
        assert_eq!(id.as_uuid(), raw);
        assert_eq!(ColumnId::from(raw).as_uuid(), raw);
    }

    #[test]
    fn default_mints_a_fresh_id() {
        assert_ne!(CardId::default(), CardId::default());
    }

    #[test]
    fn display_matches_the_inner_uuid() {
        let raw = Uuid::now_v7();
        assert_eq!(ProjectId::from_uuid(raw).to_string(), raw.to_string());
    }
}
