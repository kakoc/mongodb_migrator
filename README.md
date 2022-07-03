[<img alt="github" src="https://img.shields.io/badge/github-kakoc/mongodb_migrator?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/kakoc/mongodb_migrator)
[<img alt="crates.io" src="https://img.shields.io/crates/v/mongodb-migrator.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/syn)
[<img alt="build status" src="https://img.shields.io/travis/com/kakoc/mongodb_migrator?style=for-the-badge" height="20">](https://travis-ci.com/kakoc/mongodb_migrator)


<!-- [![Build Status](https://travis-ci.com/kakoc/mongodb_migrator.svg?token=x6zhjaVWsFLJA2pDjgQT&branch=main)](https://travis-ci.com/kakoc/mongodb_migrator) -->

Mongodb migrations management tool.

## Setup

```toml
[dependencies]
mongodb-migrator = "0.1.1"
```

## Functionality
- [Execute Rust based migrations][1]
- [Execute JavaScript based migrations][2]

[1]: https://github.com/kakoc/mongodb_migrator/blob/main/examples/as_lib.rs
[2]: https://github.com/kakoc/mongodb_migrator/blob/main/tests/shell.rs

## How to use

### Rust based migrations
```rust
use anyhow::Result;
use async_trait::async_trait;
use mongodb::Database;
use serde_derive::{Deserialize, Serialize};
use testcontainers::Docker;

use mongodb_migrator::migration::Migration;

#[tokio::main]
async fn main() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let node = docker.run(testcontainers::images::mongo::Mongo::default());
    let host_port = node.get_host_port(27017).unwrap();
    let url = format!("mongodb://localhost:{}/", host_port);
    let client = mongodb::Client::with_uri_str(url).await.unwrap();
    let db = client.database("test");

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
            .insert_one(bson::doc! { "name": "Batman" }, None)
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
            .update_one(
                bson::doc! { "name": "Batman" },
                bson::doc! { "$set": { "name": "Superman" } },
                None,
            )
            .await?;

        Ok(())
    }

    fn get_id(&self) -> &str {
        "M1"
    }
}

#[derive(Serialize, Deserialize)]
struct Users {
    name: String,
}
```
