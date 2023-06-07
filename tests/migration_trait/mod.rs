//! These tests check whether passed migrations doesn't contain duplicates
use anyhow::Result;
use async_trait::async_trait;
use mongodb_migrator::{migration::Migration, migrator::Env};

struct M0 {}

#[async_trait]
impl Migration for M0 {
    async fn up(&self, _db: Env) -> Result<()> {
        Ok(())
    }
}

pub fn migration_id_autoderived() {
    let m = M0 {};

    assert_eq!("M0", m.get_id());
}
