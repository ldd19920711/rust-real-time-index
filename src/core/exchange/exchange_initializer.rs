use crate::core::model::Symbol;
use crate::core::trade::trade_repository::TradeRepository;
use crate::exchanges::ExchangeEnum;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use crate::core::exchange::exchange_factory::ExchangeFactory;
use crate::core::exchange::exchange_manager::ExchangeManager;

pub struct ExchangeInitializer;

impl ExchangeInitializer {
    pub async fn init(
        tasks_symbols_map: &HashMap<ExchangeEnum, Vec<Symbol>>,
        manager: Arc<ExchangeManager>,
        trade_repo: Arc<TradeRepository>,
    ) {
        for (exch, symbols) in tasks_symbols_map {
            let symbol_map: HashMap<String, String> = symbols
                .iter()
                .map(|s| (s.third_symbol_name.clone(), s.symbol_name.clone()))
                .collect();
            let client = ExchangeFactory::create(exch.clone(), trade_repo.clone(), symbol_map);
            let symbol_names: HashSet<String> =
                symbols.iter().map(|s| s.third_symbol_name.clone()).collect();
            manager.add_exchange(exch.clone(), client, symbol_names).await;
        }
    }
}
