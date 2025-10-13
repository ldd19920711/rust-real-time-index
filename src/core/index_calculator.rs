use rust_decimal::prelude::*;
use std::collections::HashMap;

/// 指数配置
#[derive(Debug, Clone)]
pub struct IndexConfig {
    pub name: String,
    pub formula: String,
}

/// 计算结果
#[derive(Debug, Clone)]
pub struct Index {
    pub id: u64,
    pub symbol: String,
    pub last: Decimal,
    pub formula: String,
    pub computed_formula: String, // 增加字段，用于展示实际计算
}

/// 核心计算器
pub struct IndexCalculator {
    /// 异常比例，类似 Java 的 EXCEPTION_PERCENT_MARGIN
    pub exception_percent_margin: Decimal,
    /// 当前各交易所最新价格
    pub price_map: HashMap<String, Decimal>, // key: "Binance.BTCUSDT"
}

impl IndexCalculator {
    pub fn new(exception_percent_margin: Decimal) -> Self {
        Self {
            exception_percent_margin,
            price_map: HashMap::new(),
        }
    }

    /// 更新价格（可由 WebSocket 客户端定期写入）
    pub fn update_price(&mut self, key: &str, price: Decimal) {
        self.price_map.insert(key.to_string(), price);
    }

    pub fn calculate_index(&self, name: &str, formula: &str, time: u64) -> Option<Index> {
        if self.price_map.is_empty() {
            println!("Price map is empty");
            return None;
        }

        let formula_clean = formula.replace(" ", "").replace("(", "").replace(")", "");
        let parts: Vec<&str> = formula_clean.split('/').collect();
        if parts.is_empty() {
            return None;
        }

        // numerator
        let numerator = parts[0];
        let tickers: Vec<&str> = numerator.split('+').collect();
        let mut sum = Decimal::ZERO;
        let mut computed_parts = Vec::new();

        for t in tickers {
            if let Some(price) = self.price_map.get(t) {
                sum += *price;
                computed_parts.push(price.to_string());
            } else {
                println!("Price not found for {}", t);
                computed_parts.push("0".to_string());
            }
        }

        let mut last = sum;
        let computed_formula = if parts.len() > 1 {
            if let Ok(denom) = parts[1].parse::<Decimal>() {
                last /= denom;
                format!("({}) / {}", computed_parts.join(" + "), denom)
            } else {
                format!("({})", computed_parts.join(" + "))
            }
        } else {
            format!("({})", computed_parts.join(" + "))
        };

        last = self.check_exception(last);

        Some(Index {
            id: time,
            symbol: name.to_string(),
            last,
            formula: formula.to_string(),
            computed_formula,
        })
    }



    fn check_exception(&self, price: Decimal) -> Decimal {
        // 简单示例：可根据 median 或其他规则实现
        price // 这里暂时不处理异常
    }
}
