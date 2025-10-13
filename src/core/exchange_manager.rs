use crate::core::websocket_listener::WebSocketStatusListener;
use crate::exchanges::ExchangeEnum;
use dashmap::DashMap;
use std::{collections::HashSet, sync::Arc};
use tokio::time::{sleep, Duration};
use tracing::info;

/// ExchangeManager 管理多个 WebSocketStatusListener
pub struct ExchangeManager {
    clients: Arc<DashMap<ExchangeEnum, Arc<dyn WebSocketStatusListener>>>,
}

impl ExchangeManager {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(DashMap::new()),
        }
    }

    /// 添加交易所客户端并订阅初始 symbol
    pub async fn add_exchange(
        &self,
        exchange: ExchangeEnum,
        client: Arc<dyn WebSocketStatusListener>,
        symbols: HashSet<String>,
    ) {
        let _ = client.connect(Some(symbols.clone())).await;
        self.clients.insert(exchange, client);
    }

    /// 获取客户端
    pub fn get_client(&self, exchange: &ExchangeEnum) -> Option<Arc<dyn WebSocketStatusListener>> {
        self.clients.get(exchange).map(|v| v.clone())
    }

    /// 启动心跳任务
    pub fn spawn_heartbeat(self: Arc<Self>, interval_ms: u64, ping_msg: String) {
        let clients = self.clients.clone();
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(interval_ms)).await;

                // 先收集 Arc 到 Vec，保证 'static 生命周期
                let listeners: Vec<Arc<dyn WebSocketStatusListener>> =
                    clients.iter().map(|entry| entry.value().clone()).collect();

                for listener in listeners {
                    if listener.is_connected() {
                        let ping_msg = ping_msg.clone();
                        let listener = listener.clone();
                        // 每个 ping spawn 独立任务
                        tokio::spawn(async move {
                            let _ = listener.send_ping(&ping_msg).await;
                        });
                    }
                }
            }
        });
    }

    /// 启动自动重连任务
    pub fn spawn_reconnect(self: Arc<Self>, interval_ms: u64) {
        let clients = self.clients.clone();
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(interval_ms)).await;

                // 收集 Arc 到 Vec
                let listeners: Vec<Arc<dyn WebSocketStatusListener>> =
                    clients.iter().map(|entry| entry.value().clone()).collect();

                for listener in listeners {
                    if !listener.is_connected() {
                        let listener = listener.clone();
                        tokio::spawn(async move {
                            info!("{} disconnected, reconnecting...", listener.exchange_name());
                            let _ = listener.connect(None).await;
                        });
                    }
                }
            }
        });
    }
}
