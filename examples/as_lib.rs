use anyhow::Result;
use async_trait::async_trait;
use serde_derive::{Deserialize, Serialize};

use mongodb_migrator::{migration::Migration, migrator::Env};

#[tokio::main]
async fn main() {
    let docker = testcontainers::clients::Cli::default();
    let node = docker.run(testcontainers::images::mongo::Mongo);
    let host_port = node.get_host_port_ipv4(27017);
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
