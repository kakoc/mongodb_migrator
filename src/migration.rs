//! In order to treat the entity as migrationable it should implement [`Migration`] trait
use anyhow::Result;
use async_trait::async_trait;
use mongodb::Database;

#[async_trait]
pub trait Migration: Sync {
    async fn up(&self, db: Database) -> Result<()>;
    fn git_id(&self) -> &str;
}
