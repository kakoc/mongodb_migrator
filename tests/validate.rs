//! These tests check whether passed migrations doesn't contain duplicates
use mongodb_migrator::{error::MigrationExecution, migration::Migration};
mod utils;
use utils::{init_migrator_with_migrations, TestDb, M0, M1, M2};

#[tokio::test]
async fn validation_fails_when_passed_with_duplicates() {
    let docker = testcontainers::clients::Cli::default();
    let t = TestDb::new(&docker).await;

    let migrations: Vec<Box<dyn Migration>> =
        vec![Box::new(M0 {}), Box::new(M0 {}), Box::new(M0 {})];

    let res = init_migrator_with_migrations(t.db.clone(), migrations)
        .up()
        .await;

    match res {
        Err(MigrationExecution::PassedMigrationsWithDuplicatedIds { duplicates }) => {
            assert_eq!(duplicates.len(), 1);
            assert_eq!(
                duplicates.get(&M0 {}.get_id().to_string()).unwrap().len(),
                3
            );
        }
        _ => assert!(false),
    }
}

#[tokio::test]
async fn validation_passes_since_all_unique() {
    let docker = testcontainers::clients::Cli::default();
    let t = TestDb::new(&docker).await;

    let migrations: Vec<Box<dyn Migration>> =
        vec![Box::new(M0 {}), Box::new(M1 {}), Box::new(M2 {})];

    let res = init_migrator_with_migrations(t.db.clone(), migrations)
        .up()
        .await;

    assert!(res.is_ok());
}
