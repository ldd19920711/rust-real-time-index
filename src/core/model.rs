use chrono::{DateTime, NaiveDateTime, Utc};
use rust_decimal::Decimal;
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

#[derive(Debug, Clone, FromRow)]
pub struct IndexData {
    pub id: Option<i64>,
    pub symbol: String,
    pub last: Decimal,
    pub formula: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct IndexKlineData {
    pub id: Option<i64>,
    pub symbol: String,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub ts: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl IndexKlineData {
    pub fn new(
        id: Option<i64>,
        symbol: String,
        open: Decimal,
        high: Decimal,
        low: Decimal,
        close: Decimal,
        ts: i64,
    ) -> IndexKlineData{
        Self {
            id,
            symbol,
            open,
            high,
            low,
            close,
            ts,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}


impl IndexData {
    pub fn new(
        id: Option<i64>,
        symbol: String,
        last: Decimal,
        formula: String,
    ) -> Self {
        Self {
            id,
            symbol,
            last,
            formula,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}
