use crate::core::model::Symbol;
use crate::exchanges::ExchangeEnum;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use crate::core::exchange::exchange_manager::ExchangeManager;

pub async fn run_market_printer(
    manager: Arc<ExchangeManager>,
    task_symbols_map: Arc<HashMap<ExchangeEnum, Vec<Symbol>>>,
) {
    loop {
        sleep(Duration::from_secs(5)).await;

        for (exch, symbols) in task_symbols_map.iter() {
            if let Some(client) = manager.get_client(exch) {
                for symbol in symbols {
                    if let Some(ticker) = client.get_ticker(&symbol.symbol_name) {
                        // println!(
                        //     "[Market] {} {} 最新价: {}",
                        //     exch.name(),
                        //     symbol.symbol_name,
                        //     ticker.last_pr
                        // );
                    }
                }
            }
        }
    }
}
