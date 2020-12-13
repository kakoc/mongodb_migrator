use anyhow::Result;
use async_trait::async_trait;
use bson::{self, Bson};
use futures::stream::StreamExt;
use mongodb::{options::ClientOptions, Client, Database};
use mongodb_migrator::{
    migration::Migration, migration_record::MigrationRecord, migration_status::MigrationStatus,
};
use std::sync::{Arc, Mutex};
use testcontainers::{Container, Docker};
use tokio::{self, stream::Stream};

pub struct M0 {}
pub struct M1 {}
pub struct M2 {}
pub struct M3 {}

#[async_trait]
impl Migration for M0 {
    async fn up(&self, db: Database) -> Result<()> {
        db.collection("users")
            .insert_one(bson::doc! { "x": 0 }, None)
            .await?;

        Ok(())
    }

    fn git_id(&self) -> &str {
        "M0"
    }
}

#[async_trait]
impl Migration for M1 {
    async fn up(&self, db: Database) -> Result<()> {
        db.collection("users")
            .update_one(bson::doc! {"x": 0}, bson::doc! { "x": 1 }, None)
            .await?;

        Ok(())
    }

    fn git_id(&self) -> &str {
        "M1"
    }
}

#[async_trait]
impl Migration for M2 {
    async fn up(&self, db: Database) -> Result<()> {
        db.collection("users")
            .update_one(bson::doc! {"x": 1}, bson::doc! { "x": 2 }, None)
            .await?;

        Ok(())
    }

    fn git_id(&self) -> &str {
        "M2"
    }
}

#[async_trait]
impl Migration for M3 {
    async fn up(&self, db: Database) -> Result<()> {
        Err(anyhow::Error::msg("test error".to_string()))
    }

    fn git_id(&self) -> &str {
        "M3"
    }
}

#[tokio::test]
async fn migrations_ran_in_particular_order() {
    let docker = testcontainers::clients::Cli::default();
    let node = docker.run(testcontainers::images::mongo::Mongo::default());
    let host_port = node.get_host_port(27017).unwrap();
    let url = format!("mongodb://localhost:{}/", host_port);
    let client = mongodb::Client::with_uri_str(url.as_ref()).await.unwrap();
    let db = client.database("test");

    let migrations: Vec<Box<dyn Migration>> =
        vec![Box::new(M0 {}), Box::new(M1 {}), Box::new(M2 {})];
    let migrator = mongodb_migrator::migrator::DefaultMigrator::new()
        .with_conn(db.clone())
        .with_migrations_vec(migrations);
    migrator.up().await.unwrap();

    assert!(db
        .collection("users")
        .find_one(bson::doc! {"x": 2}, None)
        .await
        .unwrap()
        .is_some());
}

#[tokio::test]
async fn ran_migrations_saved_in_migrations_folder() {
    let docker = testcontainers::clients::Cli::default();
    let node = docker.run(testcontainers::images::mongo::Mongo::default());
    let host_port = node.get_host_port(27017).unwrap();
    let url = format!("mongodb://localhost:{}/", host_port);
    let client = mongodb::Client::with_uri_str(url.as_ref()).await.unwrap();
    let db = client.database("test");

    let migrations: Vec<Box<dyn Migration>> =
        vec![Box::new(M0 {}), Box::new(M1 {}), Box::new(M2 {})];
    let migrations_len = migrations.len();
    let migrator = mongodb_migrator::migrator::DefaultMigrator::new()
        .with_conn(db.clone())
        .with_migrations_vec(migrations);
    migrator.up().await.unwrap();

    assert_eq!(
        db.collection("migrations")
            .find(bson::doc! {}, None)
            .await
            .unwrap()
            .collect::<Vec<_>>()
            .await
            .len(),
        migrations_len
    );
}

#[tokio::test]
async fn all_ran_migrations_are_succeeded() {
    let docker = testcontainers::clients::Cli::default();
    let node = docker.run(testcontainers::images::mongo::Mongo::default());
    let host_port = node.get_host_port(27017).unwrap();
    let url = format!("mongodb://localhost:{}/", host_port);
    let client = mongodb::Client::with_uri_str(url.as_ref()).await.unwrap();
    let db = client.database("test");

    let migrations: Vec<Box<dyn Migration>> =
        vec![Box::new(M0 {}), Box::new(M1 {}), Box::new(M2 {})];
    let migrations_len = migrations.len();
    let migrator = mongodb_migrator::migrator::DefaultMigrator::new()
        .with_conn(db.clone())
        .with_migrations_vec(migrations);
    migrator.up().await.unwrap();

    assert_eq!(
        db.collection("migrations")
            .find(bson::doc! {}, None)
            .await
            .unwrap()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(|v| bson::from_bson(Bson::Document(v.unwrap())).unwrap())
            .collect::<Vec<MigrationRecord>>()
            .into_iter()
            .filter(|v| {
                match v.status {
                    MigrationStatus::Succeeded => true,
                    _ => false,
                }
            })
            .collect::<Vec<MigrationRecord>>()
            .len(),
        migrations_len
    );
}

#[tokio::test]
async fn with_failed_migration_should_stop_after_first_fail() {
    let docker = testcontainers::clients::Cli::default();
    let node = docker.run(testcontainers::images::mongo::Mongo::default());
    let host_port = node.get_host_port(27017).unwrap();
    let url = format!("mongodb://localhost:{}/", host_port);
    let client = mongodb::Client::with_uri_str(url.as_ref()).await.unwrap();
    let db = client.database("test");

    let migrations: Vec<Box<dyn Migration>> = vec![
        Box::new(M0 {}),
        Box::new(M3 {}),
        Box::new(M1 {}),
        Box::new(M2 {}),
        Box::new(M3 {}),
    ];
    let migrations_len = migrations.len();
    let migrator = mongodb_migrator::migrator::DefaultMigrator::new()
        .with_conn(db.clone())
        .with_migrations_vec(migrations);
    migrator.up().await;

    assert_eq!(
        db.collection("migrations")
            .find(bson::doc! {}, None)
            .await
            .unwrap()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(|v| bson::from_bson(Bson::Document(v.unwrap())).unwrap())
            .collect::<Vec<MigrationRecord>>()
            .into_iter()
            .filter(|v| {
                match v.status {
                    MigrationStatus::Failed => true,
                    _ => false,
                }
            })
            .collect::<Vec<MigrationRecord>>()
            .len(),
        1
    );

    assert_eq!(
        db.collection("migrations")
            .find(bson::doc! {}, None)
            .await
            .unwrap()
            .collect::<Vec<_>>()
            .await
            .len(),
        2
    );
}

// println!(
//     "{:?}",
//     db.collection("migrations")
//         .find(bson::doc! {}, None)
//         .await
//         .unwrap()
//         .collect::<Vec<_>>()
//         .await
//         .into_iter()
//         .map(|v| bson::from_bson(Bson::Document(v.unwrap())).unwrap())
//         .collect::<Vec<MigrationRecord>>()
// );

// assert!(!insert_one_result
//     .inserted_id
//     .as_object_id()
//     .unwrap()
//     .to_hex()
//     .is_empty());

// let find_one_result: bson::Document = coll
//     .find_one(bson::doc! { "x": 42 }, None)
//     .await
//     .unwrap()
//     .unwrap();
// assert_eq!(42, find_one_result.get_i32("x").unwrap())
