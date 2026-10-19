use crate::domain::{
    Account, BalanceResponse, CreateAccountRequest, CreateTransactionRequest, Transaction,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use sqlx::PgPool;
use tracing::{error, info, warn};
use uuid::Uuid;

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "healthy", "service": "ledger-api-rust" })))
}

pub async fn create_account(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateAccountRequest>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let id = Uuid::new_v4();

    let account = sqlx::query_as::<_, Account>(
        r#"
        INSERT INTO accounts (id, account_number, holder_name, account_type)
        VALUES ($1, $2, $3, $4)
        RETURNING id, account_number, holder_name, account_type, status, created_at
        "#,
    )
    .bind(id)
    .bind(&payload.account_number)
    .bind(&payload.holder_name)
    .bind(&payload.account_type.to_uppercase())
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        error!("Failed to create account: {:?}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Failed to create account", "details": e.to_string() })),
        )
    })?;

    info!("Created account: {} ({})", account.account_number, account.id);
    Ok((StatusCode::CREATED, Json(account)).into_response())
}

pub async fn list_accounts(
    State(pool): State<PgPool>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let accounts = sqlx::query_as::<_, Account>(
        "SELECT id, account_number, holder_name, account_type, status, created_at FROM accounts ORDER BY created_at DESC LIMIT 50",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        error!("Failed to list accounts: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database query failed" })))
    })?;

    Ok(Json(accounts).into_response())
}

pub async fn create_transaction(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateTransactionRequest>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    if payload.entries.len() < 2 {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({ "error": "A transaction must have at least 2 entries (Double-Entry rule)" })),
        ));
    }

    // 1. Invariant Validation: Sum of Debits MUST equal Sum of Credits
    let mut total_debit: i64 = 0;
    let mut total_credit: i64 = 0;

    for entry in &payload.entries {
        if entry.amount <= 0 {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({ "error": "Entry amount must be greater than zero" })),
            ));
        }

        match entry.direction.to_uppercase().as_str() {
            "DEBIT" => total_debit += entry.amount,
            "CREDIT" => total_credit += entry.amount,
            other => {
                return Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(json!({ "error": format!("Invalid direction '{}'. Must be DEBIT or CREDIT", other) })),
                ))
            }
        }
    }

    if total_debit != total_credit {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "error": "Double-Entry invariant violation: Sum of Debits must equal Sum of Credits",
                "total_debits": total_debit,
                "total_credits": total_credit,
                "imbalance": total_debit - total_credit
            })),
        ));
    }

    // 2. Check Idempotency Key
    let existing = sqlx::query_as::<_, Transaction>(
        "SELECT id, idempotency_key, description, posted_at, created_at FROM transactions WHERE idempotency_key = $1",
    )
    .bind(&payload.idempotency_key)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        error!("Idempotency check query failed: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" })))
    })?;

    if let Some(tx) = existing {
        warn!("Idempotency key '{}' already processed. Returning original transaction.", payload.idempotency_key);
        return Ok((StatusCode::OK, Json(tx)).into_response());
    }

    // 3. Begin PostgreSQL ACID Transaction
    let mut tx = pool.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to begin transaction" })))
    })?;

    let tx_id = Uuid::new_v4();

    let created_tx = sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (id, idempotency_key, description)
        VALUES ($1, $2, $3)
        RETURNING id, idempotency_key, description, posted_at, created_at
        "#,
    )
    .bind(tx_id)
    .bind(&payload.idempotency_key)
    .bind(&payload.description)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to insert transaction: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to record transaction", "details": e.to_string() })))
    })?;

    for entry in &payload.entries {
        let entry_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO ledger_entries (id, transaction_id, account_id, direction, amount)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(entry_id)
        .bind(tx_id)
        .bind(entry.account_id)
        .bind(&entry.direction.to_uppercase())
        .bind(entry.amount)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to insert ledger entry: {:?}", e);
            (StatusCode::BAD_REQUEST, Json(json!({ "error": "Failed to insert ledger entry", "details": e.to_string() })))
        })?;
    }

    // Commit transaction atomically
    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Commit failed" })))
    })?;

    info!("Processed transaction {} for {} cents", tx_id, total_credit);
    Ok((StatusCode::CREATED, Json(created_tx)).into_response())
}

pub async fn get_account_balance(
    State(pool): State<PgPool>,
    Path(account_id): Path<Uuid>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    // Balance calculation:
    // Credits add to balance, Debits subtract from balance
    let row: (Option<i64>, Option<i64>) = sqlx::query_as(
        r#"
        SELECT 
            COALESCE(SUM(CASE WHEN direction = 'DEBIT' THEN amount ELSE 0 END), 0)::bigint AS total_debits,
            COALESCE(SUM(CASE WHEN direction = 'CREDIT' THEN amount ELSE 0 END), 0)::bigint AS total_credits
        FROM ledger_entries
        WHERE account_id = $1
        "#,
    )
    .bind(account_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        error!("Failed to calculate balance: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Query failed" })))
    })?;

    let debits = row.0.unwrap_or(0);
    let credits = row.1.unwrap_or(0);
    let balance_cents = credits - debits;

    let balance_response = BalanceResponse {
        account_id,
        balance_cents,
        balance_formatted: format!("${:.2}", (balance_cents as f64) / 100.0),
        total_debits: debits,
        total_credits: credits,
    };

    Ok(Json(balance_response).into_response())
}
