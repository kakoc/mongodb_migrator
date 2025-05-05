use anyhow::Result;
use async_trait::async_trait;
use mongodb::Database;
use mongodb_migrator::{
    migration::Migration,
    migrator::{shell::ShellConfig, Env},
};
use serde_derive::{Deserialize, Serialize};
use testcontainers_modules::{
    mongo::Mongo,
    testcontainers::{runners::AsyncRunner, ContainerAsync},
};

pub struct TestDb {
    pub node: ContainerAsync<Mongo>,
    pub db: Database,
}

impl TestDb {
    pub async fn new() -> TestDb {
        let node = Mongo::default().start().await.unwrap();
        let host_port = node.get_host_port_ipv4(27017).await.unwrap();
        let url = format!("mongodb://localhost:{}/", host_port);
        let client = mongodb::Client::with_uri_str(url).await.unwrap();
        let db = client.database("test");

        Self { node, db }
    }
}

#[allow(dead_code)]
pub fn init_migrator_with_migrations(
    db: Database,
    migrations: Vec<Box<dyn Migration>>,
) -> mongodb_migrator::migrator::with_migrations_vec::WithMigrationsVec {
    mongodb_migrator::migrator::default::DefaultMigrator::new()
        .with_conn(db)
        .with_migrations_vec(migrations)
}

#[allow(dead_code)]
pub fn init_shell_migrator_with_migrations(
    db: Database,
    shell_config: ShellConfig,
    migrations: Vec<Box<dyn Migration>>,
) -> mongodb_migrator::migrator::with_migrations_vec::WithMigrationsVec {
    mongodb_migrator::migrator::default::DefaultMigrator::new()
        .with_conn(db)
        .with_shell_config(shell_config)
        .with_migrations_vec(migrations)
}

pub struct M0 {}
pub struct M1 {}
pub struct M2 {}
pub struct M3 {}

#[async_trait]
impl Migration for M0 {
    async fn up(&self, env: Env) -> Result<()> {
        env.db
            .expect("db is available")
            .collection("users")
            .insert_one(bson::doc! { "x": 0 }, None)
            .await?;

        Ok(())
    }

    async fn down(&self, env: Env) -> Result<()> {
        M2 {}.up(env).await?;

        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Users {
    x: usize,
}

#[async_trait]
impl Migration for M1 {
    async fn up(&self, env: Env) -> Result<()> {
        env.db
            .expect("db is available")
            .collection::<Users>("users")
            .update_one(bson::doc! {"x": 0}, bson::doc! {"$set": {"x": 1} }, None)
            .await?;

        Ok(())
    }

    async fn down(&self, env: Env) -> Result<()> {
        M1 {}.up(env).await?;

        Ok(())
    }
}

#[async_trait]
impl Migration for M2 {
    async fn up(&self, env: Env) -> Result<()> {
        env.db
            .expect("db is available")
            .collection::<Users>("users")
            .update_one(bson::doc! {"x": 1}, bson::doc! {"$set": {"x": 2} }, None)
            .await?;

        Ok(())
    }

    async fn down(&self, env: Env) -> Result<()> {
        M0 {}.up(env).await?;

        Ok(())
    }
}

#[async_trait]
impl Migration for M3 {
    async fn up(&self, _env: Env) -> Result<()> {
        Err(anyhow::Error::msg("test error".to_string()))
    }
}
