use super::{
    shell::ShellConfig, with_connection_and_migrations_vec::WithConnectionAndMigrationsVec,
    with_shell_config::WithShellConfig,
};
use crate::migration::Migration;

#[derive(Clone)]
pub struct WithConnection {
    pub db: mongodb::Database,
}

impl WithConnection {
    pub fn with_migrations_vec(
        self,
        migrations: Vec<Box<dyn Migration>>,
    ) -> WithConnectionAndMigrationsVec {
        WithConnectionAndMigrationsVec {
            migrations,
            with_connection: self,
            with_shell_config: None,
        }
    }

    pub fn with_shell_config(self, with_shell_config: ShellConfig) -> WithShellConfig {
        WithShellConfig {
            with_shell_config,
            with_connection: self,
        }
    }
}
