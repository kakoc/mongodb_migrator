use anyhow::Result;
use async_trait::async_trait;
use bson::Bson;
use futures::stream::StreamExt;
use serde_derive::{Deserialize, Serialize};
use testcontainers_modules::{mongo::Mongo, testcontainers::ContainerAsync};

use mongodb_migrator::{
    migration::Migration, migration_record::MigrationRecord, migration_status::MigrationStatus,
    migrator::Env,
};

pub async fn basic(node: &ContainerAsync<Mongo>) {
    let host_port = node.get_host_port_ipv4(27017).await.unwrap();
    let url = format!("mongodb://localhost:{}/", host_port);
    let client = mongodb::Client::with_uri_str(url).await.unwrap();
    let db = client.database("test");

    let migrations: Vec<Box<dyn Migration>> = vec![Box::new(M0 {}), Box::new(M1 {})];
    mongodb_migrator::migrator::default::DefaultMigrator::new()
        .with_conn(db.clone())
        .with_migrations_vec(migrations)
        .up()
        .await
        .unwrap();

    assert!(db
        .collection::<Users>("users")
        .find_one(bson::doc! {"name": "Superman"}, None)
        .await
        .unwrap()
        .is_some());
}

struct M0 {}
struct M1 {}

#[async_trait]
impl Migration for M0 {
    async fn up(&self, env: Env) -> Result<()> {
        env.db
            .expect("db is available")
            .collection("users")
            .insert_one(bson::doc! { "name": "Batman" }, None)
            .await?;

        Ok(())
    }
}

#[async_trait]
impl Migration for M1 {
    async fn up(&self, env: Env) -> Result<()> {
        env.db
            .expect("db is available")
            .collection::<Users>("users")
            .update_one(
                bson::doc! { "name": "Batman" },
                bson::doc! { "$set": { "name": "Superman" } },
                None,
            )
            .await?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct Users {
    name: String,
}

pub async fn custom_collection_name(node: &ContainerAsync<Mongo>) {
    let host_port = node.get_host_port_ipv4(27017).await.unwrap();
    let url = format!("mongodb://localhost:{}/", host_port);
    let client = mongodb::Client::with_uri_str(url).await.unwrap();
    let db = client.database("test");

    struct M0 {}
    #[async_trait]
    impl Migration for M0 {
        async fn up(&self, _env: Env) -> Result<()> {
            Ok(())
        }
    }
    let migrations: Vec<Box<dyn Migration>> = vec![Box::new(M0 {})];

    mongodb_migrator::migrator::default::DefaultMigrator::new()
        .with_conn(db.clone())
        .with_migrations_vec(migrations)
        .set_collection_name("foo")
        .up()
        .await
        .unwrap();

    let ms = db
        .collection("foo")
        .find(bson::doc! {}, None)
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|v| bson::from_bson::<MigrationRecord>(Bson::Document(v.unwrap())).unwrap())
        .collect::<Vec<MigrationRecord>>();

    assert_eq!(ms.len(), 1);
    assert_eq!(ms[0].status, MigrationStatus::Success);
}
