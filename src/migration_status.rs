//! Describes a migration status
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum MigrationStatus {
    /// Migration which is running now
    InProgress,
    /// Migration was successfully completed
    Succeeded,
    /// Migration was completed with an error
    Failed,
}

impl std::fmt::Display for MigrationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationStatus::InProgress => write!(f, "In Progress"),
            MigrationStatus::Succeeded => write!(f, "Succeeded"),
            MigrationStatus::Failed => write!(f, "Failed"),
        }
    }
}
