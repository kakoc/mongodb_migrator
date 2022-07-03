use anyhow::Result;
use async_trait::async_trait;
use mongodb::Database;
use serde_derive::{Deserialize, Serialize};
use testcontainers::Docker;

use mongodb_migrator::{migration::Migration, migrator::Env};

#[tokio::test]
async fn example() {
    let docker = testcontainers::clients::Cli::default();
    let node = docker.run(testcontainers::images::mongo::Mongo::default());
    let host_port = node.get_host_port(27017).unwrap();
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

    test_assert(db).await;
    write_to_examples();
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

    fn get_id(&self) -> &str {
        "M0"
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

    fn get_id(&self) -> &str {
        "M1"
    }
}

#[derive(Serialize, Deserialize)]
struct Users {
    name: String,
}

async fn test_assert(db: Database) {
    assert!(db
        .collection::<Users>("users")
        .find_one(bson::doc! {"name": "Superman"}, None)
        .await
        .unwrap()
        .is_some());
}

fn write_to_examples() {
    let example = std::fs::read_to_string("./tests/example.rs").expect("file with example present");
    let mut cleaned_example = String::new();
    for mut line in example.lines() {
        if line.starts_with("#[tokio::test]") {
            line = "#[tokio::main]";
        }

        if line.contains("async fn example") {
            line = "async fn main() {";
        }

        if line.contains("test_assert(db)") {
            continue;
        }

        if line.contains("use mongodb::Database;") {
            continue;
        }

        if line.contains("write_to_examples()") {
            continue;
        }

        if line.contains("async fn test_assert") {
            break;
        }

        cleaned_example = format!("{}\n{}", cleaned_example, line);
    }
    std::fs::write("./examples/as_lib.rs", cleaned_example).expect("example written");

    std::process::Command::new("rustfmt")
        .arg("./examples/as_lib.rs")
        .arg("--edition")
        .arg("2021")
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("command with rustfmt started")
        .wait_with_output()
        .expect("rustfmt executed");
}
