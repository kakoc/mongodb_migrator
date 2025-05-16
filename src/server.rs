use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::post,
    Router,
};
use tokio::{net::TcpListener, sync::Mutex};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    migration::Migration,
    migrator::{default::DefaultMigrator, with_migrations_vec::WithMigrationsVec},
};

struct AppState {
    migrator: WithMigrationsVec,
}

type SharedState = Arc<Mutex<AppState>>;

pub struct ServiceParams {
    pub migrator: MigratorParams,
    pub server: ServerParams,
}

pub struct MigratorParams {
    pub db: DbParams,
    pub migrations: Vec<Box<dyn Migration>>,
}

pub struct ServerParams {
    port: u16,
}

impl Default for ServiceParams {
    fn default() -> Self {
        Self {
            migrator: MigratorParams {
                db: DbParams {
                    connection_string: "mongodb://localhost:27017".to_string(),
                    log_into_db_name: "test".to_string(),
                },
                migrations: vec![],
            },
            server: ServerParams { port: 3000 },
        }
    }
}

pub struct DbParams {
    pub connection_string: String,
    pub log_into_db_name: String,
}

pub async fn server(params: ServiceParams) {
    init_tracing();

    let migrator = init_migrator(params.migrator).await;

    run_server(
        init_routing(Arc::new(Mutex::new(AppState { migrator }))),
        params.server.port,
    )
    .await;
}

fn ups() -> Router<SharedState> {
    Router::new().route("/{id}", post(up_migration_with_id))
}

fn downs() -> Router<SharedState> {
    Router::new().route("/{id}", post(down_migration_with_id))
}

async fn up_migration_with_id(
    Path(id): Path<String>,
    State(state): State<SharedState>,
) -> StatusCode {
    let r = state.lock().await.migrator.up_single_from_vec(id).await;

    if r.is_ok() {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

async fn down_migration_with_id(
    Path(id): Path<String>,
    State(state): State<SharedState>,
) -> StatusCode {
    let r = state.lock().await.migrator.down_single_from_vec(id).await;

    if r.is_ok() {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mongodb-migrator=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn init_routing(shared_state: SharedState) -> Router {
    Router::new()
        .nest("/up", ups())
        .nest("/down", downs())
        .with_state(shared_state)
}

async fn init_migrator(params: MigratorParams) -> WithMigrationsVec {
    DefaultMigrator::new()
        .with_conn(
            mongodb::Client::with_uri_str(params.db.connection_string)
                .await
                .expect("mongodb client created")
                .database(&params.db.log_into_db_name),
        )
        .with_migrations_vec(params.migrations)
}

async fn run_server(router: Router, port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    tracing::debug!("listening on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();
}
