//! These tests check how single migration via http server run works
use super::utils::{M0, M1, M2};
use axum::body::Body;
use bson::Bson;
use futures::stream::StreamExt;
use hyper::{Request, StatusCode};
use hyper_util::{client::legacy::Client, rt::TokioExecutor};
use mongodb::Database;
use mongodb_migrator::{
    migration::Migration,
    migration_record::MigrationRecord,
    server::{self, DbParams, MigratorParams, ServiceParams},
};
use testcontainers_modules::{mongo::Mongo, testcontainers::runners::AsyncRunner};

#[tokio::test]
pub async fn server_runs_migrations_by_id() {
    let migrations: Vec<Box<dyn Migration>> =
        vec![Box::new(M0 {}), Box::new(M1 {}), Box::new(M2 {})];
    let node = Mongo::default().start().await.unwrap();
    let host_port = node.get_host_port_ipv4(27017).await.unwrap();
    let url = format!("mongodb://localhost:{}/", host_port);
    let client = mongodb::Client::with_uri_str(url).await.unwrap();
    let db = client.database("test");

    let r = tokio::spawn(async move {
        tokio::spawn(async move {
            let params = ServiceParams {
                migrator: MigratorParams {
                    db: DbParams {
                        connection_string: format!("mongodb://localhost:{}/", host_port),
                        log_into_db_name: "test".to_string(),
                    },
                    migrations,
                },
                ..Default::default()
            };

            server::server(params).await;
        });

        check_ups(&db).await;

        db.drop().await.expect("test db deleted");

        check_downs(&db).await;
    })
    .await;

    assert!(r.is_ok());
}

async fn check_ups(db: &Database) {
    let migrations: Vec<Box<dyn Migration>> =
        vec![Box::new(M0 {}), Box::new(M1 {}), Box::new(M2 {})];
    let migrations_ids = migrations
        .iter()
        .map(|m| m.get_id().to_string())
        .collect::<Vec<String>>();

    let client = Client::builder(TokioExecutor::new()).build_http();

    let response = client
        .request(
            Request::builder()
                .uri(format!("http://{}/up/M0", "localhost:3000"))
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let response = client
        .request(
            Request::builder()
                .uri(format!("http://{}/up/M1", "localhost:3000"))
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let response = client
        .request(
            Request::builder()
                .uri(format!("http://{}/up/M2", "localhost:3000"))
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let all_records = db
        .collection("migrations")
        .find(bson::doc! {})
        .sort(bson::doc! {"end_date": 1})
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

async fn check_downs(db: &Database) {
    let migrations: Vec<Box<dyn Migration>> =
        vec![Box::new(M0 {}), Box::new(M1 {}), Box::new(M2 {})];
    let migrations_ids = migrations
        .iter()
        .map(|m| m.get_id().to_string())
        .collect::<Vec<String>>();

    let client = Client::builder(TokioExecutor::new()).build_http();
    let response = client
        .request(
            Request::builder()
                .uri(format!("http://{}/down/M2", "localhost:3000"))
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let response = client
        .request(
            Request::builder()
                .uri(format!("http://{}/down/M1", "localhost:3000"))
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let response = client
        .request(
            Request::builder()
                .uri(format!("http://{}/down/M0", "localhost:3000"))
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let all_records = db
        .collection("migrations")
        .find(bson::doc! {})
        .sort(bson::doc! {"end_date": 1})
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
