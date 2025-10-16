use crate::core::model::{Exchange, TickerData};
use crate::core::trade::trade_repository::{Trade, TradeRepository};
use async_trait::async_trait;
use dashmap::DashMap;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, RwLock};
use tokio::time::interval;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async, tungstenite::protocol::Message,
};
use tracing::{error, info};
use tungstenite::Utf8Bytes;

#[async_trait]
pub trait WebSocketStatusListener: Send + Sync + 'static {
    fn exchange_name(&self) -> &str;
    fn ping_msg(&self) -> &str;

    fn ping_interval(&self) -> u64;
    fn ws_url(&self) -> &str;

    /// ---- 抽象部分 ----
    fn build_sub_msg(&self, symbols: &HashSet<String>) -> Option<String>;
    async fn handle_message(
        &self,
        text: &str,
        write: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
    );

    /// ---- 通用字段访问 ----
    fn connected(&self) -> Arc<RwLock<bool>>;
    fn sub_symbol_set(&self) -> Arc<RwLock<HashSet<String>>>;
    fn store(&self) -> Arc<DashMap<String, TickerData>>;
    fn trade_repo(&self) -> Arc<TradeRepository>;
    fn symbol_map(&self) -> Arc<RwLock<HashMap<String, String>>>;

    /// ---- 通用逻辑（提供默认实现）----
    async fn connect_internal_arc(
        self: Arc<Self>,
        symbols: Option<HashSet<String>>,
    ) -> anyhow::Result<()> {
        use futures_util::{SinkExt, StreamExt};
        use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
        use tracing::{error, info};
        use tungstenite::Utf8Bytes;

        // ✅ 持有 Arc，延长生命周期
        let sub_set = self.sub_symbol_set();
        if let Some(s) = symbols {
            let mut set = sub_set.write().await;
            set.extend(s);
        }

        info!("{} connecting to {}", self.exchange_name(), self.ws_url());
        let (ws_stream, _) = connect_async(self.ws_url()).await?;
        let (mut write, mut read) = ws_stream.split();

        // ✅ 持有 Arc，延长生命周期
        let connected = self.connected();
        {
            let mut conn = connected.write().await;
            *conn = true;
        }

        // write 包装成 Arc<Mutex<_>>
        let write = Arc::new(Mutex::new(write));
        let this = Arc::clone(&self);

        // 心跳任务
        {
            let write_clone = Arc::clone(&write);
            tokio::spawn(async move {
                let ping_msg = this.ping_msg();
                let ping_interval = this.ping_interval();
                if ping_interval > 0 && !ping_msg.is_empty() {
                    let mut ticker =
                        tokio::time::interval(tokio::time::Duration::from_millis(ping_interval));
                    loop {
                        ticker.tick().await;
                        let mut write_guard = write_clone.lock().await;
                        if let Err(e) = write_guard
                            .send(Message::Text(Utf8Bytes::from(ping_msg)))
                            .await
                        {
                            error!("{} send ping error: {:?}", this.exchange_name(), e);
                            break;
                        } else {
                            // info!("{} sent ping: {}", this.exchange_name(), ping_msg);
                        }
                    }
                }
            });
        }

        // 批量订阅
        let sub_set = self.sub_symbol_set(); // 再次绑定 Arc
        let subs = sub_set.read().await;
        let symbols_vec: Vec<_> = subs.iter().cloned().collect();
        let chunk_size = 20;
        for chunk in symbols_vec.chunks(chunk_size) {
            let chunk_set: HashSet<String> = chunk.iter().cloned().collect();
            if let Some(msg) = self.build_sub_msg(&chunk_set) {
                let mut write_guard = write.lock().await;
                write_guard
                    .send(Message::Text(Utf8Bytes::from(msg)))
                    .await?;
            }
        }

        // ✅ 持有 Arc，用于 spawn
        let this = Arc::clone(&self);
        let connected = Arc::clone(&connected);

        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        let write_clone = Arc::clone(&write);
                        this.handle_message(&text, write_clone).await;
                    }
                    Err(e) => {
                        error!("{} websocket error: {:?}", this.exchange_name(), e);
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

    /// 默认 connect 调用内部实现
    async fn connect(self: Arc<Self>, symbols: Option<HashSet<String>>) {
        if let Err(e) = self.clone().connect_internal_arc(symbols).await {
            error!("{} connect error: {:?}", self.exchange_name(), e);
            *self.connected().write().await = false;
        }
    }

    /// 判断是否连接
    fn is_connected(&self) -> bool {
        futures::executor::block_on(async { *self.connected().read().await })
    }

    /// 获取最新 ticker
    fn get_ticker(&self, symbol: &str) -> Option<TickerData> {
        self.store().get(symbol).map(|v| v.clone())
    }
}
