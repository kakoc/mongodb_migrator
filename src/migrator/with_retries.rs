//! With this type of the migrator it's possible to try run failed migrations multiple times
use std::time::Duration;

use super::{with_connection::WithConnection, with_migrations_vec::WithMigrationsVec};
use crate::migration::Migration;

#[derive(Clone)]
pub struct WithRetries {
    pub with_connection: WithConnection,
    pub with_retries_per_migration: Retry,
}

#[derive(Clone, Default)]
pub struct Retry {
    pub count: usize,
    pub delay: Duration,
}

impl WithRetries {
    pub fn with_migrations_vec(self, migrations: Vec<Box<dyn Migration>>) -> WithMigrationsVec {
        WithMigrationsVec {
            migrations,
            // TODO(kakoc): rework forwarding: merge? split? -clone?
            with_shell_config: None,
            with_connection: self.with_connection,
            with_retries_per_migration: self.with_retries_per_migration,
            collection_name: None,
        }
    }
}
