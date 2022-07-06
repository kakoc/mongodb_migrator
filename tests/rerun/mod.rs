//! This tests crate contains tests that check what will be happened
//! when there are already executed migrations from the the previous "release"
use bson::{self, Bson};
use futures::stream::StreamExt;
use mongodb_migrator::{
    migration::Migration, migration_record::MigrationRecord, migration_status::MigrationStatus,
};

use super::utils::{init_migrator_with_migrations, TestDb, M0, M1, M2, M3};

pub async fn picks_only_failed<'a>(t: &TestDb<'a>) {
    let migration_record = MigrationRecord::migration_succeeded(MigrationRecord::migration_start(
        M0 {}.get_id().to_string(),
    ));

    t.db.collection("migrations")
        .insert_one(bson::to_document(&migration_record).unwrap(), None)
        .await
        .unwrap();

    let migrations: Vec<Box<dyn Migration>> = vec![
        Box::new(M0 {}),
        Box::new(M3 {}),
        Box::new(M1 {}),
        Box::new(M2 {}),
    ];

    let _ = init_migrator_with_migrations(t.db.clone(), migrations)
        .up()
        .await;

    let saved_migration_before =
        t.db.collection("migrations")
            .find_one(bson::doc! {"_id": M0{}.get_id()}, None)
            .await
            .unwrap()
            .unwrap();
    let saved_migration_before: MigrationRecord =
        bson::from_bson(Bson::Document(saved_migration_before)).unwrap();

    assert_eq!(saved_migration_before, migration_record);

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
