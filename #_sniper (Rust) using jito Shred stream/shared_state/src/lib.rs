// shared_state/src/lib.rs
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::AtomicUsize;
use std::time::Instant;
use tokio::sync::Mutex;
/// A request to sell a mint for a given amount, marking if it's urgent.
#[derive(Debug, Clone)]
pub struct SellOrder {
    pub mint: String,
    pub amount: u64,
    pub use_jito: bool, // Indicates if Jito should be used for this sell
    pub urgent: bool,
}

#[derive(Debug, Clone)]
pub struct BuyOrder {
    pub mint: String,
    pub creator: Pubkey,
    pub use_jito: bool,
    pub urgent: bool,
}
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoughtTokenInfo {
    mint: String,
    ata: Option<String>,
    amount: u64,
    fallback_amount: u64,
    timestamp: Instant,
    last_activity: Instant,
    signature: Option<String>,
    follow_up_buys: u32,
    sell_triggered: bool,
    first_sell_detected: bool,
    sol_inflow: f64,
    sol_outflow: f64,
    creator_vault: Option<Pubkey>,
    unique_buyers: std::collections::HashSet<String>,

    suspected_creator: Option<String>,
    mint_detected_at: Instant,
    buy_executed_at: Option<Instant>,
    sell_triggered_at: Option<Instant>,
    ata_balance_zeroed: bool,
    pub sell_issued_slot: u64,
    sell_confirmed: bool,
    pub buy_slot: u64,
    pub sell_slot: Option<u64>,
    pub sell_signature: Option<String>,
    sell_executed_at: Option<std::time::Instant>,
    sell_retry_count: u8,
}

impl BoughtTokenInfo {
    pub fn new(
        mint: String,
        ata: Option<String>,
        fallback_amount: u64,
        timestamp: Instant,
        signature: Option<String>,
        buy_slot: u64,
        creator_vault: Option<Pubkey>, // new param
    ) -> Self {
        Self {
            mint,
            ata,
            amount: fallback_amount,
            fallback_amount,
            timestamp,
            last_activity: timestamp,
            signature,
            follow_up_buys: 0,
            sell_triggered: false,
            first_sell_detected: false,
            sol_inflow: 0.0,
            sol_outflow: 0.0,
            unique_buyers: std::collections::HashSet::new(),
            suspected_creator: None,
            mint_detected_at: timestamp,
            buy_executed_at: None,
            sell_triggered_at: None,
            ata_balance_zeroed: false,
            buy_slot,
            sell_slot: None,
            sell_signature: None,
            sell_issued_slot: 0,
            sell_confirmed: false,
            creator_vault, // initialize the creator vault
            sell_executed_at: None,
            sell_retry_count: 0,
        }
    }

    pub fn creator_vault(&self) -> Option<&Pubkey> {
        self.creator_vault.as_ref()
    }

    pub fn mint(&self) -> &String {
        &self.mint
    }

    pub fn buy_slot(&self) -> u64 {
        self.buy_slot
    }
    pub fn set_buy_slot(&mut self, slot: u64) {
        self.buy_slot = slot;
    }

    pub fn sell_slot(&self) -> Option<u64> {
        self.sell_slot
    }

    pub fn set_sell_slot(&mut self, slot: u64) {
        self.sell_slot = Some(slot);
    }

    pub fn set_buy_executed_at(&mut self, when: Instant) {
        self.buy_executed_at = Some(when);
    }
    pub fn buy_executed_at(&self) -> Option<Instant> {
        self.buy_executed_at
    }

    pub fn record_sell(&mut self, sig: String, slot: u64) {
        self.sell_signature = Some(sig);
        self.sell_issued_slot = slot;
        self.sell_slot = Some(slot);
        self.sell_confirmed = false;
    }

    pub fn confirm_sell(&mut self) {
        self.sell_confirmed = true;
    }

    pub fn sell_confirmed(&self) -> bool {
        self.sell_confirmed
    }

    pub fn sell_issued_slot(&self) -> u64 {
        self.sell_issued_slot
    }

    pub fn set_sell_triggered_at(&mut self, when: Instant) {
        self.sell_triggered_at = Some(when);
    }
    pub fn sell_triggered_at(&self) -> Option<Instant> {
        self.sell_triggered_at
    }

    pub fn set_suspected_creator(&mut self, creator: String) {
        self.suspected_creator = Some(creator);
    }
    pub fn suspected_creator(&self) -> Option<&String> {
        self.suspected_creator.as_ref()
    }

    pub fn ata_balance_zeroed(&self) -> bool {
        self.ata_balance_zeroed
    }
    pub fn set_ata_balance_zeroed(&mut self, value: bool) {
        self.ata_balance_zeroed = value;
    }

    pub fn sol_inflow(&self) -> f64 {
        self.sol_inflow
    }
    pub fn add_sol_inflow(&mut self, amount: f64) {
        self.sol_inflow += amount;
        self.update_last_activity();
    }
    pub fn sol_outflow(&self) -> f64 {
        self.sol_outflow
    }
    pub fn add_sol_outflow(&mut self, amount: f64) {
        self.sol_outflow += amount;
        self.update_last_activity();
    }
    pub fn ata(&self) -> Option<&String> {
        self.ata.as_ref()
    }
    pub fn amount(&self) -> u64 {
        self.amount
    }
    pub fn set_amount(&mut self, amt: u64) {
        self.amount = amt;
    }
    pub fn fallback_amount(&self) -> u64 {
        self.fallback_amount
    }
    pub fn timestamp(&self) -> Instant {
        self.timestamp
    }
    pub fn last_activity(&self) -> Instant {
        self.last_activity
    }
    pub fn update_last_activity(&mut self) {
        self.last_activity = Instant::now();
    }
    pub fn signature(&self) -> Option<&String> {
        self.signature.as_ref()
    }
    pub fn set_sell_signature(&mut self, sig: String) {
        self.sell_signature = Some(sig);
    }
    pub fn sell_signature(&self) -> Option<&String> {
        self.sell_signature.as_ref()
    }
    pub fn follow_up_buys(&self) -> u32 {
        self.follow_up_buys
    }
    pub fn increment_follow_up_buys(&mut self) {
        self.follow_up_buys += 1;
        self.update_last_activity();
    }
    pub fn sell_triggered(&self) -> bool {
        self.sell_triggered
    }
    pub fn set_sell_triggered(&mut self, v: bool) {
        self.sell_triggered = v;
    }
    pub fn first_sell_detected(&self) -> bool {
        self.first_sell_detected
    }
    pub fn mark_first_sell_detected(&mut self) {
        self.first_sell_detected = true;
        self.update_last_activity();
    }
    pub fn zero_amount(&mut self) {
        self.amount = 0;
    }
    pub fn unique_buyers(&self) -> &std::collections::HashSet<String> {
        &self.unique_buyers
    }
    pub fn add_unique_buyer(&mut self, addr: String) -> bool {
        let ins = self.unique_buyers.insert(addr);
        if ins {
            self.update_last_activity();
        }
        ins
    }
    pub fn sell_executed_at(&self) -> Option<Instant> {
        self.sell_executed_at
    }
    pub fn set_sell_executed_at(&mut self, t: Instant) {
        self.sell_executed_at = Some(t);
    }
    pub fn retry_count(&self) -> u8 {
        self.sell_retry_count
    }
    pub fn increment_retry_count(&mut self) {
        self.sell_retry_count = self.sell_retry_count.saturating_add(1);
    }
    pub fn reset_retry_count(&mut self) {
        self.sell_retry_count = 0;
    }
}

lazy_static! {
    pub static ref BOUGHT_TOKENS: Mutex<HashMap<String, BoughtTokenInfo>> =
        Mutex::new(HashMap::new());
}

pub static CURRENT_SLOT: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(0));

pub static PENDING_MINTS: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(0));
