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
