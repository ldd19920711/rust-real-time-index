use std::collections::HashSet;
use std::sync::Arc;
use crate::core::model::Exchange;
use crate::core::trade_repository::TradeRepository;
use crate::exchanges::{ExchangeEnum, binance, bitget};
use crate::core::websocket_listener::WebSocketStatusListener;

pub struct ExchangeFactory;

impl ExchangeFactory {
    pub fn create(
        exch: ExchangeEnum,
        trade_repo: Arc<TradeRepository>,
    ) -> Arc<dyn WebSocketStatusListener> {
        match exch {
            ExchangeEnum::Bitget => Arc::new(bitget::client::BitgetWebSocketClient::new(
                Exchange { name: exch.name().to_string() },
                trade_repo,
            )),
            ExchangeEnum::Binance => Arc::new(binance::client::BinanceWebSocketClient::new(
                Exchange { name: exch.name().to_string() },
                trade_repo,
            )),
        }
    }
}
