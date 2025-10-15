use std::sync::Arc;
use dashmap::DashMap;
use tokio::time::{Instant, Duration};
use tracing::info;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct QuoteIndex {
    pub exchange: String,
    pub symbol: String,
}

#[derive(Clone)]
pub struct Trade {
    pub exchange: String,
    pub symbol: String,
    pub price: f64,
    pub timestamp: i64,
}

/// 内存缓存 + TTL
#[derive(Clone)]
pub struct TradeRepository {
    store: Arc<DashMap<QuoteIndex, (Trade, Instant)>>,
    ttl: Duration,
}

impl TradeRepository {
    pub fn new(ttl_minutes: u64) -> Self {
        let repo = Self {
            store: Arc::new(DashMap::new()),
            ttl: Duration::from_secs(ttl_minutes * 60),
        };
        repo.spawn_cleanup_task();
        repo
    }

    pub fn save_trade(&self, trade: Trade) {
        let idx = QuoteIndex {
            exchange: trade.exchange.clone(),
            symbol: trade.symbol.clone(),
        };
        self.store.insert(idx, (trade, Instant::now()));
    }

    pub fn get_trade(&self, exchange: &str, symbol: &str) -> Option<Trade> {
        let idx = QuoteIndex {
            exchange: exchange.to_string(),
            symbol: symbol.to_string(),
        };
        self.store.get(&idx).map(|v| v.value().0.clone())
    }

    fn spawn_cleanup_task(&self) {
        let store = self.store.clone();
        let ttl = self.ttl;
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;
                let now = Instant::now();
                let mut removed = 0;
                store.retain(|_, (_trade, instant)| {
                    let alive = now.duration_since(*instant) < ttl;
                    if !alive { removed += 1; }
                    alive
                });
                if removed > 0 {
                    info!("Cleaned up {} expired trades", removed);
                }
            }
        });
    }
}

