//! These tests check how single migration run works
use bson::Bson;
use futures::stream::StreamExt;
use mongodb::options::FindOptions;
use mongodb_migrator::{migration::Migration, migration_record::MigrationRecord};
mod utils;
use utils::{init_migrator_with_migrations, TestDb, M0, M1, M2};

// M0 -> M1 -> M2
#[tokio::test]
async fn migrations_executed_in_single_manner() {
    let docker = testcontainers::clients::Cli::default();
    let t = TestDb::new(&docker).await;

    let migrations: Vec<Box<dyn Migration>> =
        vec![Box::new(M0 {}), Box::new(M1 {}), Box::new(M2 {})];
    let migrations_ids = migrations
        .iter()
        .map(|m| m.get_id().to_string())
        .collect::<Vec<String>>();

    let migrator = init_migrator_with_migrations(t.db.clone(), migrations); // .unwrap();

    migrator
        .up_single_from_vec(M0 {}.get_id().to_string())
        .await
        .unwrap();
    migrator
        .up_single_from_vec(M1 {}.get_id().to_string())
        .await
        .unwrap();
    migrator
        .up_single_from_vec(M2 {}.get_id().to_string())
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
            .map(|v: MigrationRecord| v._id.to_string())
            .collect::<Vec<String>>();

    assert_eq!(all_records, migrations_ids);
}
