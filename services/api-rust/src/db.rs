use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use tracing::info;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    info!("Connecting to PostgreSQL connection pool...");
    let pool = PgPoolOptions::new()
        .max_connections(25)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(600))
        .connect(database_url)
        .await?;

    info!("Initializing Ledger database schema...");
    init_schema(&pool).await?;

    Ok(pool)
}

async fn init_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    // NOTE: `raw_sql` uses the simple query protocol, which allows sending
    // multiple DDL statements in a single round-trip. `sqlx::query` would use
    // the extended protocol (prepared statement) and Postgres rejects it with
    // "cannot insert multiple commands into a prepared statement" (42601).
    //
    // The advisory lock serialises concurrent initialisation. When several
    // replicas start against a fresh database they all run this block at the
    // same time, and `CREATE TABLE IF NOT EXISTS` is *not* safe under
    // concurrency: two sessions both see the table as missing, both try to
    // create it, and one fails with
    // "duplicate key value violates unique constraint pg_type_typname_nsp_index".
    // The lock is transaction-scoped, so Postgres releases it when the implicit
    // transaction of this simple query commits (or the session dies).
    sqlx::raw_sql(
        r#"
        -- Arbitrary constant, unique to this schema initialisation.
        SELECT pg_advisory_xact_lock(20261018);

        CREATE TABLE IF NOT EXISTS accounts (
            id UUID PRIMARY KEY,
            account_number VARCHAR(32) NOT NULL UNIQUE,
            holder_name VARCHAR(128) NOT NULL,
            account_type VARCHAR(16) NOT NULL,
            status VARCHAR(16) NOT NULL DEFAULT 'ACTIVE',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS transactions (
            id UUID PRIMARY KEY,
            idempotency_key VARCHAR(64) NOT NULL UNIQUE,
            description VARCHAR(255) NOT NULL,
            posted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS ledger_entries (
            id UUID PRIMARY KEY,
            transaction_id UUID NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
            account_id UUID NOT NULL REFERENCES accounts(id),
            direction VARCHAR(8) NOT NULL,
            amount BIGINT NOT NULL CHECK (amount > 0),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );

        CREATE INDEX IF NOT EXISTS idx_ledger_entries_account_id ON ledger_entries (account_id);
        CREATE INDEX IF NOT EXISTS idx_ledger_entries_created_at ON ledger_entries (created_at);
        CREATE INDEX IF NOT EXISTS idx_transactions_posted_at ON transactions (posted_at);
        "#,
    )
    .execute(pool)
    .await?;

    info!("Database schema initialized successfully.");
    Ok(())
}
