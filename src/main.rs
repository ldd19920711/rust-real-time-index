mod core;
mod exchanges;

use std::collections::HashSet;
use std::sync::Arc;
use chrono::Utc;
use rust_decimal::Decimal;
use tokio::time::{sleep, Duration};

use core::trade_repository::TradeRepository;
use core::exchange_factory::ExchangeFactory;
use core::exchange_manager::ExchangeManager;
use exchanges::ExchangeEnum;
use crate::core::index_calculator::{IndexCalculator, IndexConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 创建 TradeRepository
    let trade_repo = Arc::new(TradeRepository::new(20));

    // 创建 ExchangeManager
    let manager = Arc::new(ExchangeManager::new());

    // 需要启动的交易所
    let exchanges_to_start = Arc::new(vec![ExchangeEnum::Bitget, ExchangeEnum::Binance]);

    // 初始化 symbol
    let symbols: HashSet<String> = ["BTCUSDT", "ETHUSDT"].iter().map(|s| s.to_string()).collect();

    // 创建客户端并加入 Manager
    for exch in exchanges_to_start.clone().iter() {
        let client = ExchangeFactory::create(exch.clone(), trade_repo.clone());
        manager.add_exchange(exch.clone(), client, symbols.clone()).await;
    }

    // 启动心跳和重连任务
    manager.clone().spawn_heartbeat(30_000, "ping".to_string());
    manager.clone().spawn_reconnect(10_000);


    // 初始化指数配置
    let index_configs = vec![
        IndexConfig {
            name: "BTCUSDT".to_string(),
            formula: "(Binance.BTCUSDT + Bitget.BTCUSDT)/2".to_string(),
        },
        IndexConfig {
            name: "ETHUSDT".to_string(),
            formula: "(Binance.ETHUSDT + Bitget.ETHUSDT)/2".to_string(),
        },
    ];

    // 使用 Arc<RwLock<>> 包裹 IndexCalculator
    let calculator = Arc::new(tokio::sync::RwLock::new(IndexCalculator::new(
        Decimal::from_f64_retain(0.003f64).unwrap(),
    )));

    // 价格更新任务：轮询 ExchangeManager 的客户端，把最新价格写入 IndexCalculator
    let calculator_clone = calculator.clone();
    let manager_clone = manager.clone();
    let exchanges_to_start_clone = exchanges_to_start.clone();
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(1)).await;
            for exch in exchanges_to_start_clone.clone().iter() {
                if let Some(client) = manager_clone.get_client(exch) {
                    for symbol in &["BTCUSDT", "ETHUSDT"] {
                        if let Some(t) = client.get_ticker(symbol) {
                            let price = t.last_pr.parse::<Decimal>().unwrap_or(Decimal::ZERO);
                            let mut calc = calculator_clone.write().await;
                            calc.update_price(&format!("{}.{}", exch.name(), symbol), price);
                        }
                    }
                }
            }
        }
    });

    // 指数计算任务
    let calculator_clone = calculator.clone();
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(1)).await;

            for config in &index_configs {
                let calc = calculator_clone.read().await;
                if let Some(idx) = calc.calculate_index(&config.name, &config.formula, Utc::now().timestamp() as u64) {
                    // 先只打印BTCUSDT
                    if config.name == "BTCUSDT" {
                        println!("Index {} = {} , formula: {}, computed_formula: {}", config.name, idx.last, idx.formula, idx.computed_formula);
                    }
                }
            }
        }
    });

    // 循环打印行情
    let exchanges_to_start_clone = exchanges_to_start.clone();
    loop {
        sleep(Duration::from_secs(5)).await;

        for exch in exchanges_to_start_clone.clone().iter() {
            if let Some(client) = manager.get_client(exch) {
                if let Some(t) = client.get_ticker("BTCUSDT") {
                    println!("{} BTCUSDT 最新价: {}", exch.name(), t.last_pr);
                }
            }
        }
    }
}
