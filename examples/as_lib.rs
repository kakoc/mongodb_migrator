use anyhow::Result;
use async_trait::async_trait;
use mongodb::Database;
use testcontainers::Docker;

use mongodb_migrator::migration::Migration;

#[tokio::main]
async fn main() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let node = docker.run(testcontainers::images::mongo::Mongo::default());
    let host_port = node.get_host_port(27017).unwrap();
    let url = format!("mongodb://localhost:{}/", host_port);
    let db = mongodb::Client::with_uri_str(&url).await?.database("test");

    let migrations: Vec<Box<dyn Migration>> = vec![Box::new(M0 {}), Box::new(M1 {})];
    mongodb_migrator::migrator::DefaultMigrator::new()
        .with_conn(db.clone())
        .with_migrations_vec(migrations)
        .up()
        .await?;

    Ok(())
}

struct M0 {}
struct M1 {}

#[async_trait]
impl Migration for M0 {
    async fn up(&self, db: Database) -> Result<()> {
        db.collection("users")
            .insert_one(bson::doc! { "name": 0 }, None)
            .await?;

        Ok(())
    }

    fn get_id(&self) -> &str {
        "M0"
    }
}

#[async_trait]
impl Migration for M1 {
    async fn up(&self, db: Database) -> Result<()> {
        db.collection::<Users>("users")
            .update_one(bson::doc! {"name": 0}, bson::doc! { "name": 1 }, None)
            .await?;

        Ok(())
    }

    fn get_id(&self) -> &str {
        "M1"
    }
}

struct Users {
    _name: usize,
}
