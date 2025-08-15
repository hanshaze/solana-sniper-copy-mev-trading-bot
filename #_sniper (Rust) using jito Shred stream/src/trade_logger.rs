// trade_logger.rs
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::str::FromStr;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
// Remove std::io::BufReader / std::fs::File imports for async loading:
use std::collections::HashSet;
use std::sync::RwLock;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

static SKIP_CREATORS: Lazy<RwLock<HashSet<Pubkey>>> = Lazy::new(|| RwLock::new(HashSet::new()));

/// Full record to serialize:
#[derive(Serialize, Deserialize)]
pub struct TradeRecord {
    pub mint: String,
    pub creator: String,
    pub buy_lamports: u64,
    pub tokens_received: u64,
    pub buy_slot: u64,
    pub sell_lamports: u64,
    pub tokens_sold: u64,
    pub sell_slot: u64,
    pub pnl_lamports: i128,
    pub timestamp: i64, // unix secs
}

// At startup, load trade_records.jsonl and populate SKIP_CREATORS.
// Assumes each line is a JSON object matching TradeRecord.
pub async fn load_skip_creators_from_file(path: &str) -> anyhow::Result<()> {
    // Use tokio::fs::File so we can await.
    let file = match File::open(path).await {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // No file yet: nothing to skip
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // Acquire write lock synchronously (no `.await`).
    let mut skip_set = SKIP_CREATORS.write().unwrap();
    while let Some(line) = lines.next_line().await? {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match serde_json::from_str::<TradeRecord>(line) {
            Ok(rec) => {
                if rec.pnl_lamports < 0 {
                    if let Ok(pubkey) = Pubkey::from_str(&rec.creator) {
                        skip_set.insert(pubkey);
                    }
                }
            }
            Err(_) => {
                log::warn!("Invalid trade record line: {}", line);
            }
        }
    }
    Ok(())
}

// When a new trade record is written and is negative, also update SKIP_CREATORS in-memory:
pub async fn record_trade_and_maybe_skip(path: &str, rec: &TradeRecord) -> anyhow::Result<()> {
    // append to file asynchronously
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await?;
    let json = serde_json::to_string(rec)?;
    file.write_all(json.as_bytes()).await?;
    file.write_all(b"\n").await?;
    if rec.pnl_lamports < 0 {
        if let Ok(pubkey) = Pubkey::from_str(&rec.creator) {
            // Acquire write lock synchronously:
            SKIP_CREATORS.write().unwrap().insert(pubkey);
        }
    }
    Ok(())
}

/// In-memory partials keyed by mint:
static PARTIALS: Lazy<Mutex<HashMap<String, PartialTrade>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

struct PartialTrade {
    creator: String,
    buy_lamports: u64,
    tokens_received: u64,
    buy_slot: u64,
}

/// Call when a buy is confirmed:
pub async fn record_buy(
    mint: &str,
    creator: &Pubkey,
    buy_lamports: u64,
    tokens_received: u64,
    buy_slot: u64,
) {
    let mut map = PARTIALS.lock().await;
    map.insert(
        mint.to_string(),
        PartialTrade {
            creator: creator.to_string(),
            buy_lamports,
            tokens_received,
            buy_slot,
        },
    );
}

/// Call when a sell is confirmed:
pub async fn record_sell(mint: &str, sell_lamports: u64, tokens_sold: u64, sell_slot: u64) {
    let mut map = PARTIALS.lock().await;
    if let Some(partial) = map.remove(mint) {
        let pnl = sell_lamports as i128 - partial.buy_lamports as i128;
        let record = TradeRecord {
            mint: mint.to_string(),
            creator: partial.creator.clone(),
            buy_lamports: partial.buy_lamports,
            tokens_received: partial.tokens_received,
            buy_slot: partial.buy_slot,
            sell_lamports,
            tokens_sold,
            sell_slot,
            pnl_lamports: pnl,
            timestamp: chrono::Utc::now().timestamp(),
        };
        // Append to file and update skip-set if needed
        if let Err(e) = record_trade_and_maybe_skip("trade_records.jsonl", &record).await {
            log::error!("Failed to append trade record: {:?}", e);
        }
    }
}
