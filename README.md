[![Build Status](https://travis-ci.com/kakoc/mongodb_migrator.svg?token=x6zhjaVWsFLJA2pDjgQT&branch=main)](https://travis-ci.com/kakoc/mongodb_migrator)

Mongodb migrations management tool.

## How to use

```rust
use anyhow::Result;
use async_trait::async_trait;
use mongodb::Database;

use mongodb_migrator::migration::Migration;

#[tokio::main]
async fn main() -> Result<()> {
    let db = mongodb::Client::with_uri_str("mongodb://localhost:27017")
        .await?
        .database("test");

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
            .insert_one(bson::doc! { "x": 0 }, None)
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
        db.collection("users")
            .update_one(bson::doc! {"x": 0}, bson::doc! { "x": 1 }, None)
            .await?;

        Ok(())
    }

    fn get_id(&self) -> &str {
        "M1"
    }
}
```
