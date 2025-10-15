use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::{sleep, Duration};
use crate::core::db::config_repository::ConfigRepository;
use crate::core::index::calculator_manager::CalculatorManager;
use crate::core::model::{IndexConfig, IndexData, IndexKlineData};

pub async fn run_index_calculator(
    calculators: Arc<CalculatorManager>,
    index_configs: Vec<IndexConfig>,
    config_repo: Arc<ConfigRepository>,
    kline_sender: UnboundedSender<IndexKlineData>,
) {
    // 存储每个 symbol 的分钟 OHLC
    let mut ohlc_map: HashMap<String, (i64, rust_decimal::Decimal, rust_decimal::Decimal, rust_decimal::Decimal, rust_decimal::Decimal)> = HashMap::new();

    loop {
        sleep(Duration::from_secs(1)).await;

        for config in &index_configs {
            let mut calcs = calculators.calculators.write().await;
            if let Some(calc) = calcs.get_mut(&config.name) {
                if let Some(idx) = calc.calculate_index(
                    &config.name,
                    &config.formula,
                    chrono::Utc::now().timestamp() as u64,
                ) {
                    println!(
                        "Index {} = {} , formula: {}, computed_formula: {}",
                        idx.symbol, idx.last, idx.formula, idx.computed_formula
                    );

                    let now = chrono::Utc::now();
                    let index_id = now.timestamp_millis();
                    let minute_key = now.timestamp() / 60; // 当前分钟

                    // ---------- 更新 OHLC ----------
                    let entry = ohlc_map.entry(idx.symbol.clone()).or_insert((
                        minute_key,
                        idx.last, // open
                        idx.last, // high
                        idx.last, // low
                        idx.last, // close
                    ));

                    if entry.0 == minute_key {
                        entry.2 = entry.2.max(idx.last); // high
                        entry.3 = entry.3.min(idx.last); // low
                        entry.4 = idx.last;              // close
                    } else {
                        // 新一分钟，发送上一分钟的K线
                        let kline = IndexKlineData::new(
                            Some(minute_key),
                            idx.symbol.clone(),
                            entry.1,
                            entry.2,
                            entry.3,
                            entry.4,
                            entry.0,
                        );
                        let _ = kline_sender.send(kline);

                        // 重置为当前分钟
                        *entry = (minute_key, idx.last, idx.last, idx.last, idx.last);
                    }

                    // ---------- 每5秒持久化 ----------
                    if index_id % 5000 != 0 {
                        continue;
                    }

                    let config_repo = config_repo.clone();
                    let index_data = IndexData::new(
                        Some(index_id),
                        idx.symbol.clone(),
                        idx.last.clone(),
                        idx.formula.clone(),
                    );
                    if let Err(e) = config_repo
                        .insert_index_data(&format!("index_data_{}", &config.name), &index_data)
                        .await
                    {
                        eprintln!("Error saving index data: {:?}", e);
                    }
                }
            }
        }
    }
}
