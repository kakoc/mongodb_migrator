//! Describes a migration status
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum MigrationStatus {
    /// Migration which is running now
    InProgress,
    /// Migration was successfully completed
    Success,
    /// Migration was completed with an error
    Fail,
}

// impl std::fmt::Display for MigrationStatus {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             MigrationStatus::InProgress => write!(f, "In Progress"),
//             MigrationStatus::Success => write!(f, "Success"),
//             MigrationStatus::Fail => write!(f, "Fail"),
//         }
//     }
// }
