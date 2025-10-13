use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Exchange {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TickerData {
    pub last_pr: String,
    pub ts: String,
    pub inst_id: String,
}
