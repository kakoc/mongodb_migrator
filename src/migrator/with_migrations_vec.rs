use std::borrow::Cow;
use std::{collections::BTreeMap, ops::Range, thread::sleep};

use bson::{Bson, Document};
use futures::StreamExt;
use mongodb::{options::UpdateOptions, results::InsertOneResult};

use super::{
    shell::Shell, with_connection::WithConnection, with_retries::Retry,
    with_shell_config::WithShellConfig, Env,
};
use crate::{
    error::MigrationExecution, migration::Migration, migration_record::MigrationRecord,
    migration_status::MigrationStatus,
};

pub struct WithMigrationsVec {
    pub with_shell_config: Option<WithShellConfig>,
    pub with_connection: WithConnection,
    pub migrations: Vec<Box<dyn Migration>>,
    pub with_retries_per_migration: Retry,
    pub collection_name: Option<String>,
}

impl WithMigrationsVec {
    /// Set custom migrations collection name
    pub fn set_collection_name<S: Into<String>>(
        &mut self,
        collection_name: S,
    ) -> &mut WithMigrationsVec {
        self.collection_name = Some(collection_name.into());
        self
    }

    /// Get collection name
    fn get_collection_name(&self) -> Cow<'static, str> {
        match self.collection_name.clone() {
            None => "migrations".into(),
            Some(collection_name) => collection_name.into(),
        }
    }

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

    async fn get_migrations_ids_to_execute_from_index(&self, range: Range<usize>) -> Vec<String> {
        let ids = self.migrations[range]
            .iter()
            .map(|migration| migration.get_id().to_string())
            .collect::<Vec<String>>();

        let mut failed = self.with_connection
                .db
                .clone()
                .collection(&self.get_collection_name())
                .find(
                    bson::doc! {"_id": {"$in": ids.clone()}, "status": format!("{:?}", MigrationStatus::Fail)},
                    None,
                )
		.await.unwrap().collect::<Vec<_>>().await
		.into_iter()
		// TODO(koc_kakoc): replace unwrap?
		.map(|v| bson::from_bson(Bson::Document(v.unwrap())).unwrap())
		.map(|v: MigrationRecord| v._id)
		.collect::<Vec<String>>();

        // TODO(koc_kakoc): use Set
        let all = self
            .with_connection
            .db
            .clone()
            .collection(&self.get_collection_name())
            .find(bson::doc! {}, None)
            .await
            .unwrap()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            // TODO(koc_kakoc): replace unwrap?
            .map(|v| bson::from_bson(Bson::Document(v.unwrap())).unwrap())
            .map(|v: MigrationRecord| v._id)
            .collect::<Vec<String>>();

        failed.extend(ids.into_iter().filter(|id| !all.contains(id)));
        failed
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
        self.exec(
            Range {
                start: 0,
                end: self.migrations.len(),
            },
            OperationType::Up,
        )
        .await
    }

    async fn exec(
        &self,
        range: Range<usize>,
        operation_type: OperationType,
    ) -> Result<(), MigrationExecution> {
        self.validate()?;

        let ids = self.get_migrations_ids_to_execute_from_index(range).await;

        tracing::info!(
            message = "the following migrations are going to be executed",
            ids = format!("{:?}", ids),
            op = format!("{:?}", operation_type.clone())
        );

        let it = match operation_type {
            OperationType::Up => self
                .migrations
                .iter()
                .filter(|m| ids.contains(&m.get_id().to_string()))
                .collect::<Vec<_>>(),
            OperationType::Down => self
                .migrations
                .iter()
                .rev()
                .filter(|m| ids.contains(&m.get_id().to_string()))
                .collect::<Vec<_>>(),
        };

        for (i, migration) in it.into_iter().enumerate() {
            let mut retries = self.with_retries_per_migration.count;

            while let Err(e) = self
                .try_run_migration(&**migration, i, operation_type.clone())
                .await
            {
                self.trace_result(&**migration, &Err(e.clone()), operation_type.clone());
                if retries == 0 {
                    return Err(e);
                }
                retries -= 1;
                sleep(self.with_retries_per_migration.delay);
            }
        }

        Ok(())
    }

    pub async fn down(&self) -> Result<(), MigrationExecution> {
        self.exec(
            Range {
                start: 0,
                end: self.migrations.len(),
            },
            OperationType::Down,
        )
        .await
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
                .collection::<MigrationRecord>(&self.get_collection_name())
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

    /// Tries to up a migration from the passed before vec
    pub async fn up_single_from_vec(&self, migration_id: String) -> Result<(), MigrationExecution> {
        let migration = self
            .migrations
            .iter()
            .enumerate()
            .position(|(_index, migration)| migration.get_id() == migration_id);

        if let Some(i) = migration {
            self.exec(
                Range {
                    start: i,
                    end: i + 1,
                },
                OperationType::Up,
            )
            .await
        } else {
            Err(MigrationExecution::MigrationFromVecNotFound { migration_id })
        }
    }

    /// Tries do rollback a migration from the bassed before vec
    pub async fn down_single_from_vec(
        &self,
        migration_id: String,
    ) -> Result<(), MigrationExecution> {
        let migration = self
            .migrations
            .iter()
            .enumerate()
            .position(|(_index, migration)| migration.get_id() == migration_id);

        if let Some(i) = migration {
            self.exec(
                Range {
                    start: i,
                    end: i + 1,
                },
                OperationType::Down,
            )
            .await
        } else {
            Err(MigrationExecution::MigrationFromVecNotFound { migration_id })
        }
    }

    #[allow(clippy::result_large_err)]
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

        if !duplicates.is_empty() {
            Err(MigrationExecution::PassedMigrationsWithDuplicatedIds { duplicates })
        } else {
            Ok(())
        }
    }

    #[allow(clippy::result_large_err)]
    fn prepare_initial_migration_record(
        &self,
        migration: &dyn Migration,
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
        migration: &dyn Migration,
        serialized_to_document_migration_record: Document,
        i: usize,
    ) -> Result<InsertOneResult, MigrationExecution> {
        self.with_connection
            .db
            .clone()
            .collection(&self.get_collection_name())
            .insert_one(serialized_to_document_migration_record, None)
            .await
            .map_err(|error| MigrationExecution::InProgressStatusNotSaved {
                migration_id: migration.get_id().to_string(),
                additional_info: error,
                next_not_executed_migrations_ids: self.get_not_executed_migrations_ids(i),
            })
    }

    async fn save_executed_migration_record(
        &self,
        migration: &dyn Migration,
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
            .collection::<MigrationRecord>(&self.get_collection_name())
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

    async fn up_migration(
        &self,
        migration: &dyn Migration,
        shell: Option<Shell>,
        migration_record: &MigrationRecord,
    ) -> MigrationRecord {
        migration
            .up(Env {
                db: Some(self.with_connection.db.clone()),
                shell,
            })
            .await
            .map_or_else(
                |_| migration_record.clone().migration_failed(),
                |_| migration_record.clone().migration_succeeded(),
            )
    }

    async fn down_migration(
        &self,
        migration: &dyn Migration,
        shell: Option<Shell>,
        migration_record: &MigrationRecord,
    ) -> MigrationRecord {
        migration
            .down(Env {
                db: Some(self.with_connection.db.clone()),
                shell,
            })
            .await
            .map_or_else(
                |_| migration_record.clone().migration_failed(),
                |_| migration_record.clone().migration_succeeded(),
            )
    }

    async fn try_run_migration(
        &self,
        migration: &dyn Migration,
        i: usize,
        operation_type: OperationType,
    ) -> Result<(), MigrationExecution> {
        tracing::info!(
            id = migration.get_id(),
            op = format!("{:?}", operation_type),
            status = format!("{:?}", MigrationStatus::InProgress)
        );

        let (serialized_to_document_migration_record, migration_record) =
            self.prepare_initial_migration_record(migration, i)?;

        let res = self
            .save_initial_migration_record(migration, serialized_to_document_migration_record, i)
            .await?;

        let shell = self.try_get_mongo_shell();

        let migration_record = match operation_type {
            OperationType::Up => self.up_migration(migration, shell, &migration_record).await,
            OperationType::Down => {
                self.down_migration(migration, shell, &migration_record)
                    .await
            }
        };

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

    fn trace_result(
        &self,
        migration: &dyn Migration,
        migration_result: &Result<(), MigrationExecution>,
        operation_type: OperationType,
    ) {
        tracing::info!(
            id = migration.get_id(),
            op = format!("{:?}", operation_type),
            status = format!(
                "{:?}",
                (if migration_result.is_ok() {
                    MigrationStatus::Success
                } else {
                    MigrationStatus::Fail
                })
            )
        );
    }
}

#[derive(Debug, Clone)]
enum OperationType {
    Up,
    Down,
}
