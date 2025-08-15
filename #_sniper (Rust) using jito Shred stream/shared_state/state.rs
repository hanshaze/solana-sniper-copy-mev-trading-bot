// shared_state/src/lib.rs
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use lazy_static::lazy_static;

#[derive(Debug, Clone)]
pub struct TokenTrackingState {
    pub mint: String,
    pub first_buyer: Option<String>,
    pub first_buy_amount: f64,
    pub suspected_creators: HashSet<String>,
    pub sell_detected: bool,
    pub sell_split_ratio: usize,
}


lazy_static! {
    pub static ref TRACKED_TOKENS: Arc<Mutex<HashMap<String, TokenTrackingState>>> =
        Arc::new(Mutex::new(HashMap::new()));
}
