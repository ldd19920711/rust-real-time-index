use crate::core::index::calculator_manager::CalculatorManager;
use crate::core::model::Symbol;
use crate::exchanges::ExchangeEnum;
use std::collections::HashMap;
use std::sync::Arc;
use rust_decimal::Decimal;
use crate::core::exchange::exchange_manager::ExchangeManager;

pub async fn run_price_updater(
    manager: Arc<ExchangeManager>,
    calculators: Arc<CalculatorManager>,
    task_symbols_map: Arc<HashMap<ExchangeEnum, Vec<Symbol>>>,
) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        for (exch, symbols) in task_symbols_map.iter() {
            if let Some(client) = manager.get_client(exch) {
                for symbol in symbols {
                    if let Some(t) = client.get_ticker(&symbol.symbol_name) {
                        let price = t.last_pr.parse::<Decimal>().unwrap_or(Decimal::ZERO);
                        calculators.update_price(&symbol.symbol_name, &format!("{}.{}", exch.name(), &symbol.symbol_name), price).await;
                    }
                }
            }
        }
    }
}
