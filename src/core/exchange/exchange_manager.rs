use crate::core::ws::websocket_listener::WebSocketStatusListener;
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
        // 先 clone 一份 Arc 用于 connect
        let client_clone = client.clone();
        let _ = client_clone.connect(Some(symbols.clone())).await;

        // 插入 DashMap 使用原始 Arc
        self.clients.insert(exchange, client);
    }


    /// 获取客户端
    pub fn get_client(&self, exchange: &ExchangeEnum) -> Option<Arc<dyn WebSocketStatusListener>> {
        self.clients.get(exchange).map(|v| v.clone())
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
