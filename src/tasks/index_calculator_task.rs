use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::{sleep, Duration, Instant};
use crate::core::db::config_repository::ConfigRepository;
use crate::core::index::calculator_manager::CalculatorManager;
use crate::core::model::{IndexConfig, IndexData, IndexKlineData, KlineInterval};
use rust_decimal::Decimal;

use std::collections::hash_map::Entry;
use tracing::{debug, error};

pub async fn run_index_calculator(
    calculators: Arc<CalculatorManager>,
    index_configs: Vec<IndexConfig>,
    config_repo: Arc<ConfigRepository>,
    kline_sender: UnboundedSender<IndexKlineData>,
    intervals: Vec<KlineInterval>, // 支持多周期
) {
    // 每个指数的周期 OHLC (aligned_ts, open, high, low, close)
    let mut ohlc_map: HashMap<(String, KlineInterval), (i64, Decimal, Decimal, Decimal, Decimal)> = HashMap::new();
    // 记录每个周期上次发送的 close
    let mut last_sent: HashMap<(String, KlineInterval), Decimal> = HashMap::new();
    // 每个 config.name 上次持久化的 5 秒分组
    let mut last_groups: HashMap<String, i64> = HashMap::new();

    loop {
        sleep(Duration::from_secs(1)).await;
        let loop_start = Instant::now(); // 记录循环开始时间

        let now = chrono::Utc::now();
        let now_timestamp = now.timestamp();
        let index_id = now.timestamp_millis();

        for config in &index_configs {
            let calcs = calculators.calculators.read().await;
            if let Some(calc) = calcs.get(&config.name) {
                if let Some(idx) = calc.calculate_index(
                    &config.name,
                    &config.formula,
                    now_timestamp as u64,
                ) {
                    debug!(
                        "Index {} = {} , formula: {}, computed_formula: {}",
                        idx.symbol, idx.last, idx.formula, idx.computed_formula
                    );

                    // ---------------- 多周期 K 线 ----------------
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
                                    // 跨周期
                                    let prev_kline = IndexKlineData::new(
                                        Some(entry.0),
                                        idx.symbol.clone(),
                                        interval,
                                        entry.1, entry.2, entry.3, entry.4,
                                        entry.0,
                                    );
                                    let _ = kline_sender.send(prev_kline);

                                    // 初始化新周期 OHLC
                                    let last_close = entry.4; // 上一个周期的 close
                                    *entry = (aligned_ts, last_close, last_close, last_close, last_close);

                                    // 立即发送新周期初始 K 线
                                    let init_kline = IndexKlineData::new(
                                        Some(aligned_ts),
                                        idx.symbol.clone(),
                                        interval,
                                        last_close, last_close, last_close, last_close,
                                        aligned_ts,
                                    );
                                    let _ = kline_sender.send(init_kline);
                                }
                            }
                            Entry::Vacant(vac) => {
                                // 第一次出现该指数周期，初始化 OHLC
                                vac.insert((aligned_ts, idx.last, idx.last, idx.last, idx.last));

                                let init_kline = IndexKlineData::new(
                                    Some(aligned_ts),
                                    idx.symbol.clone(),
                                    interval,
                                    idx.last, idx.last, idx.last, idx.last,
                                    aligned_ts,
                                );
                                let _ = kline_sender.send(init_kline);
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


                    // ---------------- 每5秒持久化 ----------------
                    let group = now_timestamp / 5;
                    let config_name_clone = String::from(&config.name);
                    let last_group = last_groups.entry(config_name_clone.clone()).or_insert(0);

                    if group != *last_group {
                        *last_group = group;

                        let config_repo = config_repo.clone();
                        let index_data = IndexData::new(
                            Some(index_id),
                            idx.symbol.clone(),
                            idx.last,
                            idx.formula.clone(),
                        );
                        debug!("Persisting index_data for {}: {:?}", &config_name_clone, index_data);

                        tokio::spawn(async move {
                            if let Err(e) = config_repo
                                .insert_index_data(&format!("index_data_{}", &config_name_clone), &index_data)
                                .await
                            {
                                error!("Error saving index data: {:?}", e);
                            }
                        });
                    }
                }
            }
        }

        let loop_duration = loop_start.elapsed();
        if loop_duration.as_millis() > 50 {
            println!("run_index_calculator loop took: {:?}", loop_duration);
        }
    }
}
