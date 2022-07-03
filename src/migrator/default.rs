use mongodb::Database;

use super::with_connection::WithConnection;

pub struct DefaultMigrator {}

impl DefaultMigrator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn with_conn(self, db: Database) -> WithConnection {
        WithConnection { db }
    }
}
