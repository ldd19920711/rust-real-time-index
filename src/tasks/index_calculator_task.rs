use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::{sleep, Duration};
use crate::core::db::config_repository::ConfigRepository;
use crate::core::index::calculator_manager::CalculatorManager;
use crate::core::model::{IndexConfig, IndexData, IndexKlineData, KlineInterval};
use rust_decimal::Decimal;

use std::collections::hash_map::Entry;

pub async fn run_index_calculator(
    calculators: Arc<CalculatorManager>,
    index_configs: Vec<IndexConfig>,
    config_repo: Arc<ConfigRepository>,
    kline_sender: UnboundedSender<IndexKlineData>,
    intervals: Vec<KlineInterval>, // 支持多周期
) {
    // 每个指数的分钟 OHLC (minute_key_seconds, open, high, low, close)
    let mut ohlc_map: HashMap<(String, KlineInterval), (i64, Decimal, Decimal, Decimal, Decimal)> = HashMap::new();
    let mut last_group: i64 = 0; // 上次 5 秒持久化组

    loop {
        sleep(Duration::from_secs(1)).await;

        let now = chrono::Utc::now();
        let now_timestamp = now.timestamp();
        let index_id = now.timestamp_millis();

        for config in &index_configs {
            // 获取 calculator
            let calcs = calculators.calculators.read().await;
            if let Some(calc) = calcs.get(&config.name) {
                if let Some(idx) = calc.calculate_index(
                    &config.name,
                    &config.formula,
                    now_timestamp as u64,
                ) {
                    // println!(
                    //     "Index {} = {} , formula: {}, computed_formula: {}",
                    //     idx.symbol, idx.last, idx.formula, idx.computed_formula
                    // );

                    for &interval in &intervals {
                        let interval_sec = interval.seconds();
                        let interval_key = (idx.symbol.clone(), interval);
                        let aligned_ts = (now_timestamp / interval_sec) * interval_sec;

                        match ohlc_map.entry(interval_key.clone()) {
                            Entry::Occupied(mut occ) => {
                                let entry = occ.get_mut();
                                if entry.0 == aligned_ts {
                                    // 同周期，更新 OHLC
                                    entry.2 = entry.2.max(idx.last); // high
                                    entry.3 = entry.3.min(idx.last); // low
                                    entry.4 = idx.last;              // close
                                } else {
                                    // 跨周期，发送上一周期最终 K 线
                                    let prev_kline = IndexKlineData::new(
                                        Some(entry.0),
                                        idx.symbol.clone(),
                                        interval,
                                        entry.1, entry.2, entry.3, entry.4,
                                        entry.0,
                                    );
                                    let _ = kline_sender.send(prev_kline);

                                    // 重置为当前周期
                                    *entry = (aligned_ts, idx.last, idx.last, idx.last, idx.last);
                                }
                            }
                            Entry::Vacant(vac) => {
                                vac.insert((aligned_ts, idx.last, idx.last, idx.last, idx.last));
                            }
                        }

                        // 持续更新当前周期 K 线
                        let current_kline = ohlc_map.get(&interval_key).unwrap();
                        let kline = IndexKlineData::new(
                            Some(current_kline.0),
                            idx.symbol.clone(),
                            interval,
                            current_kline.1,
                            current_kline.2,
                            current_kline.3,
                            current_kline.4,
                            current_kline.0,
                        );
                        let _ = kline_sender.send(kline);
                    }

                    // ---------- 每5秒持久化 ----------
                    let group = now_timestamp / 5;
                    if group != last_group {
                        last_group = group;

                        let config_repo = config_repo.clone();
                        let index_data = IndexData::new(
                            Some(index_id),
                            idx.symbol.clone(),   // 用 config.name
                            idx.last,
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
}
