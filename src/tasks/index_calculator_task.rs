use crate::core::index::calculator_manager::CalculatorManager;
use crate::core::model::IndexConfig;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

pub async fn run_index_calculator(
    calculators: Arc<CalculatorManager>,
    index_configs: Vec<IndexConfig>,
) {
    loop {
        sleep(Duration::from_secs(1)).await;

        // 遍历所有指数配置进行计算
        for config in &index_configs {
            let mut calcs = calculators.calculators.write().await;
            if let Some(calc) = calcs.get_mut(&config.name) {
                if let Some(idx) =
                    calc.calculate_index(&config.name, &config.formula, chrono::Utc::now().timestamp() as u64)
                {
                    println!(
                        "Index {} = {} , formula: {}, computed_formula: {}",
                        idx.symbol, idx.last, idx.formula, idx.computed_formula
                    );
                }
            }
        }
    }
}
