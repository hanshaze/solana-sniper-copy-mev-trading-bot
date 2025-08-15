// src/shred_stream.rs
// Module to replace Geyser-based streams with Jito Shred-stream
use crate::apply_sol_flow_rules;
use crate::MIN_SOL_INFLOW_TRIGGER;
use crate::MIN_SOL_OUTFLOW_TRIGGER;
use crate::PENDING_MINTS;
use crate::PUMP_PROGRAM_PUBKEY;
use crate::{flatten_transaction_response, try_send_buy_if_allowed};
use anyhow::Context;
use backoff::future::retry;
use backoff::ExponentialBackoff;
use jito_protos::shredstream::{
    shredstream_proxy_client::ShredstreamProxyClient, SubscribeEntriesRequest,
};
use log::{debug, info, warn};
use once_cell::sync::Lazy;
use pump_interface::PumpProgramIx;
use shared_state::CURRENT_SLOT;
use shared_state::{BuyOrder, SellOrder, BOUGHT_TOKENS};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_entry::entry::Entry as SolanaEntry;
use solana_sdk::message::VersionedMessage;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::VersionedTransaction;
use solana_transaction_status::{
    ConfirmedTransactionWithStatusMeta, TransactionStatusMeta, TransactionWithStatusMeta,
    VersionedTransactionWithStatusMeta,
};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::atomic::Ordering;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;
const FALLBACK_SELL_AFTER_SLOTS: u64 = 50;

struct MintTracker {
    creator: Pubkey,
    last_activity: Instant,
    sol_inflow: f64,
    sol_outflow: f64,
    waiting_sw: bool,
    silence_until: Instant,
}

static MINT_TRACKERS: Lazy<Mutex<HashMap<String, MintTracker>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// Silence window duration before we begin listening for the "second wave"
static SILENCE_DURATION: Lazy<Duration> = Lazy::new(|| {
    let secs = std::env::var("SW_SILENCE_SECS").unwrap_or_else(|_| "60".into());
    Duration::from_secs(secs.parse().unwrap_or(60))
});


// at top of file, make sure you have:
static SECOND_WAVE_MODE: Lazy<bool> = Lazy::new(|| {
    std::env::var("SECOND_WAVE_MODE").map(|v| v == "true").unwrap_or(false)
});


/// Begin tracking a newly detected mint, recording its creator
/// and setting up the silence window before second‐wave sniping.
async fn start_tracking_mint(mint: String, creator: Pubkey) {
    let now = Instant::now();
    let silence = *SILENCE_DURATION; // from your SW_SILENCE_SECS env

    let tracker = MintTracker {
        creator,
        last_activity: now,
        sol_inflow: 0.0,
        sol_outflow: 0.0,
        waiting_sw: false,
        silence_until: now + silence,
    };

    MINT_TRACKERS.lock().await.insert(mint, tracker);
}

/// Subscribe to Jito shredstream and manage new‐mint sniping vs. second‐wave mode
pub async fn shred_subscribe_transactions(
    url: String,
    sell_sender: Sender<SellOrder>,
    mint_sender: Sender<BuyOrder>,
) -> anyhow::Result<()> {
    let backoff = ExponentialBackoff::default();
    let mut client = retry(backoff, || async {
        ShredstreamProxyClient::connect(url.clone())
            .await
            .map_err(backoff::Error::transient)
    })
    .await?;

    let mut stream = client
        .subscribe_entries(SubscribeEntriesRequest::default())
        .await?
        .into_inner();

    while let Some(Ok(batch)) = stream.next().await {
        let slot = batch.slot;
        CURRENT_SLOT.store(slot, Ordering::Relaxed);

        let entries: Vec<SolanaEntry> = bincode::deserialize(&batch.entries)?;
        for sol_entry in entries {
            for tx in sol_entry.transactions {
                if let Some((mint, creator)) = detect_raw_create(&tx) {
                    info!("New mint {} @ slot {}", mint, slot);

                    // Always start the silence‐window tracker
                    let mint_clone = mint.clone();
                    tokio::spawn(async move {
                        start_tracking_mint(mint_clone, creator).await;
                    });

                    // Only fire the *immediate* new‐mint buy if NOT in second‐wave mode
                    if !*SECOND_WAVE_MODE {
                        try_send_buy_if_allowed(
                            &mint,
                            creator,
                            true,   // first_buy
                            true,  // urgent=false
                            &mint_sender,
                        )
                        .await;
                    }
                }

                // offload your reconcilers (flows, confirms, sells)
                let sell_tx = sell_sender.clone();
                let mint_tx = mint_sender.clone();
                tokio::spawn(async move {
                    let confirmed = confirm_from_versioned(tx, slot);
                    reconcile_flows(&confirmed, &sell_tx, &mint_tx).await;
                    reconcile_buys(&confirmed, slot).await;
                    reconcile_sells(&confirmed, slot, &sell_tx).await;
                });
            }
        }
    }

    Ok(())
}
/// Update flows and trigger second-wave sniping, but only after our own buy has confirmed.
async fn reconcile_flows(
    confirmed: &ConfirmedTransactionWithStatusMeta,
    sell_sender: &Sender<SellOrder>,
    mint_sender: &Sender<BuyOrder>,
) {
    // only complete transactions
    let vtx = match &confirmed.tx_with_meta {
        TransactionWithStatusMeta::Complete(v) => v,
        _ => return,
    };
    let now = Instant::now();

    for tx_ix in flatten_transaction_response(vtx) {
        // skip non-Pump instructions
        if tx_ix.instruction.program_id != *PUMP_PROGRAM_PUBKEY {
            continue;
        }
        let mint = tx_ix.instruction.accounts[2].pubkey.to_string();

        // don’t start counting flows until our buy has landed
        {
            let tokens = BOUGHT_TOKENS.lock().await;
            if tokens.get(&mint).map(|e| e.buy_slot()).unwrap_or(0) == 0 {
                continue;
            }
        }

        if let Some(tracker) = MINT_TRACKERS.lock().await.get_mut(&mint) {
            tracker.last_activity = now;

            if let Ok(ix) = PumpProgramIx::deserialize(&tx_ix.instruction.data) {
                match ix {
                    PumpProgramIx::Buy(buy_args) => {
                        let sol_in = buy_args.max_sol_cost as f64 / 1e9;
                        tracker.sol_inflow += sol_in;
                        debug!("SOL inflow +{:.6} on {}", sol_in, mint);

                        if tracker.waiting_sw {
                            info!("🚀 Second wave hit on {} → sniping", mint);
                            try_send_buy_if_allowed(
                                &mint,
                                tracker.creator,
                                true,
                                true,
                                mint_sender,
                            ).await;
                            tracker.waiting_sw = false;
                        }
                    }
                    PumpProgramIx::Sell(sell_args) => {
                        let sol_out = sell_args.min_sol_output as f64 / 1e9;
                        tracker.sol_outflow += sol_out;
                        debug!("SOL outflow +{:.6} on {}", sol_out, mint);
                    }
                    _ => {}
                }
            }

            if !tracker.waiting_sw && now >= tracker.silence_until {
                tracker.waiting_sw = true;
                info!("⏱ {} entered second-wave watch mode", mint);
            }
        }
    }
}

/// Confirm our buy, then reset that mint’s tracker so we only count SOL flows from now on.
async fn reconcile_buys(
    confirmed: &ConfirmedTransactionWithStatusMeta,
    slot: u64,
) {
    // pull out on-chain signature (if any)
    let sig_opt = match &confirmed.tx_with_meta {
        TransactionWithStatusMeta::Complete(v) => v.transaction.signatures.get(0).cloned(),
        _ => None,
    };

    if let Some(sig) = sig_opt {
        let sig_str = sig.to_string();
        let mut tokens = BOUGHT_TOKENS.lock().await;

        // find our minted entry by matching its stored signature
        if let Some((mint, entry)) = tokens
            .iter_mut()
            .find(|(_, e)| e.signature() == Some(&sig_str))
        {
            // confirm the buy
            entry.set_buy_slot(slot);
            entry.set_buy_executed_at(Instant::now());
            PENDING_MINTS.fetch_sub(1, Ordering::Relaxed);
            info!("✅ Confirmed BUY for {} @ slot {}", mint, slot);

            // re-arm second-wave tracker
            if let Some(tracker) = MINT_TRACKERS.lock().await.get_mut(mint) {
                let now = Instant::now();
                tracker.last_activity  = now;
                tracker.sol_inflow     = 0.0;
                tracker.sol_outflow    = 0.0;
                tracker.waiting_sw     = false;
                tracker.silence_until  = now + *SILENCE_DURATION;
                info!("🔄 Re-armed second-wave tracker for {}", mint);
            }
        }
    }
}



fn detect_raw_create(vtx: &VersionedTransaction) -> Option<(String, Pubkey)> {
    let msg = &vtx.message;
    let keys = msg.static_account_keys();
    debug!(
        "🔑 static keys (len={}): {:?}",
        keys.len(),
        keys.iter().take(4).collect::<Vec<_>>()
    );

    for instr in msg.instructions() {
        let program_key = keys[instr.program_id_index as usize];
        if program_key == *PUMP_PROGRAM_PUBKEY {
            debug!(
                "🧩 Found PumpProgramIx at idx={} data_prefix={:02x?}",
                instr.program_id_index,
                &instr.data[..4.min(instr.data.len())]
            );

            match PumpProgramIx::deserialize(&instr.data) {
                Ok(PumpProgramIx::Create(args)) => {
                    let mint_key = keys[instr.accounts[0] as usize];
                    debug!("✅ Deserialized Create; mint = {}", mint_key);
                    return Some((mint_key.to_string(), args.creator));
                }
                Ok(other) => {
                    debug!("ℹ️  Other PumpProgramIx::{:?}, skipping", other);
                }
                Err(e) => {
                    debug!("❌ deserialize error: {}", e);
                }
            }
        }
    }

    None
}

/// Quickly wrap a VersionedTransaction in a ConfirmedTransactionWithStatusMeta
fn confirm_from_versioned(
    tx: solana_sdk::transaction::VersionedTransaction,
    slot: u64,
) -> ConfirmedTransactionWithStatusMeta {
    // minimal meta with defaults:
    let meta = TransactionStatusMeta {
        status: Ok(()),
        fee: 0,
        pre_balances: vec![],
        post_balances: vec![],
        inner_instructions: None,
        log_messages: None,
        pre_token_balances: None,
        post_token_balances: None,
        rewards: None,
        loaded_addresses: Default::default(),
        return_data: None,
        compute_units_consumed: None,
    };

    let vtx = VersionedTransactionWithStatusMeta {
        transaction: tx,
        meta,
    };
    ConfirmedTransactionWithStatusMeta {
        slot,
        tx_with_meta: TransactionWithStatusMeta::Complete(vtx),
        block_time: None,
    }
}
/// Decode raw bytes and slot into ConfirmedTransactionWithStatusMeta
pub fn decode_txn_from_bytes(
    raw: &[u8],
    slot: u64,
) -> anyhow::Result<ConfirmedTransactionWithStatusMeta> {
    use bincode;
    use solana_sdk::transaction::VersionedTransaction;
    use solana_transaction_status::TransactionStatusMeta;

    // 1. Deserialize the wire-format VersionedTransaction
    let tx: VersionedTransaction = bincode::deserialize(raw)
        .context("failed to deserialize VersionedTransaction from shred entry")?;

    // 2. Build a minimal TransactionStatusMeta with defaults
    let meta = TransactionStatusMeta {
        status: Ok(()),
        fee: 0,
        pre_balances: vec![],
        post_balances: vec![],
        inner_instructions: None,
        log_messages: None,
        pre_token_balances: None,
        post_token_balances: None,
        rewards: None,
        loaded_addresses: Default::default(),
        return_data: None,
        compute_units_consumed: None,
    };

    // 3. Wrap into VersionedTransactionWithStatusMeta and then into ConfirmedTransactionWithStatusMeta
    let vtx = VersionedTransactionWithStatusMeta {
        transaction: tx,
        meta,
    };
    Ok(ConfirmedTransactionWithStatusMeta {
        slot,
        tx_with_meta: TransactionWithStatusMeta::Complete(vtx),
        block_time: None,
    })
}

const LOW_BUY_FREQ_WINDOW_SLOTS: u64 = 10;
const MIN_BUYS_IN_WINDOW: usize = 6;

const BUY_CONFIRM_TIMEOUT_SECS: u64 = 60;

const SELL_CONFIRM_TIMEOUT_SECS: u64 = 60;

async fn reconcile_sells(
    confirmed: &ConfirmedTransactionWithStatusMeta,
    slot: u64,
    sell_sender: &Sender<SellOrder>,
) {
    // --- 0) remove any entries that are already confirmed sells, just in case ---
    {
        let mut tokens = BOUGHT_TOKENS.lock().await;
        let to_remove: Vec<String> = tokens
            .iter()
            .filter_map(|(mint, entry)| {
                if entry.sell_confirmed() {
                    Some(mint.clone())
                } else {
                    None
                }
            })
            .collect();
        for mint in to_remove {
            tokens.remove(&mint);
        }
    }

    // 1) On-chain confirmations
    if let Some(sig) = match &confirmed.tx_with_meta {
        TransactionWithStatusMeta::Complete(v) => {
            v.transaction.signatures.get(0).map(|s| s.to_string())
        }
        _ => None,
    } {
        // find mint without holding a mutable borrow across removal
        let mint_opt = {
            let tokens = BOUGHT_TOKENS.lock().await;
            tokens
                .iter()
                .find(|(_, e)| e.sell_signature().as_deref() == Some(&sig))
                .map(|(m, _)| m.clone())
        };

        if let Some(mint) = mint_opt {
            // confirm + slot in one lock
            {
                let mut tokens = BOUGHT_TOKENS.lock().await;
                if let Some(entry) = tokens.get_mut(&mint) {
                    entry.confirm_sell();
                    entry.set_sell_slot(slot);
                }
                // guard against underflow
                if PENDING_MINTS.load(Relaxed) > 0 {
                    PENDING_MINTS.fetch_sub(1, Relaxed);
                }
            }

            // remove in a separate lock
            {
                let mut tokens = BOUGHT_TOKENS.lock().await;
                tokens.remove(&mint);
            }

            log::info!("✅ Confirmed SELL for {} @ slot {}", mint, slot);
            return;
        }
    }

    // 2) retry logic
    // build stale list under lock
    let now = Instant::now();
    let timeout = Duration::from_secs(SELL_CONFIRM_TIMEOUT_SECS);
    let stale: Vec<String> = {
        let tokens = BOUGHT_TOKENS.lock().await;
        tokens
            .iter()
            .filter_map(|(mint, entry)| {
                if entry.sell_signature().is_some()
                    && !entry.sell_confirmed()
                    && entry.retry_count() < 3
                    && entry
                        .sell_executed_at()
                        .map(|t| now.duration_since(t) > timeout)
                        .unwrap_or(false)
                {
                    Some(mint.clone())
                } else {
                    None
                }
            })
            .collect()
    };

    // drop lock, then re-send each stale sell
    for mint in stale {
        let mut tokens = BOUGHT_TOKENS.lock().await;
        if let Some(entry) = tokens.get_mut(&mint) {
            entry.increment_retry_count();
            log::info!(
                "⌛ Sell for {} timed out; retry #{}, re-sending…",
                mint,
                entry.retry_count()
            );
            let amount = entry.amount();
            let _ = sell_sender
                .send(SellOrder {
                    mint: mint.clone(),
                    amount,
                    use_jito: true,
                    urgent: true,
                })
                .await;
            // record when we issued this retry
            entry.set_sell_executed_at(Instant::now());
        }
    }
}

/// Find mint & creator from Create ix
fn extract_mint_and_creator(
    confirmed: &ConfirmedTransactionWithStatusMeta,
) -> Option<(String, Pubkey)> {
    if let TransactionWithStatusMeta::Complete(v) = &confirmed.tx_with_meta {
        for tx_ix in flatten_transaction_response(v) {
            if tx_ix.instruction.program_id == *crate::PUMP_PROGRAM_PUBKEY {
                if let Ok(PumpProgramIx::Create(args)) =
                    PumpProgramIx::deserialize(&tx_ix.instruction.data)
                {
                    // the very first account is the mint
                    if let Some(meta) = tx_ix.instruction.accounts.get(0) {
                        // return (mint_pubkey_string, creator_pubkey)
                        return Some((meta.pubkey.to_string(), args.creator));
                    }
                }
            }
        }
    }
    None
}
