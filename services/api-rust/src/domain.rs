use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "VARCHAR", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountType {
    Asset,
    Liability,
    Equity,
    Revenue,
    Expense,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "VARCHAR", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Direction {
    Debit,
    Credit,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Account {
    pub id: Uuid,
    pub account_number: String,
    pub holder_name: String,
    pub account_type: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub account_number: String,
    pub holder_name: String,
    pub account_type: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub idempotency_key: String,
    pub description: String,
    pub posted_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct LedgerEntry {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub account_id: Uuid,
    pub direction: String,
    pub amount: i64, // In cents! (e.g. $10.00 = 1000)
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEntryRequest {
    pub account_id: Uuid,
    pub direction: String, // "DEBIT" or "CREDIT"
    pub amount: i64,       // In cents, must be > 0
}

#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub idempotency_key: String,
    pub description: String,
    pub entries: Vec<CreateEntryRequest>,
}

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub account_id: Uuid,
    pub balance_cents: i64,
    pub balance_formatted: String,
    pub total_debits: i64,
    pub total_credits: i64,
}
