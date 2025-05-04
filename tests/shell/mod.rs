use anyhow::Result;
use async_trait::async_trait;
use mongodb_migrator::{
    migration::Migration,
    migrator::{shell::ShellConfig, Env},
};
use serde_derive::{Deserialize, Serialize};

use super::utils::{init_shell_migrator_with_migrations, TestDb};

pub async fn shell(t: &TestDb<'_>) {
    let host_port = t.node.get_host_port_ipv4(27017);
    let shell_config = ShellConfig {
        port: host_port as usize,
        ..Default::default()
    };
    let migrations: Vec<Box<dyn Migration>> = vec![Box::new(M0 {}), Box::new(M1 {})];

    init_shell_migrator_with_migrations(t.db.clone(), shell_config, migrations)
        .up()
        .await
        .unwrap();

    assert!(t
        .db
        .collection::<Users>("users")
        .find_one(bson::doc! {"name": "Superman"}, None)
        .await
        .unwrap()
        .is_some());
}

pub struct M0 {}
pub struct M1 {}

#[async_trait]
impl Migration for M0 {
    async fn up(&self, env: Env) -> Result<()> {
        env.shell.expect("shell is available").execute(
            "test",
            "db.getCollection('users').insertOne({name: 'Batman'});",
        )?;

        Ok(())
    }
}

#[async_trait]
impl Migration for M1 {
    async fn up(&self, env: Env) -> Result<()> {
        env.shell.expect("shell is available").execute(
            "test",
            "db.getCollection('users').updateOne({name: 'Batman'}, {$set: {name: 'Superman'}});",
        )?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct Users {
    name: String,
}
