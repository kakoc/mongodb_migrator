use std::collections::BTreeMap;

use bson::{Bson, Document};
use futures::StreamExt;
use mongodb::{options::UpdateOptions, results::InsertOneResult};

use super::{
    shell::Shell, with_connection::WithConnection, with_shell_config::WithShellConfig, Env,
};
use crate::{
    error::MigrationExecution, migration::Migration, migration_record::MigrationRecord,
    migration_status::MigrationStatus,
};

pub struct WithMigrationsVec {
    pub with_shell_config: Option<WithShellConfig>,
    pub with_connection: WithConnection,
    pub migrations: Vec<Box<dyn Migration>>,
}

impl WithMigrationsVec {
    fn get_not_executed_migrations_ids(&self, first_failed_migration_index: usize) -> Vec<String> {
        if self.migrations.len() - 1 == first_failed_migration_index {
            vec![]
        } else {
            self.migrations[first_failed_migration_index + 1..]
                .iter()
                .map(|m| m.get_id().to_string())
                .collect::<Vec<_>>()
        }
    }

    async fn get_migrations_ids_to_execute_from_index(&self, lookup_from: usize) -> Vec<String> {
        if self.migrations.len() - 1 == lookup_from {
            vec![]
        } else {
            let ids = self.migrations[lookup_from..]
                .into_iter()
                .map(|migration| migration.get_id().to_string())
                .collect::<Vec<String>>();

            let mut failed = self.with_connection
                .db
                .clone()
                .collection("migrations")
                .find(
                    bson::doc! {"_id": {"$in": ids.clone()}, "status": format!("{:?}", MigrationStatus::Fail)},
                    None,
                )
		.await.unwrap().collect::<Vec<_>>().await
		.into_iter()
		// TODO(koc_kakoc): replace unwrap?
		.map(|v| bson::from_bson(Bson::Document(v.unwrap())).unwrap())
		.map(|v: MigrationRecord| v._id.to_string())
		.collect::<Vec<String>>();

            // TODO(koc_kakoc): use Set
            let all = self
                .with_connection
                .db
                .clone()
                .collection("migrations")
                .find(bson::doc! {}, None)
                .await
                .unwrap()
                .collect::<Vec<_>>()
                .await
                .into_iter()
                // TODO(koc_kakoc): replace unwrap?
                .map(|v| bson::from_bson(Bson::Document(v.unwrap())).unwrap())
                .map(|v: MigrationRecord| v._id.to_string())
                .collect::<Vec<String>>();

            failed.extend(ids.into_iter().filter(|id| !all.contains(&id)));
            failed
        }
    }

    /// This function executes all passed migrations in the passed order
    /// for migration in migrations
    ///   createInProgressBson
    ///   handleIfFailed
    ///   saveInMongoAsInProgress
    ///   handleIfResultWasntSaved
    ///   up
    ///   createFinishedBson
    ///   handleIfFailed
    ///   saveInMongoAsFinished
    ///   handleIfResultWasntSaved
    ///   returnIfMigrationUpWithFailedResultWithAllNextSavedAsFail
    pub async fn up(&self) -> Result<(), MigrationExecution> {
        self.validate()?;

        // TODO(koc_kakoc): execute only failed or not stored in migrations collections
        let ids = self.get_migrations_ids_to_execute_from_index(0).await;
        for (i, migration) in self
            .migrations
            .iter()
            .filter(|m| ids.contains(&m.get_id().to_string()))
            .enumerate()
        {
            self.try_run_migration(migration, i).await?;
        }

        Ok(())
    }

    async fn save_not_executed_migrations(
        &self,
        save_from_index: usize,
    ) -> Result<(), MigrationExecution> {
        if self.migrations.len() - 1 == save_from_index {
            return Ok(());
        }

        for (i, migration) in self.migrations[save_from_index..].iter().enumerate() {
            let migration_record = MigrationRecord::migration_start(migration.get_id().to_string());
            let migration_record = MigrationRecord::migration_failed(migration_record);
            let serialized_to_document_migration_record = bson::to_document(&migration_record)
                .map_err(|error| MigrationExecution::InitialMigrationRecord {
                    migration_id: migration.get_id().to_string(),
                    migration_record: migration_record.clone(),
                    next_not_executed_migrations_ids: self.get_not_executed_migrations_ids(i),
                    additional_info: error,
                })?;

            let mut u_o: UpdateOptions = Default::default();
            u_o.upsert = Some(true);

            self.with_connection
                .db
                .clone()
                .collection::<MigrationRecord>("migrations")
                .update_one(
                    bson::doc! {"_id": &migration_record._id},
                    bson::doc! {"$set": serialized_to_document_migration_record},
                    u_o,
                )
                .await
                .map_err(
                    |error| MigrationExecution::FinishedButNotSavedDueMongoError {
                        migration_id: migration.get_id().to_string(),
                        migration_status: format!("{:?}", &migration_record.status),
                        additional_info: error,
                        next_not_executed_migrations_ids: self.get_not_executed_migrations_ids(i),
                    },
                )?;
        }

        Ok(())
    }

    pub async fn up_single_from_vec(&self, migration_id: String) -> Result<(), MigrationExecution> {
        self.validate()?;

        let migration = self
            .migrations
            .iter()
            .enumerate()
            .find(|(_index, migration)| migration.get_id().to_string() == migration_id);

        if migration.is_some() {
            let (index, migration) = migration.unwrap();
            self.try_run_migration(migration, index).await
        } else {
            Err(MigrationExecution::MigrationFromVecNotFound { migration_id })
        }
    }

    fn validate(&self) -> Result<(), MigrationExecution> {
        let mut entries = BTreeMap::new();
        self.migrations
            .iter()
            .enumerate()
            .for_each(|(index, migration)| {
                let entry = entries
                    .entry(migration.get_id().to_string())
                    .or_insert(vec![]);
                entry.push(index);
            });

        let duplicates = entries
            .into_iter()
            .filter(|(_id, indices)| indices.len() > 1)
            .collect::<BTreeMap<String, Vec<usize>>>();

        if duplicates.len() > 0 {
            Err(MigrationExecution::PassedMigrationsWithDuplicatedIds { duplicates })
        } else {
            Ok(())
        }
    }

    fn prepare_initial_migration_record(
        &self,
        migration: &Box<dyn Migration>,
        i: usize,
    ) -> Result<(Document, MigrationRecord), MigrationExecution> {
        let migration_record = MigrationRecord::migration_start(migration.get_id().to_string());

        Ok((
            bson::to_document(&migration_record).map_err(|error| {
                MigrationExecution::InitialMigrationRecord {
                    migration_id: migration.get_id().to_string(),
                    migration_record: migration_record.clone(),
                    next_not_executed_migrations_ids: self.get_not_executed_migrations_ids(i),
                    additional_info: error,
                }
            })?,
            migration_record,
        ))
    }

    async fn save_initial_migration_record(
        &self,
        migration: &Box<dyn Migration>,
        serialized_to_document_migration_record: Document,
        i: usize,
    ) -> Result<InsertOneResult, MigrationExecution> {
        Ok(self
            .with_connection
            .db
            .clone()
            .collection("migrations")
            .insert_one(serialized_to_document_migration_record, None)
            .await
            .map_err(|error| MigrationExecution::InProgressStatusNotSaved {
                migration_id: migration.get_id().to_string(),
                additional_info: error,
                next_not_executed_migrations_ids: self.get_not_executed_migrations_ids(i),
            })?)
    }

    async fn save_executed_migration_record(
        &self,
        migration: &Box<dyn Migration>,
        migration_record: &MigrationRecord,
        serialized_to_document_migration_record: Document,
        res: InsertOneResult,
        i: usize,
    ) -> Result<(), MigrationExecution> {
        let mut u_o: UpdateOptions = Default::default();
        u_o.upsert = Some(true);

        self.with_connection
            .db
            .clone()
            .collection::<MigrationRecord>("migrations")
            .update_one(
                bson::doc! {"_id": res.inserted_id},
                bson::doc! {"$set": serialized_to_document_migration_record},
                u_o,
            )
            .await
            .map_err(
                |error| MigrationExecution::FinishedButNotSavedDueMongoError {
                    migration_id: migration.get_id().to_string(),
                    migration_status: format!("{:?}", &migration_record.status),
                    additional_info: error,
                    next_not_executed_migrations_ids: self.get_not_executed_migrations_ids(i),
                },
            )?;

        Ok(())
    }

    fn try_get_mongo_shell(&self) -> Option<Shell> {
        if self.with_shell_config.is_some() {
            Some(Shell {
                config: self
                    .with_shell_config
                    .clone()
                    .expect("shell config is present")
                    .with_shell_config,
            })
        } else {
            None
        }
    }

    async fn run_migration(
        &self,
        migration: &Box<dyn Migration>,
        shell: Option<Shell>,
        migration_record: &MigrationRecord,
    ) -> MigrationRecord {
        migration
            .clone()
            .up(Env {
                db: Some(self.with_connection.db.clone()),
                shell,
                ..Default::default()
            })
            .await
            .map_or_else(
                |_| migration_record.clone().migration_failed(),
                |_| migration_record.clone().migration_succeeded(),
            )
    }

    async fn try_run_migration(
        &self,
        migration: &Box<dyn Migration>,
        i: usize,
    ) -> Result<(), MigrationExecution> {
        let (serialized_to_document_migration_record, migration_record) =
            self.prepare_initial_migration_record(migration, i)?;

        let res = self
            .save_initial_migration_record(migration, serialized_to_document_migration_record, i)
            .await?;

        let shell = self.try_get_mongo_shell();
        let migration_record = self
            .run_migration(migration, shell, &migration_record)
            .await;

        let serialized_to_document_migration_record = bson::to_document(&migration_record)
            .map_err(
                |error| MigrationExecution::FinishedButNotSavedDueToSerialization {
                    migration_id: migration.get_id().to_string(),
                    migration_status: format!("{:?}", &migration_record.status),
                    migration_record: migration_record.clone(),
                    next_not_executed_migrations_ids: self.get_not_executed_migrations_ids(i),
                    additional_info: error,
                },
            )?;

        self.save_executed_migration_record(
            migration,
            &migration_record,
            serialized_to_document_migration_record,
            res,
            i,
        )
        .await?;

        if migration_record.status == MigrationStatus::Fail {
            self.save_not_executed_migrations(i + 1).await?;
            return Err(MigrationExecution::FinishedAndSavedAsFail {
                migration_id: migration.get_id().to_string(),
                next_not_executed_migrations_ids: self.get_not_executed_migrations_ids(i),
            });
        }

        Ok(())
    }
}
