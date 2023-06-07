//! [`MigrationRecord`] describes the document which will be stored
//! in the migrations collection.  
//! It contains all useful attributes which might be used in order
//! to understand the current state of a particular migration

use chrono::DateTime;
use chrono::Utc;
use serde_derive::{Deserialize, Serialize};

use crate::migration_status::MigrationStatus;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct MigrationRecord {
    pub _id: String,
    pub start_date: Option<chrono::DateTime<Utc>>,
    pub end_date: Option<chrono::DateTime<Utc>>,
    pub status: MigrationStatus,
    pub duration: Option<i64>,
}

impl MigrationRecord {
    pub fn migration_start(migration_name: String) -> Self {
        MigrationRecord {
            _id: migration_name,
            start_date: Some(Utc::now()),
            end_date: None,
            status: MigrationStatus::InProgress,
            duration: None,
        }
    }

    pub fn migration_succeeded(self) -> Self {
        let end_date = Utc::now();

        MigrationRecord {
            end_date: Some(end_date),
            status: MigrationStatus::Success,
            duration: Some(self.calc_migration_duration(end_date)),
            ..self
        }
    }

    pub fn migration_failed(self) -> Self {
        let end_date = Utc::now();

        MigrationRecord {
            end_date: Some(end_date),
            status: MigrationStatus::Fail,
            duration: Some(self.calc_migration_duration(end_date)),
            ..self
        }
    }

    fn calc_migration_duration(&self, end_date: DateTime<Utc>) -> i64 {
        if self.start_date.is_none() {
            0
        } else {
            (end_date.time() - self.start_date.unwrap().time()).num_milliseconds()
        }
    }
}
