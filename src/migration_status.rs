use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum MigrationStatus {
    InProgress,
    Succeeded,
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
