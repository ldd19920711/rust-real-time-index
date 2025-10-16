use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tokio::sync::RwLock;
use crate::core::trade::trade_repository::TradeRepository;

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
    pub interval: KlineInterval,      // 使用枚举
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
        interval: KlineInterval,
        open: Decimal,
        high: Decimal,
        low: Decimal,
        close: Decimal,
        ts: i64,
    ) -> IndexKlineData{
        Self {
            id,
            symbol,
            interval,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "varchar")] // 对应数据库的 VARCHAR 类型
#[derive(Hash)]
pub enum KlineInterval {
    #[sqlx(rename = "1m")]
    OneMinute,
    #[sqlx(rename = "5m")]
    FiveMinutes,
    #[sqlx(rename = "15m")]
    FifteenMinutes,
    #[sqlx(rename = "1h")]
    OneHour,
    #[sqlx(rename = "4h")]
    FourHours,
    #[sqlx(rename = "1d")]
    OneDay,
}

impl Display for KlineInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            KlineInterval::OneMinute => "1m".to_string(),
            KlineInterval::FiveMinutes => "5m".to_string(),
            KlineInterval::FifteenMinutes => "15m".to_string(),
            KlineInterval::OneHour => "1h".to_string(),
            KlineInterval::FourHours => "4h".to_string(),
            KlineInterval::OneDay => "1d".to_string(),
        };
        write!(f, "{}", str)
    }
}

impl KlineInterval {

    pub(crate) fn seconds(&self) -> i64 {
        match self {
            KlineInterval::OneMinute => 60,
            KlineInterval::FiveMinutes => 60 * 5,
            KlineInterval::FifteenMinutes => 60 * 15,
            KlineInterval::OneHour => 60 * 60,
            KlineInterval::FourHours => 60 * 60 * 4,
            KlineInterval::OneDay => 60 * 60 * 24,
        }
    }
}


#[async_trait::async_trait]
pub trait ExchangeWsHandler: Send + Sync {
    /// 返回交易所名称
    fn name(&self) -> &str;

    /// 构建订阅消息（不同交易所格式不同）
    fn build_sub_msg(&self, symbols: &HashSet<String>) -> Option<String>;

    /// 解析行情消息（不同交易所格式不同）
    async fn handle_message(
        &self,
        text: &str,
        store: Arc<DashMap<String, TickerData>>,
        trade_repo: Arc<TradeRepository>,
        symbol_map: Arc<RwLock<HashMap<String, String>>>,
    );
}
