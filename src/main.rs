mod app;
mod core;
mod exchanges;
mod tasks;

use std::sync::Arc;
use crate::core::db::db_config::init_pool_for_postgres;
use app::App;
use crate::core::db::config_repository::ConfigRepository;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();
    let pool = init_pool_for_postgres().await?;
    let config_repo = ConfigRepository::new(pool.clone());
    let index_configs = config_repo.get_active_configs().await?;
    let app = App::new(pool.clone()).await?;
    let config_repo_arc = Arc::new(config_repo);
    let (kline_tx, kline_rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(tasks::kline_saver::start_kline_saver(kline_rx, config_repo_arc.clone()));
    app.run(index_configs, config_repo_arc.clone(), kline_tx).await;
    Ok(())
}
