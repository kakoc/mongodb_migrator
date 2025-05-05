use utils::TestDb;

mod basic;
mod fail;
mod migration_trait;
mod rerun;
mod sequence;
mod server;
mod shell;
mod single_run_migrations;
mod utils;
mod validate;
mod version_numbers;

#[tokio::test]
async fn run_all_tests() {
    let t = TestDb::new().await;
    macro_rules! run_test {
        ($b:stmt) => {
            $b
            t.db.drop(None).await.expect("test db deleted");
        };
    }

    run_test!(basic::basic(&t.node).await);
    run_test!(basic::custom_collection_name(&t.node).await);

    run_test!(fail::with_failed_migration_should_stop_after_first_fail_and_save_failed_with_next_not_executed_as_failed(&t).await);

    run_test!(rerun::picks_only_failed(&t).await);

    run_test!(sequence::migrations_executed_in_specified_order(&t).await);
    run_test!(sequence::all_migrations_have_success_status(&t).await);
    run_test!(sequence::migrations_not_just_saved_as_executed_but_really_affected_target(&t).await);
    run_test!(sequence::down_migrations_executed_in_specified_order(&t).await);

    run_test!(shell::shell(&t).await);

    run_test!(single_run_migrations::migrations_executed_in_single_manner(&t).await);
    run_test!(single_run_migrations::down_migrations_executed_in_single_manner(&t).await);

    run_test!(validate::validation_fails_when_passed_with_duplicates(&t).await);
    run_test!(validate::validation_passes_since_all_unique(&t).await);

    run_test!(version_numbers::test_readme_deps());

    run_test!(migration_trait::migration_id_autoderived());
}
