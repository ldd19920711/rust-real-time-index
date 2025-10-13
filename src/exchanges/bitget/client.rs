use std::{collections::HashSet, sync::Arc};
use tokio::sync::RwLock;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use dashmap::DashMap;
use tracing::{info, error};
use serde_json::json;
use tungstenite::Utf8Bytes;
use async_trait::async_trait;
use crate::core::websocket_listener::{WebSocketStatusListener};
use crate::exchanges::bitget::model::{Exchange, TickerData};

#[derive(Clone)]
pub struct BitgetWebSocketClient {
    pub exchange: Arc<Exchange>,
    pub ws_url: String,
    pub sub_symbol_set: Arc<RwLock<HashSet<String>>>,
    pub store: Arc<DashMap<String, TickerData>>,
}

impl BitgetWebSocketClient {
    pub fn new(exchange: Exchange) -> Self {
        Self {
            exchange: Arc::new(exchange),
            ws_url: "wss://ws.bitget.com/v2/ws/public".to_string(),
            sub_symbol_set: Arc::new(RwLock::new(HashSet::new())),
            store: Arc::new(DashMap::new()),
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

        // 构建订阅消息
        let subs = self.sub_symbol_set.read().await;
        if let Some(msg) = self.build_sub_msg(&subs) {
            write.send(Message::Text(Utf8Bytes::from(msg))).await?;
        }

        let store = self.store.clone();
        let exchange = self.exchange.clone();

        // 异步接收消息
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if !text.contains("ticker") { continue; }
                        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&text) {
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
                                        store.insert(inst_id.to_string(), ticker);
                                    }
                                }
                            }
                        }
                    }
                    Ok(Message::Ping(_)) => {}
                    Err(e) => {
                        error!("{} websocket error: {:?}", exchange.name, e);
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
impl WebSocketStatusListener for BitgetWebSocketClient {
    fn exchange_name(&self) -> &str { &self.exchange.name }

    async fn connect(&self, symbols: Option<HashSet<String>>) {
        let _ = self.connect_internal(symbols).await;
    }

    fn build_sub_msg(&self, symbols: &HashSet<String>) -> Option<String> {
        let args: Vec<_> = symbols.iter()
            .map(|s| json!({"instType":"SPOT","channel":"ticker","instId": s}))
            .collect();
        Some(json!({"op":"subscribe","args":args}).to_string())
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
}
