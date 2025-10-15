use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone)]
pub struct Exchange {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TickerData {
    pub last_pr: String,
    pub ts: String,
    pub inst_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IndexConfig {
    pub id: i32,
    pub name: String,
    pub formula: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Task {
    pub id: i64,
    pub exchange_name: String,
    pub symbol_ids: String, // "1,2,3"
    pub is_enabled: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Symbol {
    pub id: i64,
    pub symbol_name: String,
    pub exchange_name: String,
    pub third_symbol_name: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
