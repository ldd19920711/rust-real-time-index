use std::sync::Arc;
use tokio::sync::mpsc;
use chrono::Utc;
use rust_decimal::Decimal;
use tokio::task::id;
use crate::core::index::calculator_manager::CalculatorManager;
use crate::core::model::{IndexConfig, IndexKlineData};

pub async fn run_index_calculator(
    calculators: Arc<CalculatorManager>,
    index_configs: Vec<IndexConfig>,
    kline_sender: mpsc::UnboundedSender<IndexKlineData>,
) {
    // 每个symbol上一分钟的时间戳
    let mut last_ts_map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        for config in &index_configs {
            let mut calcs = calculators.calculators.write().await;
            if let Some(calc) = calcs.get_mut(&config.name) {
                if let Some(idx) =
                    calc.calculate_index(&config.name, &config.formula, Utc::now().timestamp() as u64)
                {
                    let minute_ts = idx.id / 60;
                    let last_ts = last_ts_map.get(&idx.symbol).copied().unwrap_or(0) as i64;

                    // 构造 IndexKline
                    let kline = if last_ts == minute_ts as i64 {
                        // 同一分钟，更新高低收
                        IndexKlineData::new(
                            Some(1),
                            idx.symbol.clone(),
                            // calc.get_open(&idx.symbol).unwrap_or(idx.last),
                            // calc.get_high(&idx.symbol).unwrap_or(idx.last).max(idx.last),
                            // calc.get_low(&idx.symbol).unwrap_or(idx.last).min(idx.last),
                            idx.last,
                            idx.last,
                            idx.last,
                            idx.last,
                            minute_ts as i64,
                        )
                    } else {
                        // 新一分钟，重置 OHLC
                        IndexKlineData::new(
                            Some(1),
                            idx.symbol.clone(),
                            idx.last,
                            idx.last,
                            idx.last,
                            idx.last,
                            minute_ts as i64,
                        )
                    };

                    let _ = kline_sender.send(kline);
                    last_ts_map.insert(idx.symbol.clone(), minute_ts as i64);
                }
            }
        }
    }
}
