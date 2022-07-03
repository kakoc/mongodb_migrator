//! In order to treat the entity as migrationable it should implement [`Migration`] trait
use anyhow::Result;
use async_trait::async_trait;

use crate::migrator::Env;

#[async_trait]
pub trait Migration: Sync {
    /// Runs a migration.
    async fn up(&self, env: Env) -> Result<()>;

    /// Rollbacks a migration.
    /// Since not every migration could be rollbacked
    /// and it often happens there is a blank implementation
    async fn down(&self, _env: Env) -> Result<()> {
        Ok(())
    }

    /// A status about a migration will be stored in a db collection with the following document id
    fn get_id(&self) -> &str;
}
