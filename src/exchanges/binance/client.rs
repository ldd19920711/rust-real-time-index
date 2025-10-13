use crate::core::model::{Exchange, TickerData};
use crate::core::trade_repository::{Trade, TradeRepository};
use crate::core::websocket_listener::WebSocketStatusListener;
use crate::exchanges::bitget::client::BitgetWebSocketClient;
use async_trait::async_trait;
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::io::Bytes;
use std::{collections::HashSet, sync::Arc};
use tokio::sync::RwLock;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info};
use tungstenite::Utf8Bytes;

#[derive(Clone)]
pub struct BinanceWebSocketClient {
    pub exchange: Arc<Exchange>,
    pub ws_url: String,
    pub sub_symbol_set: Arc<RwLock<HashSet<String>>>,
    pub store: Arc<DashMap<String, TickerData>>,
    connected: Arc<RwLock<bool>>,
    trade_repo: Arc<TradeRepository>,
    ticker_suffix: String,
}

impl BinanceWebSocketClient {
    pub fn new(exchange: Exchange, trade_repo: Arc<TradeRepository>) -> Self {
        Self {
            exchange: Arc::new(exchange),
            ws_url: "wss://stream.binance.com:443/ws/btcusdt@miniTicker".to_string(),
            sub_symbol_set: Arc::new(RwLock::new(HashSet::new())),
            store: Arc::new(DashMap::new()),
            connected: Arc::new(RwLock::new(false)),
            trade_repo,
            ticker_suffix: "@miniTicker".to_string(),
        }
    }

    async fn connect_internal(&self, symbols: Option<HashSet<String>>) -> anyhow::Result<()> {
        if let Some(s) = symbols {
            let mut set = self.sub_symbol_set.write().await;
            set.extend(s);
        }

        info!("{} connecting to {}", self.exchange.name, self.ws_url);
        let (ws_stream, _) = connect_async(&self.ws_url).await?;
        let (mut write, mut read) = ws_stream.split();

        {
            let mut conn = self.connected.write().await;
            *conn = true;
        }

        // 构建订阅消息
        let subs = self.sub_symbol_set.read().await;
        if let Some(msg) = self.build_sub_msg(&subs) {
            write.send(Message::Text(Utf8Bytes::from(msg))).await?;
        }

        let store = self.store.clone();
        let exchange = self.exchange.clone();
        let connected = self.connected.clone();
        let trade_repo = self.trade_repo.clone();
        // let ticker_suffix = self.ticker_suffix.clone();

        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if text.contains("MiniTicker") {
                            // 解析行情
                            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&text) {
                                let symbol = json_val["s"].as_str().unwrap_or_default();
                                let price = json_val["c"].as_str().unwrap_or_default();
                                let ts = json_val["E"].as_i64().unwrap_or(0);

                                let ticker = TickerData {
                                    last_pr: price.to_string(),
                                    inst_id: symbol.to_string(),
                                    ts: ts.to_string(),
                                };
                                store.insert(symbol.to_string(), ticker.clone());

                                if let Ok(p) = price.parse::<f64>() {
                                    let trade = Trade {
                                        exchange: exchange.name.clone(),
                                        symbol: symbol.to_string(),
                                        price: p,
                                        timestamp: ts,
                                    };
                                    trade_repo.save_trade(trade);
                                }
                            }
                        } else if text.contains("ping") {
                            let pong_msg = text.replace("ping", "pong");
                            let _ = write
                                .send(Message::Text(Utf8Bytes::from(pong_msg.clone())))
                                .await;
                            info!("{} ping/pong: {}", exchange.name, pong_msg);
                        }
                    }
                    Ok(Message::Ping(_)) => {
                        // let _ = write.send(Message::Pong(Bytes::)).await;
                    }
                    Err(e) => {
                        error!("{} websocket error: {:?}", exchange.name, e);
                        let mut conn = connected.write().await;
                        *conn = false;
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }
}

#[async_trait]
impl WebSocketStatusListener for BinanceWebSocketClient {
    fn exchange_name(&self) -> &str {
        &self.exchange.name
    }

    async fn connect(&self, symbols: Option<HashSet<String>>) {
        if let Err(e) = self.connect_internal(symbols).await {
            error!("{} connect error: {:?}", self.exchange.name, e);
            let mut conn = self.connected.write().await;
            *conn = false;
        }
    }

    fn build_sub_msg(&self, symbols: &HashSet<String>) -> Option<String> {
        if symbols.is_empty() {
            return None;
        }
        let args: Vec<_> = symbols
            .iter()
            .map(|s| format!("{}{}", s.to_lowercase(), self.ticker_suffix))
            .collect();
        Some(
            json!({
                "method": "SUBSCRIBE",
                "params": args,
                "id": chrono::Utc::now().timestamp_millis()
            })
            .to_string(),
        )
    }

    async fn subscription(&self) {
        let subs = self.sub_symbol_set.read().await;
        if let Some(msg) = self.build_sub_msg(&subs) {
            info!("{} subscription msg: {}", self.exchange.name, msg);
        }
    }

    async fn send_ping(&self, ping_msg: &str) {
        info!("{} send ping: {}", self.exchange.name, ping_msg);
    }

    fn is_connected(&self) -> bool {
        futures::executor::block_on(async { *self.connected.read().await })
    }

    fn get_ticker(&self, symbol: &str) -> Option<TickerData> {
        self.store.get(symbol).map(|v| v.clone())
    }
}