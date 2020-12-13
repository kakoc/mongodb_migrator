//! Migrator runs passed migrations - entities which implement [`Migration`] trait
use anyhow::Result;
use std::path::PathBuf;

use crate::migration::Migration;
use crate::migration_record::MigrationRecord;
use crate::migration_status::MigrationStatus;

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
        for (i, migration) in self.migrations.iter().enumerate() {
            let migration_record = MigrationRecord::migration_start(migration.git_id().to_string());

            let serialized_to_document_migration_record = if let Ok(v) =
                bson::to_document(&migration_record)
            {
                v
            } else {
                return Err(anyhow::Error::msg(format!(
		    "failed to create an initial record document for the migration which will be ran - {migration_id}
                     record attempted to serialize: {migration_record:?}
                     the migration: {migration_id}, and following it: {next_not_ran_migrations:?} weren't run",
		     migration_id = migration.git_id(),
		     migration_record = migration_record,
                     next_not_ran_migrations = self.migrations[i + 1..].iter().map(|m| m.git_id()).collect::<Vec<&str>>()
                )));
            };

            let res = self
                .with_connection
                .db
                .clone()
                .collection("migrations")
                .insert_one(serialized_to_document_migration_record, None)
                .await;

            if let Err(error) = res {
                return Err(anyhow::Error::msg(format!(
		    "failed to write an initial record document for the migration which will be ran - {migration_id}
                     the migration: {migration_id}, and following it: {next_not_ran_migrations:?} weren't run
                     additional_info: {additional_info}",
		     migration_id = migration.git_id(),
                     next_not_ran_migrations = self.migrations[i + 1..].iter().map(|m| m.git_id()).collect::<Vec<&str>>(),
		     additional_info = error
                )));
            }

            let result = migration.clone().up(self.with_connection.db.clone()).await;

            let migration_record = if result.is_ok() {
                migration_record.migration_succeeded()
            } else {
                migration_record.migration_failed()
            };

            let serialized_to_document_migration_record = if let Ok(v) =
                bson::to_document(&migration_record)
            {
                v
            } else {
                return Err(anyhow::Error::msg(format!(
		    "migration - {migration_id} has finished with the status: {migration_status}
                     but the migration_record attempted to be writted as a migration result into migrations collections
                     wasn't successfully serialized: {migration_record:?}, this is why it hasn't written
                     due to inconsistency, following it migrations: {next_not_ran_migrations:?} weren't run",
		     migration_id = migration.git_id(),
		     migration_status = &migration_record.status,
		     migration_record = migration_record,
                     next_not_ran_migrations = self.migrations[i + 1..].iter().map(|m| m.git_id()).collect::<Vec<&str>>()
                )));
            };

            let res = self
                .with_connection
                .db
                .clone()
                .collection("migrations")
                .update_one(
                    bson::doc! {"_id": res.unwrap().inserted_id},
                    serialized_to_document_migration_record,
                    None,
                )
                .await;

            if let Err(error) = res {
                return Err(anyhow::Error::msg(format!(
		    "failed to write the migration record document for the migration - {migration_id} with its result
                     that migration was completed with the status: {migration_status}
                     due to inconsistency, following it migrations: {next_not_ran_migrations:?} weren't run
                     additional_info: {additional_info}",
		     migration_id = migration.git_id(),
		     migration_status = &migration_record.status,
                     next_not_ran_migrations = self.migrations[i + 1..].iter().map(|m| m.git_id()).collect::<Vec<&str>>(),
		     additional_info = error
                )));
            }

            if migration_record.status == MigrationStatus::Failed {
                return Err(anyhow::Error::msg(format!(
                    "migration wasn't completed successfully - {migration_id}
                     due to that, following it migrations: {next_not_ran_migrations:?} weren't run",
                    migration_id = migration.git_id(),
                    next_not_ran_migrations = self.migrations[i + 1..]
                        .iter()
                        .map(|m| m.git_id())
                        .collect::<Vec<&str>>(),
                )));
            }
        }

        Ok(())
    }
}
