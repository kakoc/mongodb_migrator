use std::time::Duration;

use super::{
    shell::ShellConfig,
    with_migrations_vec::WithMigrationsVec,
    with_retries::{Retry, WithRetries},
    with_shell_config::WithShellConfig,
};
use crate::migration::Migration;

#[derive(Clone)]
pub struct WithConnection {
    pub db: mongodb::Database,
}

impl WithConnection {
    pub fn with_migrations_vec(self, migrations: Vec<Box<dyn Migration>>) -> WithMigrationsVec {
        WithMigrationsVec {
            migrations,
            with_connection: self,
            with_shell_config: None,
            with_retries_per_migration: Default::default(),
        }
    }

    pub fn with_shell_config(self, with_shell_config: ShellConfig) -> WithShellConfig {
        WithShellConfig {
            with_shell_config,
            with_connection: self,
        }
    }

    pub fn with_retries(
        self,
        retries_per_migration_count: usize,
        retry_delay: Duration,
    ) -> WithRetries {
        WithRetries {
            with_connection: self,
            with_retries_per_migration: Retry {
                count: retries_per_migration_count,
                delay: retry_delay,
            },
        }
    }
}
