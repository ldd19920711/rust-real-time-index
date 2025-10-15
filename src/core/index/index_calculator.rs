use rust_decimal::prelude::*;
use std::collections::HashMap;
use chrono::{Duration, Utc};

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
    pub index_name: String,
    /// 异常比例，类似 Java 的 EXCEPTION_PERCENT_MARGIN
    pub exception_percent_margin: Decimal,
    /// 当前各交易所最新价格
    pub price_map: HashMap<String, Decimal>, // key: "Binance.BTCUSDT"

    pub index_list : Vec<Index>,
}

impl IndexCalculator {
    pub fn new(index_name: String, exception_percent_margin: Decimal) -> Self {
        Self {
            index_name,
            exception_percent_margin,
            price_map: HashMap::new(),
            index_list: Vec::new(),
        }
    }

    /// 更新价格（可由 WebSocket 客户端定期写入）
    pub fn update_price(&mut self, key: &str, price: Decimal) {
        self.price_map.insert(key.to_string(), price);
    }

    pub fn calculate_index(&mut self, name: &str, formula: &str, time: u64) -> Option<Index> {
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
        let mut valid_count = 0u32; // 实际存在价格的个数
        let mut computed_parts = Vec::new();

        for t in tickers {
            if let Some(price) = self.price_map.get(t) {
                sum += *price;
                valid_count += 1;
                computed_parts.push(price.to_string());
            } else {
                println!("Price not found for {}", t);
            }
        }

        if valid_count == 0 {
            return None; // 所有价格都缺失，无法计算
        }

        // 用实际存在价格的数量平均
        let mut last = sum / Decimal::from(valid_count);

        // // 如果公式里有除法，继续除
        // if parts.len() > 1 {
        //     if let Ok(denom) = parts[1].parse::<Decimal>() {
        //         last /= denom;
        //     }
        // }

        // 构建 computed_formula：使用有效数据的个数来除，而不是公式里原来的除数
        let computed_formula = if valid_count > 1 {
            format!("({}) / {}", computed_parts.join(" + "), valid_count)
        } else {
            computed_parts.join(" + ")
        };

        last = self.check_exception(last);

        let index = Index {
            id: time,
            symbol: name.to_string(),
            last,
            formula: formula.to_string(),
            computed_formula,
        };

        self.index_list.push(index.clone());

        Some(index)
    }


    pub fn calculate_edp(&mut self) -> Option<Decimal> {
        if self.index_list.is_empty() {
            return None;
        }

        // 当前时间 - 10分钟
        let cutoff = Utc::now() - Duration::minutes(10);
        let cutoff_ms = cutoff.timestamp_millis();

        // 删除旧数据 + 累加最新价格
        let mut sum = Decimal::ZERO;

        // 过滤保留有效数据
        self.index_list.retain(|idx| idx.id >= cutoff_ms as u64);

        for idx in &self.index_list {
            sum += idx.last;
        }

        // 如果全被清理掉了
        if self.index_list.is_empty() {
            return None;
        }

        // 求平均
        let avg = sum / Decimal::from(self.index_list.len() as u64);

        println!(
            "Calculate EDP [{}] symbol [{}] size [{}]",
            avg, self.index_name, self.index_list.len()
        );

        Some(avg.round_dp_with_strategy(8, rust_decimal::RoundingStrategy::MidpointAwayFromZero))
    }


    fn check_exception(&self, price: Decimal) -> Decimal {
        // 简单示例：可根据 median 或其他规则实现
        price // 这里暂时不处理异常
    }
}
