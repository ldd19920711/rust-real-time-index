use crate::core::index::index_calculator::IndexCalculator;
use rust_decimal::Decimal;
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

pub type SharedCalculators = Arc<RwLock<HashMap<String, IndexCalculator>>>;

pub struct CalculatorManager {
    pub calculators: SharedCalculators,
}

impl CalculatorManager {
    pub fn new(calculators: HashMap<String, IndexCalculator>) -> Self {
        Self {
            calculators: Arc::new(RwLock::new(calculators)),
        }
    }

    pub async fn update_price(&self, index_name: &str, key: &str, price: Decimal) {
        let mut calcs = self.calculators.write().await;
        if let Some(calc) = calcs.get_mut(index_name) {
            calc.update_price(key, price);
        }
    }
}
