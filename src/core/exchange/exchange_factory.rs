use crate::core::model::Exchange;
use crate::core::trade::trade_repository::TradeRepository;
use crate::core::ws::websocket_listener::WebSocketStatusListener;
use crate::exchanges::{binance, bitget, okex, ExchangeEnum};
use std::collections::HashMap;
use std::sync::Arc;

pub struct ExchangeFactory;

impl ExchangeFactory {
    pub fn create(
        exch: ExchangeEnum,
        trade_repo: Arc<TradeRepository>,
        symbol_map: HashMap<String, String>,
    ) -> Arc<dyn WebSocketStatusListener> {
        match exch {
            ExchangeEnum::Bitget => Arc::new(bitget::client::BitgetWebSocketClient::new(
                Exchange {
                    name: exch.name().to_string(),
                },
                trade_repo,
                symbol_map,
                "ping".to_string(),
                15_000,
            )),
            ExchangeEnum::Binance => Arc::new(binance::client::BinanceWebSocketClient::new(
                Exchange {
                    name: exch.name().to_string(),
                },
                trade_repo,
                symbol_map,
                "".to_string(),
                15_000,
            )),
            ExchangeEnum::Okex => Arc::new(okex::client::OkexWebSocketClient::new(
                Exchange {
                    name: exch.name().to_string(),
                },
                trade_repo,
                symbol_map,
                "ping".to_string(),
                15_000,
            )),
        }
    }
}
