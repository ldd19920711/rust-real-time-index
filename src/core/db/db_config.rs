use std::env;
use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;

pub async fn init_pool_for_postgres() -> anyhow::Result<Pool<Postgres>> {
    let db_host = env::var("DB_HOST")?;
    let db_port = env::var("DB_PORT")?;
    let db_name = env::var("DB_NAME")?;
    let db_user = env::var("DB_USER")?;
    let db_password = env::var("DB_PASSWORD")?;
    let db_max_connections = env::var("DB_MAX_CONNECTIONS")?.parse::<u32>()?;

    let pool = PgPoolOptions::new()
        .max_connections(db_max_connections)
        .connect(&format!(
            "postgres://{}:{}@{}:{}/{}",
            db_user, db_password, db_host, db_port, db_name
        ))
        .await?;
    Ok(pool)
}