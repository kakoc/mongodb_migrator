//! This tests crate contains tests that check migrations execution order
use bson::{self, Bson};
use futures::stream::StreamExt;
use mongodb::options::FindOptions;
use mongodb_migrator::{
    migration::Migration, migration_record::MigrationRecord, migration_status::MigrationStatus,
};

use super::utils::{init_migrator_with_migrations, TestDb, Users, M0, M1, M2};

// M0 -> M1 -> M2
pub async fn migrations_executed_in_specified_order(t: &TestDb<'_>) {
    let migrations: Vec<Box<dyn Migration>> =
        vec![Box::new(M0 {}), Box::new(M1 {}), Box::new(M2 {})];
    let migrations_ids = migrations
        .iter()
        .map(|m| m.get_id().to_string())
        .collect::<Vec<String>>();

    init_migrator_with_migrations(t.db.clone(), migrations)
        .up()
        .await
        .unwrap();

    let mut f_o: FindOptions = Default::default();
    f_o.sort = Some(bson::doc! {"end_date": 1});

    let all_records =
        t.db.collection("migrations")
            .find(bson::doc! {}, f_o)
            .await
            .unwrap()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(|v| bson::from_bson(Bson::Document(v.unwrap())).unwrap())
            .map(|v: MigrationRecord| v._id)
            .collect::<Vec<String>>();

    assert_eq!(all_records, migrations_ids);
}

/// M0(Success) , M1(Success) , M2(Success)
pub async fn all_migrations_have_success_status(t: &TestDb<'_>) {
    let migrations: Vec<Box<dyn Migration>> =
        vec![Box::new(M0 {}), Box::new(M1 {}), Box::new(M2 {})];
    let migrations_len = migrations.len();

    init_migrator_with_migrations(t.db.clone(), migrations)
        .up()
        .await
        .unwrap();

    let all_records =
        t.db.collection("migrations")
            .find(bson::doc! {}, None)
            .await
            .unwrap()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(|v| bson::from_bson(Bson::Document(v.unwrap())).unwrap())
            .map(|v: MigrationRecord| v.status)
            .collect::<Vec<MigrationStatus>>();

    assert_eq!(all_records.len(), migrations_len);
    assert!(all_records.iter().all(|v| *v == MigrationStatus::Success));
}

pub async fn migrations_not_just_saved_as_executed_but_really_affected_target(t: &TestDb<'_>) {
    let migrations: Vec<Box<dyn Migration>> =
        vec![Box::new(M0 {}), Box::new(M1 {}), Box::new(M2 {})];

    init_migrator_with_migrations(t.db.clone(), migrations)
        .up()
        .await
        .unwrap();

    assert!(t
        .db
        .collection::<Users>("users")
        .find_one(bson::doc! {"x": 2}, None)
        .await
        .unwrap()
        .is_some());
}

// M2 -> M1 -> M0
// TODO(kakoc) : move all down to a separate folder?
pub async fn down_migrations_executed_in_specified_order(t: &TestDb<'_>) {
    let migrations: Vec<Box<dyn Migration>> =
        vec![Box::new(M0 {}), Box::new(M1 {}), Box::new(M2 {})];
    let migrations_ids = migrations
        .iter()
        .map(|m| m.get_id().to_string())
        .collect::<Vec<String>>();

    init_migrator_with_migrations(t.db.clone(), migrations)
        .down()
        .await
        .unwrap();

    let mut f_o: FindOptions = Default::default();
    f_o.sort = Some(bson::doc! {"end_date": 1});

    let all_records =
        t.db.collection("migrations")
            .find(bson::doc! {}, f_o)
            .await
            .unwrap()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(|v| bson::from_bson(Bson::Document(v.unwrap())).unwrap())
            .map(|v: MigrationRecord| v._id)
            .collect::<Vec<String>>();

    assert_eq!(
        all_records,
        migrations_ids.into_iter().rev().collect::<Vec<String>>()
    );
}
