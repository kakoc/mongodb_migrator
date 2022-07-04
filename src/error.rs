use std::collections::BTreeMap;

use mongodb::error::Error as MongoDbError;
use thiserror::Error;

use crate::migration_record::MigrationRecord;

#[derive(Error, Debug)]
pub enum MigrationExecution {
    #[error("Failed to create an initial record document for the migration which will be executed - {migration_id}
	    record attempted to serialize: {migration_record:?}
	    the migration: {migration_id}, and following it: {next_not_executed_migrations_ids:?} weren't executed")]
    InitialMigrationRecord {
        migration_id: String,
        migration_record: MigrationRecord,
        next_not_executed_migrations_ids: Vec<String>,
        additional_info: bson::ser::Error,
    },
    #[error("Failed to write an initial record document for the migration which will be executed - {migration_id}
	    the migration: {migration_id}, and following it: {next_not_executed_migrations_ids:?} weren't executed
	    additional_info: {additional_info}")]
    InProgressStatusNotSaved {
        migration_id: String,
        next_not_executed_migrations_ids: Vec<String>,
        additional_info: MongoDbError,
    },
    #[error("Migration - {migration_id} has finished with the status: {migration_status}
	    but the migration_record attempted to be writted as a migration result into migrations collections
	    wasn't successfully serialized: {migration_record:?}, this is why it hasn't written
	    due to inconsistency, following it migrations: {next_not_executed_migrations_ids:?} weren't executed")]
    FinishedButNotSavedDueToSerialization {
        migration_id: String,
        migration_status: String,
        migration_record: MigrationRecord,
        next_not_executed_migrations_ids: Vec<String>,
        additional_info: bson::ser::Error,
    },
    #[error("Failed to write the migration record document for the migration - {migration_id} with its result
	    that migration was completed with the status: {migration_status}
	    due to inconsistency, following it migrations: {next_not_executed_migrations_ids:?} weren't executed
	    additional_info: {additional_info}")]
    FinishedButNotSavedDueMongoError {
        migration_id: String,
        migration_status: String,
        additional_info: MongoDbError,
        next_not_executed_migrations_ids: Vec<String>,
    },
    #[error(
        "Migration wasn't completed successfully - {migration_id}
	 due to that, following it migrations: {next_not_executed_migrations_ids:?} weren't executed"
    )]
    FinishedAndSavedAsFail {
        migration_id: String,
        next_not_executed_migrations_ids: Vec<String>,
    },
    #[error(
        "Migrations weren't executed since there are several migrations with duplicated ids(id, indices vec):
	 {duplicates:?}"
    )]
    PassedMigrationsWithDuplicatedIds {
        duplicates: BTreeMap<String, Vec<usize>>,
    },
}
