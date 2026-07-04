mod common;

use common::db_test;

db_test! {
    async fn provisions_a_usable_migrated_database(pool: PgPool) {
        let one: i32 = sqlx::query_scalar("SELECT 1")
            .fetch_one(&pool)
            .await
            .expect("query the provisioned database");
        assert_eq!(one, 1);
    }
}

db_test! {
    async fn a_test_can_write_to_its_own_database(pool: PgPool) {
        sqlx::query("CREATE TABLE only_here (id int)")
            .execute(&pool)
            .await
            .expect("create a table in this test's database");
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM only_here")
            .fetch_one(&pool)
            .await
            .expect("read the table");
        assert_eq!(count, 0);
    }
}

db_test! {
    async fn databases_are_isolated_between_tests(pool: PgPool) {
        // The table created by the other test must not exist here - each test runs
        // against its own database cloned from the (empty) template.
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'only_here')",
        )
        .fetch_one(&pool)
        .await
        .expect("check table isolation");
        assert!(!exists, "each test must get an isolated database");
    }
}
