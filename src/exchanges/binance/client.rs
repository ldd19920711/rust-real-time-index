use std::{collections::{HashSet, HashMap}, sync::Arc};
use async_trait::async_trait;
use dashmap::DashMap;
use tokio::sync::{Mutex, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};
use futures_util::{SinkExt, StreamExt};
use futures_util::stream::SplitSink;
use tokio::net::TcpStream;
use tracing::{info, error};
use tungstenite::Utf8Bytes;

use crate::core::model::{Exchange, TickerData};
use crate::core::trade::trade_repository::{Trade, TradeRepository};
use crate::core::ws::websocket_listener::WebSocketStatusListener;

#[derive(Clone)]
pub struct BinanceWebSocketClient {
    pub exchange: Arc<Exchange>,
    pub ws_url: String,
    pub sub_symbol_set: Arc<RwLock<HashSet<String>>>,
    pub store: Arc<DashMap<String, TickerData>>,
    connected: Arc<RwLock<bool>>,
    trade_repo: Arc<TradeRepository>,
    ticker_suffix: String,
    pub symbol_map: Arc<RwLock<HashMap<String, String>>>,
    pub ping_msg : String,
    pub ping_interval : u64,
}

impl BinanceWebSocketClient {
    pub fn new(exchange: Exchange, trade_repo: Arc<TradeRepository>, symbol_map: HashMap<String, String>, ping_msg: String, ping_interval: u64) -> Self {
        Self {
            exchange: Arc::new(exchange),
            ws_url: "wss://stream.binance.com:443/ws/btcusdt@miniTicker".to_string(),
            sub_symbol_set: Arc::new(RwLock::new(HashSet::new())),
            store: Arc::new(DashMap::new()),
            connected: Arc::new(RwLock::new(false)),
            trade_repo,
            ticker_suffix: "@miniTicker".to_string(),
            symbol_map: Arc::new(RwLock::new(symbol_map)),
            ping_msg,
            ping_interval,
        }
    }
}

#[async_trait]
impl WebSocketStatusListener for BinanceWebSocketClient {
    fn exchange_name(&self) -> &str {
        &self.exchange.name
    }
    
    fn ping_msg(&self) -> &str {
        &self.ping_msg
    }
    
    fn ping_interval(&self) -> u64 {
        self.ping_interval
    }

    fn ws_url(&self) -> &str {
        &self.ws_url
    }

    fn build_sub_msg(&self, symbols: &HashSet<String>) -> Option<String> {
        if symbols.is_empty() {
            return None;
        }
        let args: Vec<_> = symbols
            .iter()
            .map(|s| format!("{}{}", s.to_lowercase(), self.ticker_suffix))
            .collect();
        Some(serde_json::json!({
            "method": "SUBSCRIBE",
            "params": args,
            "id": chrono::Utc::now().timestamp_millis()
        }).to_string())
    }

    async fn handle_message(&self, text: &str, write: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>) {
        if !text.contains("MiniTicker") {
            if text.contains("ping") {
                let pong_msg = text.replace("ping", "pong");
                let _ = write
                    .lock()
                    .await
                    .send(Message::Text(Utf8Bytes::from(pong_msg.clone())))
                    .await;
                info!("{} ping/pong: {}", self.exchange.name, pong_msg);
            }
            return;
        }
        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(text) {
            let symbol = json_val["s"].as_str().unwrap_or_default();
            let price = json_val["c"].as_str().unwrap_or_default();
            let ts = json_val["E"].as_i64().unwrap_or(0);

            let ticker = TickerData {
                last_pr: price.to_string(),
                inst_id: symbol.to_string(),
                ts: ts.to_string(),
            };

            // ✅ 先绑定 Arc，避免临时值被释放
            let symbol_map_lock = self.symbol_map();
            let symbol_map = symbol_map_lock.read().await;
            let symbol_name = symbol_map.get(symbol).unwrap_or(&String::from(symbol)).to_string();

            self.store().insert(symbol_name.clone(), ticker.clone());

            if let Ok(p) = price.parse::<f64>() {
                let trade = Trade {
                    exchange: self.exchange_name().to_string(),
                    symbol: symbol_name,
                    price: p,
                    timestamp: ts,
                };
                self.trade_repo().save_trade(trade);
            }
        }
    }

    fn connected(&self) -> Arc<RwLock<bool>> {
        Arc::clone(&self.connected)
    }

    fn sub_symbol_set(&self) -> Arc<RwLock<HashSet<String>>> {
        Arc::clone(&self.sub_symbol_set)
    }

    fn store(&self) -> Arc<DashMap<String, TickerData>> {
        Arc::clone(&self.store)
    }

    fn trade_repo(&self) -> Arc<TradeRepository> {
        Arc::clone(&self.trade_repo)
    }

    fn symbol_map(&self) -> Arc<RwLock<HashMap<String, String>>> {
        Arc::clone(&self.symbol_map)
    }
}
