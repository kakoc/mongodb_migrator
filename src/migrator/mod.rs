//! Migrator runs passed migrations - entities which implement [`Migration`] trait
pub mod default;
pub mod shell;
pub mod with_connection;
pub mod with_migrations_vec;
pub mod with_shell_config;

use mongodb::Database;

use self::{
    default::DefaultMigrator, shell::Shell, with_connection::WithConnection,
    with_migrations_vec::WithMigrationsVec, with_shell_config::WithShellConfig,
};

pub enum Migrator {
    DefaultMigrator(DefaultMigrator),
    WithConnection(WithConnection),
    WithMigrationsVec(WithMigrationsVec),
    WithShellConfig(WithShellConfig),
}

#[derive(Clone)]
pub struct Env {
    pub db: Option<Database>,
    pub shell: Option<Shell>,
}

impl Default for Env {
    fn default() -> Self {
        Self {
            db: None,
            shell: None,
        }
    }
}
