//! This crate provides a convinient way of how to manage migrations.  
//! It's inteded to be used as a library:
//! You maintain all migrations by your own via implementing [`migration::Migration`] trait  
//! and pass a sequence of migrations to the [`migrator::Migrator`] on every bootstrap of your system
//!
//! # Example
//!
//! ```
//! use anyhow::Result;
//! use async_trait::async_trait;
//! use mongodb::Database;
//! use serde_derive::{Deserialize, Serialize};
//! use testcontainers::Docker;
//!
//! use mongodb_migrator::{migration::Migration, migrator::Env};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let docker = testcontainers::clients::Cli::default();
//!     let node = docker.run(testcontainers::images::mongo::Mongo::default());
//!     let host_port = node.get_host_port(27017).unwrap();
//!     let url = format!("mongodb://localhost:{}/", host_port);
//!     let client = mongodb::Client::with_uri_str(url).await.unwrap();
//!     let db = client.database("test");
//!
//!     let migrations: Vec<Box<dyn Migration>> = vec![Box::new(M0 {}), Box::new(M1 {})];
//!     mongodb_migrator::migrator::default::DefaultMigrator::new()
//!         .with_conn(db.clone())
//!         .with_migrations_vec(migrations)
//!         .up()
//!         .await?;
//!
//!     Ok(())
//! }
//!
//! struct M0 {}
//! struct M1 {}
//!
//! #[async_trait]
//! impl Migration for M0 {
//!     async fn up(&self, env: Env) -> Result<()> {
//!         env.db.expect("db is available").collection("users")
//!             .insert_one(bson::doc! { "name": "Batman" }, None)
//!             .await?;
//!
//!         Ok(())
//!     }
//! }
//!
//! #[async_trait]
//! impl Migration for M1 {
//!     async fn up(&self, env: Env) -> Result<()> {
//!         env.db.expect("db is available").collection::<Users>("users")
//!             .update_one(
//!                 bson::doc! { "name": "Batman" },
//!                 bson::doc! { "$set": { "name": "Superman" } },
//!                 None,
//!             )
//!             .await?;
//!
//!         Ok(())
//!     }
//! }
//!
//! #[derive(Serialize, Deserialize)]
//! struct Users {
//!     name: String,
//! }
//! ```

pub mod error;
pub mod migration;
pub mod migration_record;
pub mod migration_status;
pub mod migrator;
pub mod server;
