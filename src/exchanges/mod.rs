pub mod bitget;
pub mod binance;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExchangeEnum {
    Bitget,
    Binance,
}

impl ExchangeEnum {
    pub fn name(&self) -> &'static str {
        match self {
            ExchangeEnum::Bitget => "Bitget",
            ExchangeEnum::Binance => "Binance",
        }
    }
}
