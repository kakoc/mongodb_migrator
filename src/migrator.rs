use anyhow::Result;
use std::path::PathBuf;

use crate::migration::Migration;
use crate::migration_record::MigrationRecord;

pub struct DefaultMigrator {}

pub struct WithConnection {
    pub db: mongodb::Database,
}

pub struct WithMigrationsFolder {
    pub migrations_folder: PathBuf,
}

pub struct WithMigrationsVec {
    pub migrations: Vec<String>,
}

pub struct WithConnectionAndMigrationsFolder {
    pub with_connection: WithConnection,
    pub migrations_folder: PathBuf,
}

pub struct WithConnectionAndMigrationsVec {
    pub with_connection: WithConnection,
    pub migrations: Vec<Box<dyn Migration>>,
}

pub enum Migrator {
    DefaultMigrator,
    WithConnection(WithConnection),
    WithMigrationsFolder(WithMigrationsFolder),
    WithConnectionAndMigrationsFolder,
    WithConnectionAndMigrationsVec,
}

impl WithConnection {
    pub fn with_migrations_folder(
        self,
        migrations_folder: PathBuf,
    ) -> WithConnectionAndMigrationsFolder {
        WithConnectionAndMigrationsFolder {
            migrations_folder,
            with_connection: self,
        }
    }

    pub fn with_migrations_vec(
        self,
        migrations: Vec<Box<dyn Migration>>,
    ) -> WithConnectionAndMigrationsVec {
        WithConnectionAndMigrationsVec {
            migrations,
            with_connection: self,
        }
    }
}

impl DefaultMigrator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn with_conn(self, db: mongodb::Database) -> WithConnection {
        WithConnection { db }
    }

    pub fn with_migrations_folder(self, migrations_folder: PathBuf) -> WithMigrationsFolder {
        WithMigrationsFolder { migrations_folder }
    }
}

impl WithConnectionAndMigrationsVec {
    pub async fn up(&self) -> Result<()> {
        for migration in self.migrations.iter() {
            let migration_record =
                MigrationRecord::migration_start(migration.get_name().to_string());

            self.with_connection
                .db
                .clone()
                .collection("migrations")
                .insert_one(bson::to_document(&migration_record)?, None)
                .await?;

            let result = migration.clone().up(self.with_connection.db.clone()).await;

            let migration_record = if result.is_ok() {
                migration_record.migration_succeeded()
            } else {
                migration_record.migration_failed()
            };

            // TODO
            // if migration_record couldn't be created
            // but a migration has already ran
            // log.error about inconsistent state
            // the same about writing about completion

            self.with_connection
                .db
                .clone()
                .collection("migrations")
                .insert_one(bson::to_document(&migration_record)?, None)
                .await?;
        }

        Ok(())
    }
}
