use crate::core::model::{IndexKlineData, KlineInterval, Symbol};
use crate::core::index::index_calculator::IndexCalculator;
use crate::exchanges::ExchangeEnum;
use crate::tasks::{index_calculator_task, market_printer, price_updater};

use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use crate::core::db::config_repository::ConfigRepository;
use crate::core::exchange::exchange_factory::ExchangeFactory;
use crate::core::exchange::exchange_manager::ExchangeManager;
use crate::core::index::calculator_manager::CalculatorManager;
use crate::core::trade::trade_repository::TradeRepository;

pub struct App {
    pub manager: Arc<ExchangeManager>,
    pub calculators: Arc<CalculatorManager>,
    pub task_symbols_map: Arc<HashMap<ExchangeEnum, Vec<Symbol>>>,
}

impl App {
    pub async fn new(pool: sqlx::Pool<sqlx::Postgres>) -> anyhow::Result<Self> {
        let config_repo = ConfigRepository::new(pool.clone());

        // 获取配置
        let index_configs = config_repo.get_active_configs().await?;
        let tasks = config_repo.get_enabled_tasks().await?;
        println!("Loaded {} tasks from DB", tasks.len());

        // 查询 task 对应的 symbol
        let mut task_symbols_map: HashMap<ExchangeEnum, Vec<Symbol>> = HashMap::new();
        for task in &tasks {
            let ids: Vec<i32> = task.symbol_ids.split(',').filter_map(|s| s.parse().ok()).collect();
            let symbols = config_repo.get_symbols_by_ids(&ids, &task.exchange_name).await?;
            task_symbols_map.insert(ExchangeEnum::from_name(&task.exchange_name).unwrap(), symbols);
        }
        let task_symbols_map = Arc::new(task_symbols_map);

        // TradeRepository
        let trade_repo = Arc::new(TradeRepository::new(20));

        // ExchangeManager
        let manager = Arc::new(ExchangeManager::new());
        for task in &tasks {
            if !task.is_enabled {
                continue;
            }
            let exchange_enum = ExchangeEnum::from_name(&task.exchange_name).unwrap();
            if let Some(symbols) = task_symbols_map.get(&exchange_enum) {
                let symbol_map: HashMap<String, String> = symbols
                    .iter()
                    .map(|s| (s.third_symbol_name.clone(), s.symbol_name.clone()))
                    .collect();
                let client = ExchangeFactory::create(
                    exchange_enum.clone(),
                    trade_repo.clone(),
                    symbol_map,
                );
                let symbol_names: HashSet<String> = symbols.iter().map(|s| s.third_symbol_name.clone()).collect();
                manager.add_exchange(exchange_enum, client, symbol_names).await;
            }
        }

        manager.clone().spawn_heartbeat(30_000, "ping".to_string());
        manager.clone().spawn_reconnect(10_000);

        // 初始化计算器
        let mut calculators_map: HashMap<String, IndexCalculator> = HashMap::new();
        for config in &index_configs {
            calculators_map.insert(
                config.name.clone(),
                IndexCalculator::new(config.name.clone(), Decimal::from_f64_retain(0.003).unwrap()),
            );
        }
        let calculators = Arc::new(CalculatorManager::new(calculators_map));

        Ok(Self {
            manager,
            calculators,
            task_symbols_map,
        })
    }

    pub async fn run(self, index_configs: Vec<crate::core::model::IndexConfig>, config_repo: Arc<ConfigRepository>, kline_tx: UnboundedSender<IndexKlineData>) {
        let manager = self.manager.clone();
        let calculators = self.calculators.clone();
        let task_symbols_map = self.task_symbols_map.clone();

        // 启动三个异步任务
        tokio::spawn(price_updater::run_price_updater(
            manager.clone(),
            calculators.clone(),
            task_symbols_map.clone(),
        ));
        let config_repo_arc = config_repo.clone();
        tokio::spawn(index_calculator_task::run_index_calculator(
            calculators.clone(),
            index_configs.clone(),
            config_repo_arc,
            kline_tx,
            vec![
                KlineInterval::OneMinute,
                KlineInterval::FiveMinutes,
                KlineInterval::FifteenMinutes,
            ]
        ));

        tokio::spawn(market_printer::run_market_printer(
            manager.clone(),
            task_symbols_map.clone(),
        ));

        // 阻塞主线程，保持 tokio runtime 运行
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    }
}
