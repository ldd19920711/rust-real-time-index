use std::{collections::{HashSet, HashMap}, sync::Arc};
use async_trait::async_trait;
use dashmap::DashMap;
use tokio::sync::{Mutex, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};
use futures_util::{SinkExt, StreamExt};
use futures_util::stream::SplitSink;
use tracing::{info, error};
use tungstenite::Utf8Bytes;
use serde_json::json;
use tokio::net::TcpStream;
use crate::core::model::{Exchange, TickerData};
use crate::core::trade::trade_repository::{Trade, TradeRepository};
use crate::core::ws::websocket_listener::WebSocketStatusListener;

#[derive(Clone)]
pub struct BitgetWebSocketClient {
    pub exchange: Arc<Exchange>,
    pub ws_url: String,
    pub sub_symbol_set: Arc<RwLock<HashSet<String>>>,
    pub store: Arc<DashMap<String, TickerData>>,
    connected: Arc<RwLock<bool>>,
    trade_repo: Arc<TradeRepository>,
    pub symbol_map: Arc<RwLock<HashMap<String, String>>>,
    pub ping_msg : String,
    pub ping_interval : u64,
}

impl BitgetWebSocketClient {
    pub fn new(exchange: Exchange, trade_repo: Arc<TradeRepository>, symbol_map: HashMap<String, String>, ping_msg: String, ping_interval: u64) -> Self {
        Self {
            exchange: Arc::new(exchange),
            ws_url: "wss://ws.bitget.com/v2/ws/public".to_string(),
            sub_symbol_set: Arc::new(RwLock::new(HashSet::new())),
            store: Arc::new(DashMap::new()),
            connected: Arc::new(RwLock::new(false)),
            trade_repo,
            symbol_map: Arc::new(RwLock::new(symbol_map)),
            ping_msg,
            ping_interval,
        }
    }
}

#[async_trait]
impl WebSocketStatusListener for BitgetWebSocketClient {
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
        if symbols.is_empty() { return None; }
        let args: Vec<_> = symbols.iter()
            .map(|s| json!({"instType":"SPOT","channel":"ticker","instId": s}))
            .collect();
        Some(json!({"op":"subscribe","args": args}).to_string())
    }

    async fn handle_message(&self, text: &str, write: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>) {
        if !text.contains("ticker") { return; }

        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(text) {
            if let Some(data_array) = json_val["data"].as_array() {
                for d in data_array {
                    if let (Some(last), Some(inst_id), Some(ts)) =
                        (d["lastPr"].as_str(), d["instId"].as_str(), d["ts"].as_str())
                    {
                        let ticker = TickerData {
                            last_pr: last.to_string(),
                            inst_id: inst_id.to_string(),
                            ts: ts.to_string(),
                        };

                        let symbol_map_lock = self.symbol_map();
                        let symbol_map = symbol_map_lock.read().await;
                        let symbol_name = symbol_map.get(inst_id).unwrap_or(&inst_id.to_string()).to_string();

                        self.store().insert(symbol_name.clone(), ticker.clone());

                        if let Ok(price) = last.parse::<f64>() {
                            let trade = Trade {
                                exchange: self.exchange_name().to_string(),
                                symbol: symbol_name,
                                price,
                                timestamp: ts.parse().unwrap_or(0),
                            };
                            self.trade_repo().save_trade(trade);
                        }
                    }
                }
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
