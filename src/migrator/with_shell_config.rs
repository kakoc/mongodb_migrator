//! With this type of the migrator it's possible write JavaScript based migrations
//! and run them via mongo shell(--eval flag)
use super::{
    shell::ShellConfig, with_connection::WithConnection, with_migrations_vec::WithMigrationsVec,
};
use crate::migration::Migration;

#[derive(Clone)]
pub struct WithShellConfig {
    pub with_shell_config: ShellConfig,
    pub with_connection: WithConnection,
}

impl WithShellConfig {
    pub fn with_migrations_vec(self, migrations: Vec<Box<dyn Migration>>) -> WithMigrationsVec {
        WithMigrationsVec {
            migrations,
            // TODO(kakoc): rework forwarding: merge? split? -clone?
            with_shell_config: Some(self.clone()),
            with_connection: self.with_connection,
            with_retries_per_migration: Default::default(),
            collection_name: None,
        }
    }
}
