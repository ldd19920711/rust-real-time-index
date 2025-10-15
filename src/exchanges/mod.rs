pub mod bitget;
pub mod binance;
pub mod okex;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExchangeEnum {
    Bitget,
    Binance,
    Okex,
}

impl ExchangeEnum {
    pub fn name(&self) -> &'static str {
        match self {
            ExchangeEnum::Bitget => "Bitget",
            ExchangeEnum::Binance => "Binance",
            ExchangeEnum::Okex => "Okex",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Bitget" => Some(ExchangeEnum::Bitget),
            "Binance" => Some(ExchangeEnum::Binance),
            "Okex" => Some(ExchangeEnum::Okex),
            _ => None,
        }
    }
}
