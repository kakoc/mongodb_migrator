//! These tests check whether passed migrations doesn't contain duplicates
use super::utils::{init_migrator_with_migrations, TestDb, M0, M1, M2};
use mongodb_migrator::{error::MigrationExecution, migration::Migration};

pub async fn validation_fails_when_passed_with_duplicates(t: &TestDb) {
    let migrations: Vec<Box<dyn Migration>> =
        vec![Box::new(M0 {}), Box::new(M0 {}), Box::new(M0 {})];

    let res = init_migrator_with_migrations(t.db.clone(), migrations)
        .up()
        .await;

    match res {
        Err(MigrationExecution::PassedMigrationsWithDuplicatedIds { duplicates }) => {
            assert_eq!(duplicates.len(), 1);
            assert_eq!(duplicates.get(M0 {}.get_id()).unwrap().len(), 3);
        }
        _ => unreachable!(),
    }
}

pub async fn validation_passes_since_all_unique(t: &TestDb) {
    let migrations: Vec<Box<dyn Migration>> =
        vec![Box::new(M0 {}), Box::new(M1 {}), Box::new(M2 {})];

    let res = init_migrator_with_migrations(t.db.clone(), migrations)
        .up()
        .await;

    assert!(res.is_ok());
}
