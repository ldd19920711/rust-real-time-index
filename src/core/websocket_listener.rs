use std::{collections::HashSet, sync::Arc};
use async_trait::async_trait;
use dashmap::DashMap;
use tokio::{sync::RwLock, time::{sleep, Duration}};
use tracing::{info, error};
use crate::core::model::TickerData;

/// WebSocket 抽象 trait，等价 Java WebSocketStatusListener
#[async_trait]
pub trait WebSocketStatusListener: Send + Sync + 'static {
    fn exchange_name(&self) -> &str;

    /// 连接或重连
    async fn connect(&self, symbols: Option<HashSet<String>>);

    /// 构建订阅消息 JSON
    fn build_sub_msg(&self, symbols: &HashSet<String>) -> Option<String>;

    /// 执行订阅
    async fn subscription(&self);

    /// 发送心跳
    async fn send_ping(&self, ping_msg: &str);

    /// 是否已连接
    fn is_connected(&self) -> bool;

    fn get_ticker(&self, symbol: &str) -> Option<TickerData>;
}

/// WebSocketManager 管理多个 listener
pub struct WebSocketManager<T: WebSocketStatusListener + ?Sized + 'static> {
    pub clients: Arc<DashMap<String, Arc<T>>>,
}

impl<T: WebSocketStatusListener + ?Sized + 'static> WebSocketManager<T> {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(DashMap::new()),
        }
    }

    /// 自动心跳任务
    pub fn spawn_heartbeat(self: Arc<Self>, interval: u64, ping_msg: String) {
        let clients = self.clients.clone();
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(interval)).await;
                for entry in clients.iter() {
                    let listener = entry.value().clone(); // Arc<T>
                    if listener.is_connected() {
                        listener.send_ping(&ping_msg).await;
                    }
                }
            }
        });
    }

    /// 自动重连任务
    pub fn spawn_reconnect(self: Arc<Self>, interval: u64) {
        let clients = self.clients.clone();
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(interval)).await;
                for entry in clients.iter() {
                    let listener = entry.value().clone();
                    if !listener.is_connected() {
                        info!("{} disconnected, reconnecting...", listener.exchange_name());
                        listener.connect(None).await;
                    }
                }
            }
        });
    }
}
