mod core;
mod exchanges;

use std::collections::HashSet;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::info;
use exchanges::bitget::client::BitgetWebSocketClient;
use exchanges::bitget::model::Exchange;
use core::websocket_listener::WebSocketManager;
use crate::core::trade_repository::TradeRepository;
use crate::core::websocket_listener::WebSocketStatusListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let trade_repo = Arc::new(TradeRepository::new(20));


    let client = Arc::new(BitgetWebSocketClient::new(Exchange { name: "Bitget".to_string() }, trade_repo.clone()));

    let mut symbols = HashSet::new();
    symbols.insert("BTCUSDT".to_string());
    symbols.insert("ETHUSDT".to_string());

    client.connect(Some(symbols.clone())).await;

    let manager = Arc::new(WebSocketManager::<BitgetWebSocketClient>::new());
    manager.clients.insert(client.exchange_name().to_string(), client.clone());

    // 启动心跳
    manager.clone().spawn_heartbeat(30_000, "ping".to_string());

    // 启动自动重连
    manager.clone().spawn_reconnect(10_000);

    loop {
        sleep(Duration::from_secs(5)).await;
        if let Some(t) = client.store.get("BTCUSDT") {
            info!("BTCUSDT 最新价: {}", t.last_pr);
        }
    }
}
