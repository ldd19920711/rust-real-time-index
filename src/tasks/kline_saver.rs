use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;
use sqlx::PgPool;
use crate::core::db::config_repository::ConfigRepository;
use crate::core::model::IndexKlineData;

pub async fn start_kline_saver(
    mut rx: UnboundedReceiver<IndexKlineData>,
    config_repository: Arc<ConfigRepository>,
) {
    // 每个 symbol 启动一个独立任务
    let mut symbol_channels: HashMap<String, tokio::sync::mpsc::UnboundedSender<IndexKlineData>> = HashMap::new();

    while let Some(kline) = rx.recv().await {
        let symbol = kline.symbol.clone();
        if let Some(tx) = symbol_channels.get(&symbol) {
            let _ = tx.send(kline);
        } else {
            // 新 symbol：创建 channel + spawn 处理任务
            let (tx, mut sub_rx) = tokio::sync::mpsc::unbounded_channel();
            tx.send(kline.clone()).ok();
            let config_repo = config_repository.clone();
            let sym = symbol.clone();

            tokio::spawn(async move {
                while let Some(k) = sub_rx.recv().await {
                    let table_name = format!("index_kline_{}", sym.to_lowercase());
                    if let Err(e) = config_repo.insert_index_kline_data(&k).await {
                        eprintln!("Failed to insert kline for {}: {:?}", sym, e);
                    }
                }
            });

            symbol_channels.insert(symbol.clone(), tx);
        }
    }
}
