use sqlx::{PgPool, Result};
use crate::core::model::{IndexConfig, IndexData, IndexKlineData, Symbol, Task};

pub struct ConfigRepository {
    pool: PgPool,
}

impl ConfigRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 获取所有启用的指数配置
    pub async fn get_active_configs(&self) -> Result<Vec<IndexConfig>> {
        let configs = sqlx::query_as::<_, IndexConfig>(
            "SELECT * FROM index_config WHERE is_active = TRUE ORDER BY id",
        )
            .fetch_all(&self.pool)
            .await?;
        Ok(configs)
    }

    /// 新增一个配置
    pub async fn insert_config(&self, name: &str, formula: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO index_config (name, formula) VALUES ($1, $2)
             ON CONFLICT (name) DO UPDATE SET formula = EXCLUDED.formula, updated_at = now()",
        )
            .bind(name)
            .bind(formula)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 删除一个配置
    pub async fn delete_config(&self, name: &str) -> Result<()> {
        sqlx::query(
            "DELETE FROM index_config WHERE name = $1",
        )
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 获取所有启用任务
    pub async fn get_enabled_tasks(&self) -> Result<Vec<Task>> {
        let tasks = sqlx::query_as::<_, Task>(
            "SELECT * FROM task WHERE is_enabled = TRUE ORDER BY id",
        )
            .fetch_all(&self.pool)
            .await?;
        Ok(tasks)
    }

    /// 新增或更新任务
    pub async fn upsert_task(&self, exchange_name: &str, symbol_ids: &str, is_enabled: bool) -> Result<()> {
        sqlx::query(
            "INSERT INTO task (exchange_name, symbol_ids, is_enabled, created_at, updated_at)
             VALUES ($1, $2, $3, now(), now())
             ON CONFLICT (exchange_name)
             DO UPDATE SET symbol_ids = EXCLUDED.symbol_ids, is_enabled = EXCLUDED.enabled, updated_at = now()",
        )
            .bind(exchange_name)
            .bind(symbol_ids)
            .bind(is_enabled)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 删除任务
    pub async fn delete_task(&self, exchange_name: &str) -> Result<()> {
        sqlx::query("DELETE FROM task WHERE exchange_name = $1")
            .bind(exchange_name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // ------------------ symbol ------------------

    /// 根据 id 列表和交易所获取 symbol
    pub async fn get_symbols_by_ids(&self, ids: &[i32], exchange_name: &str) -> Result<Vec<Symbol>> {
        let symbols = sqlx::query_as::<_, Symbol>(
            "SELECT * FROM symbol WHERE id = ANY($1) AND exchange_name = $2"
        )
            .bind(ids)
            .bind(exchange_name)
            .fetch_all(&self.pool)
            .await?;
        Ok(symbols)
    }

    /// 新增或更新 symbol
    pub async fn upsert_symbol(&self, symbol_name: &str, exchange_name: &str, third_symbol_name: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO symbol (symbol_name, exchange_name, third_symbol_name)
             VALUES ($1, $2, $3)
             ON CONFLICT (symbol_name, exchange_name)
             DO UPDATE SET third_symbol_name = EXCLUDED.third_symbol_name"
        )
            .bind(symbol_name)
            .bind(exchange_name)
            .bind(third_symbol_name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 删除 symbol
    pub async fn delete_symbol(&self, symbol_name: &str, exchange_name: &str) -> Result<()> {
        sqlx::query("DELETE FROM symbol WHERE symbol_name = $1 AND exchange_name = $2")
            .bind(symbol_name)
            .bind(exchange_name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

}

impl ConfigRepository{

    pub async fn insert_index_data(
        &self,
        table_name: &str, // 动态表名
        index_data: &IndexData,
    ) -> Result<()> {
        // 注意：表名直接拼接，需要保证安全性，防止 SQL 注入
        let sql = format!(
            r#"
            INSERT INTO {} (id, symbol, last, formula, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO UPDATE
              SET last = EXCLUDED.last,
                  formula = EXCLUDED.formula,
                  updated_at = EXCLUDED.updated_at
            "#,
            table_name
        );

        sqlx::query(&sql)
            .bind(index_data.id)
            .bind(&index_data.symbol)
            .bind(&index_data.last)
            .bind(&index_data.formula)
            .bind(index_data.created_at)
            .bind(index_data.updated_at)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

impl ConfigRepository {
    pub async fn insert_index_kline_data(&self,
                        index_data: &IndexKlineData,) -> anyhow::Result<()> {
        let sql = r#"
            INSERT INTO index_kline_data (id, symbol, open, high, low, close, ts, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO UPDATE
              SET open = EXCLUDED.open,
                  high = EXCLUDED.high,
                  low = EXCLUDED.low,
                  close = EXCLUDED.close,
                  updated_at = EXCLUDED.updated_at
            "#;
        sqlx::query(&sql)
            .bind(&index_data.id)
            .bind(&index_data.symbol)
            .bind(&index_data.open)
            .bind(&index_data.high)
            .bind(&index_data.low)
            .bind(&index_data.close)
            .bind(index_data.ts)
            .bind(index_data.created_at)
            .bind(index_data.updated_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}