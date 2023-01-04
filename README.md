[<img alt="github" src="https://img.shields.io/badge/github-kakoc/mongodb_migrator?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/kakoc/mongodb_migrator)
[<img alt="crates.io" src="https://img.shields.io/crates/v/mongodb-migrator.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/mongodb-migrator)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logoColor=white&logo=docs.rs" height="20">](https://docs.rs/mongodb-migrator/latest/mongodb_migrator)
[<img alt="build status" src="https://img.shields.io/travis/com/kakoc/mongodb_migrator?style=for-the-badge" height="20">](https://travis-ci.com/kakoc/mongodb_migrator)
[![codecov](https://codecov.io/gh/kakoc/mongodb_migrator/branch/main/graph/badge.svg)](https://codecov.io/gh/kakoc/mongodb_migrator)

Mongodb migrations management tool.

## Setup

```toml
[dependencies]
mongodb-migrator = "0.1.7"
```

## Functionality
- [Execute Rust based migrations][1]
- [Execute JavaScript based migrations][2]
- [Run as RESTful service][3]

[1]: https://github.com/kakoc/mongodb_migrator/blob/main/examples/as_lib.rs
[2]: https://github.com/kakoc/mongodb_migrator/blob/main/tests/shell.rs
[3]: https://github.com/kakoc/mongodb_migrator/blob/main/tests/server/mod.rs

## How to use

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
}

#[derive(Serialize, Deserialize)]
struct Users {
    name: String,
}
```

## Roadmap

- [x] Rust based migrations
- [x] JavaScript based migrations
- [ ] Logging
- [x] Rollbacks
- [ ] Cli tool
- [ ] UI dashboard
- [x] RESTful service
- [ ] As npm package
- [ ] Stragegies
	- [ ] Fail first
	- [ ] Try all



