//! Newtype ID wrappers for core entities. Using i64 (SQLite rowid) for fast graph joins.

use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(pub i64);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<i64> for $name {
            fn from(id: i64) -> Self {
                Self(id)
            }
        }
    };
}

define_id!(FileId);
define_id!(SymbolId);
define_id!(EdgeId);
define_id!(CommunityId);
define_id!(ChapterId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_display() {
        assert_eq!(FileId(42).to_string(), "42");
        assert_eq!(SymbolId(1).to_string(), "1");
    }

    #[test]
    fn id_serde_roundtrip() {
        let id = FileId(99);
        let json = serde_json::to_string(&id).unwrap();
        let back: FileId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn id_from_i64() {
        let id: FileId = 7.into();
        assert_eq!(id.0, 7);
    }
}
