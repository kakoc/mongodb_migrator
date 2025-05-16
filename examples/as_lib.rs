use anyhow::Result;
use async_trait::async_trait;
use serde_derive::{Deserialize, Serialize};

use mongodb_migrator::{migration::Migration, migrator::Env};

use testcontainers_modules::{mongo::Mongo, testcontainers::runners::AsyncRunner};

#[tokio::main]
async fn main() {
    let node = Mongo::default().start().await.unwrap();
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
}

struct M0 {}
struct M1 {}

#[async_trait]
impl Migration for M0 {
    async fn up(&self, env: Env) -> Result<()> {
        env.db
            .expect("db is available")
            .collection("users")
            .insert_one(bson::doc! { "name": "Batman" })
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
            )
            .await?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct Users {
    name: String,
}
