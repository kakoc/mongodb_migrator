//! This tests crate contains tests that check state when a migration failed
use bson::{self, Bson};
use futures::stream::StreamExt;
use mongodb_migrator::{
    migration::Migration, migration_record::MigrationRecord, migration_status::MigrationStatus,
};

use super::utils::{init_migrator_with_migrations, TestDb, Users, M0, M1, M2, M3};

pub async fn with_failed_migration_should_stop_after_first_fail_and_save_failed_with_next_not_executed_as_failed(
    t: &TestDb,
) {
    let migrations: Vec<Box<dyn Migration>> = vec![
        Box::new(M0 {}),
        Box::new(M3 {}),
        Box::new(M1 {}),
        Box::new(M2 {}),
    ];

    let _ = init_migrator_with_migrations(t.db.clone(), migrations)
        .up()
        .await;

    assert!(t
        .db
        .collection::<Users>("users")
        .find_one(bson::doc! {"x": 0}, None)
        .await
        .unwrap()
        .is_some());

    assert_eq!(
        t.db.collection("migrations")
            .find(bson::doc! {}, None)
            .await
            .unwrap()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(|v| bson::from_bson(Bson::Document(v.unwrap())).unwrap())
            .collect::<Vec<MigrationRecord>>()
            .into_iter()
            .filter(|v| v.status == MigrationStatus::Fail)
            .collect::<Vec<MigrationRecord>>()
            .len(),
        3
    );

    assert_eq!(
        t.db.collection::<MigrationRecord>("migrations")
            .find(bson::doc! {}, None)
            .await
            .unwrap()
            .collect::<Vec<_>>()
            .await
            .len(),
        4
    );
}
