use sqlx::{Connection, PgConnection, PgPool};
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::{ContainerAsync, ImageExt};
use tokio::sync::{Mutex, OnceCell};
use uuid::Uuid;

const TEMPLATE_DB: &str = "kanban_test_template";

static SHARED: OnceCell<SharedPostgres> = OnceCell::const_new();
static CREATE_DB_LOCK: Mutex<()> = Mutex::const_new(());

struct SharedPostgres {
    _container: ContainerAsync<Postgres>,
    host: String,
    port: u16,
}

impl SharedPostgres {
    fn url(&self, db: &str) -> String {
        format!(
            "postgres://postgres:postgres@{}:{}/{}",
            self.host, self.port, db
        )
    }
}

async fn shared() -> &'static SharedPostgres {
    SHARED
        .get_or_init(|| async {
            let container = Postgres::default()
                .with_tag("18-alpine")
                .with_label("com.kanban.test", "1")
                .start()
                .await
                .expect("start postgres testcontainer");
            let host = container
                .get_host()
                .await
                .expect("container host")
                .to_string();
            let port = container
                .get_host_port_ipv4(5432)
                .await
                .expect("container port");
            let shared = SharedPostgres {
                _container: container,
                host,
                port,
            };

            // Build the migrated template, then close the pool so the template has
            // no open connections (required by CREATE DATABASE ... TEMPLATE).
            let mut admin = PgConnection::connect(&shared.url("postgres"))
                .await
                .expect("connect to admin database");
            sqlx::query(sqlx::AssertSqlSafe(format!(
                r#"CREATE DATABASE "{TEMPLATE_DB}""#
            )))
            .execute(&mut admin)
            .await
            .expect("create template database");
            admin.close().await.ok();

            let template = PgPool::connect(&shared.url(TEMPLATE_DB))
                .await
                .expect("connect to template database");
            sqlx::migrate!("./migrations")
                .run(&template)
                .await
                .expect("migrate template database");
            template.close().await;

            shared
        })
        .await
}

/// Clones a fresh, already-migrated database from the template and returns a pool.
pub async fn setup_db() -> PgPool {
    let shared = shared().await;
    let db = format!("test_{}", Uuid::now_v7().simple());

    {
        let _guard = CREATE_DB_LOCK.lock().await;
        let mut admin = PgConnection::connect(&shared.url("postgres"))
            .await
            .expect("connect to admin database");
        // `db` is a controlled `test_<uuid>` identifier; DDL can't take bind params.
        sqlx::query(sqlx::AssertSqlSafe(format!(
            r#"CREATE DATABASE "{db}" TEMPLATE "{TEMPLATE_DB}""#
        )))
        .execute(&mut admin)
        .await
        .expect("clone test database");
        admin.close().await.ok();
    }

    PgPool::connect(&shared.url(&db))
        .await
        .expect("connect to test database")
}

#[allow(dead_code)]
pub async fn fresh_database_url() -> String {
    let shared = shared().await;
    let db = format!("test_{}", Uuid::now_v7().simple());

    let _guard = CREATE_DB_LOCK.lock().await;
    let mut admin = PgConnection::connect(&shared.url("postgres"))
        .await
        .expect("connect to admin database");
    sqlx::query(sqlx::AssertSqlSafe(format!(r#"CREATE DATABASE "{db}""#)))
        .execute(&mut admin)
        .await
        .expect("create empty database");
    admin.close().await.ok();

    shared.url(&db)
}

/// Defines a Postgres-backed test. The body receives a `PgPool` connected to an
/// isolated, migrated database cloned from the per-binary template.
///
/// ```ignore
/// db_test! {
///     async fn inserts_a_row(pool: PgPool) { /* ... */ }
/// }
/// ```
macro_rules! db_test {
    (
        $(#[$meta:meta])*
        async fn $name:ident($pool:ident : PgPool) $body:block
    ) => {
        #[tokio::test]
        $(#[$meta])*
        async fn $name() {
            let $pool: sqlx::PgPool = $crate::common::setup_db().await;
            $body
        }
    };
}

pub(crate) use db_test;
