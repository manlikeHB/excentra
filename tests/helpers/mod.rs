use dotenvy::dotenv;
use sqlx::{Connection, PgPool, postgres::PgPoolOptions};
use uuid::Uuid;

/// Holds everything a single integration test needs.
/// When this value is dropped (end of test, or panic), the schema is
/// automatically deleted — so tests never leave garbage behind.
pub struct TestContext {
    /// The pool your test code uses to talk to the DB.
    pub pool: PgPool,

    /// Name of the isolated schema created for this test run,
    /// e.g. "test_3f2a1b4c...". Kept so Drop knows what to delete.
    schema_name: String,

    /// A plain connection to `excentra_test` (no schema override).
    /// We need a separate handle for DROP SCHEMA because the main pool
    /// may refuse new queries while it's being shut down.
    cleanup_pool: PgPool,
}

impl TestContext {
    pub async fn new() -> Self {
        dotenv().ok();

        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run integration tests");

        let (prefix, _db_name) = database_url
            .rsplit_once('/')
            .expect("DATABASE_URL has no '/' — is it a valid Postgres URL?");

        let base_url = format!("{}/postgres", prefix);
        let test_db_url = format!("{}/excentra_test", prefix);

        //    "CREATE DATABASE" cannot run inside a transaction, so we use
        //    a plain single connection rather than a pool here.
        let mut base_conn = sqlx::postgres::PgConnection::connect(&base_url)
            .await
            .expect("Failed to connect to postgres maintenance DB");

        // It's fine if the DB already exists — just ignore that error.
        let _ = sqlx::query("CREATE DATABASE excentra_test")
            .execute(&mut base_conn)
            .await;

        // Build the schema name for this particular test run.
        // .simple() gives us a compact hex string with no hyphens
        let schema_name = format!("test_{}", Uuid::new_v4().simple());

        //  Build the test pool.
        //    after_connect runs our SET search_path on *every* connection
        //    that the pool hands out, so all queries — including those
        //    inside sqlx::migrate! — land in the right schema.
        let schema_for_connect = schema_name.clone();
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .after_connect(move |conn, _meta| {
                // after_connect must return a pinned, boxed future.
                let schema = schema_for_connect.clone();
                Box::pin(async move {
                    sqlx::query(&format!("SET search_path TO {}", schema))
                        .execute(conn)
                        .await?;
                    Ok(())
                })
            })
            .connect(&test_db_url)
            .await
            .expect("Failed to connect to excentra_test");

        // Create the schema, then run migrations into it.
        sqlx::query(&format!("CREATE SCHEMA {}", schema_name))
            .execute(&pool)
            .await
            .expect("Failed to create test schema");

        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        // Build a plain cleanup pool (no search_path override).
        let cleanup_pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&test_db_url)
            .await
            .expect("Failed to create cleanup pool");

        TestContext {
            pool,
            schema_name,
            cleanup_pool,
        }
    }
}

// Drop: runs when the TestContext goes out of scope — even on panic.
//
// The problem: Drop is a *synchronous* trait method, but we need to
// run an async query (DROP SCHEMA)
impl Drop for TestContext {
    fn drop(&mut self) {
        let schema_name = self.schema_name.clone();
        let cleanup_pool = self.cleanup_pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                // CASCADE drops all tables inside the schema too.
                let _ = sqlx::query(&format!("DROP SCHEMA {} CASCADE", schema_name))
                    .execute(&cleanup_pool)
                    .await;
            });
        })
    }
}
