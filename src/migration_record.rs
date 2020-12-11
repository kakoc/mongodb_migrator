use chrono::DateTime;
use chrono::Utc;
use serde_derive::{Deserialize, Serialize};

use crate::migration_status::MigrationStatus;

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationRecord {
    pub start_date: chrono::DateTime<Utc>,
    pub end_date: Option<chrono::DateTime<Utc>>,
    pub status: MigrationStatus,
    pub duration: Option<i64>,
    pub name: String,
}

impl MigrationRecord {
    pub fn migration_start(migration_name: String) -> Self {
        MigrationRecord {
            start_date: Utc::now(),
            end_date: None,
            status: MigrationStatus::InProgress,
            duration: None,
            name: migration_name,
        }
    }

    pub fn migration_succeeded(self) -> Self {
        let end_date = Utc::now();

        MigrationRecord {
            end_date: Some(end_date),
            status: MigrationStatus::Succeeded,
            duration: Some(self.calc_migration_duration(end_date)),
            ..self
        }
    }

    pub fn migration_failed(self) -> Self {
        let end_date = Utc::now();

        MigrationRecord {
            end_date: Some(end_date),
            status: MigrationStatus::Failed,
            duration: Some(self.calc_migration_duration(end_date)),
            ..self
        }
    }

    fn calc_migration_duration(&self, end_date: DateTime<Utc>) -> i64 {
        (end_date.time() - self.start_date.time()).num_milliseconds()
    }
}
