// tx.rs

use crate::{
    common::{logger::Logger, rpc},
    services::{jito, nextblock},
};
use anyhow::{anyhow, Result};
use base64;
use once_cell::sync::Lazy;
use reqwest::Client as HttpClient;
use serde_json::json;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, hash::Hash, instruction::Instruction,
    signature::Keypair, signer::Signer, system_instruction, transaction::Transaction,
};
use spl_token::ui_amount_to_amount;
use std::{env, sync::Arc, time::Duration};
use tokio::{spawn, sync::Mutex, time::Instant};

//——————————————————————————————————————————————————
// 1) One global HTTP client for all block‐engine calls
//——————————————————————————————————————————————————
static HTTP_CLIENT: Lazy<HttpClient> = Lazy::new(|| {
    HttpClient::builder()
        .pool_max_idle_per_host(0)
        .build()
        .expect("failed to build HTTP client")
});

//——————————————————————————————————————————————————
// 2) Cached blockhash, refreshed every 500 ms in the background
//——————————————————————————————————————————————————
pub static LATEST_BLOCKHASH: Lazy<Mutex<Hash>> = Lazy::new(|| Mutex::new(Hash::default()));

/// Spawn this once at startup to keep our blockhash fresh.
pub fn spawn_blockhash_refresher(rpc: Arc<RpcClient>) {
    spawn(async move {
        loop {
            if let Ok(hash) = rpc.get_latest_blockhash() {
                *LATEST_BLOCKHASH.lock().await = hash;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });
}

//——————————————————————————————————————————————————
// 3) One-time init of tip account & tip value (Jito vs NextBlock)
//——————————————————————————————————————————————————
static TIP_ACCOUNT: Lazy<Mutex<solana_sdk::pubkey::Pubkey>> =
    Lazy::new(|| Mutex::new(solana_sdk::pubkey::Pubkey::default()));
static TIP_VALUE: Lazy<Mutex<f64>> = Lazy::new(|| Mutex::new(0.0));

/// Call this **once** at startup (before any swaps).
/// `use_nextblock` comes from e.g. `env("BLOCK_ENGINE_PROVIDER") == "nextblock"`,
/// `use_priority_tip` from your config.
pub async fn init_tip_state(use_nextblock: bool, use_priority_tip: bool) -> Result<()> {
    if use_nextblock {
        nextblock::init_tip_accounts().await?;
        let base = nextblock::get_tip_value().await?;
        let mut acct = TIP_ACCOUNT.lock().await;
        let mut val = TIP_VALUE.lock().await;
        *acct = nextblock::get_tip_account().await?;
        *val = if use_priority_tip { base * 2.0 } else { base };
    } else {
        jito::init_tip_accounts().await?;
        let base = jito::get_tip_value().await?;
        let mut acct = TIP_ACCOUNT.lock().await;
        let mut val = TIP_VALUE.lock().await;
        *acct = jito::get_tip_account().await?;
        *val = if use_priority_tip { base * 5.0 } else { base };
    }
    Ok(())
}

//——————————————————————————————————————————————————
// 4) Pre-built compute-budget instructions
//——————————————————————————————————————————————————
fn unit_price() -> u64 {
    env::var("UNIT_PRICE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1)
}

fn unit_limit() -> u32 {
    env::var("UNIT_LIMIT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(300_000)
}

static BUDGET_INSTRUCTIONS: Lazy<Vec<Instruction>> = Lazy::new(|| {
    vec![
        ComputeBudgetInstruction::set_compute_unit_price(unit_price()),
        ComputeBudgetInstruction::set_compute_unit_limit(unit_limit()),
    ]
});

//——————————————————————————————————————————————————
// 5) The refactored send‐tx function (handles RPC, Jito & NextBlock)
//——————————————————————————————————————————————————
#[allow(deprecated, unused_variables)]
pub async fn new_signed_and_send(
    client: &RpcClient,
    keypair: &Keypair,
    mut instructions: Vec<Instruction>,
    use_priority_engine: bool, // e.g. swap_config.use_jito
    use_priority_tip: bool,    // e.g. swap_config.use_priority_tip
    logger: &Logger,
) -> Result<Vec<String>> {
    // a) If *not* using a block‐engine, prepend compute‐budget ixs
    if !use_priority_engine {
        instructions.splice(0..0, BUDGET_INSTRUCTIONS.iter().cloned());
    }

    // b) Grab our cached blockhash
    let blockhash = *LATEST_BLOCKHASH.lock().await;

    // c) Dispatch
    let start = Instant::now();
    let mut sigs = Vec::new();

    if use_priority_engine {
        // i) build tip‐transfer ix
        let tip_account = *TIP_ACCOUNT.lock().await;
        let tip_value = *TIP_VALUE.lock().await;
        let lamports = ui_amount_to_amount(tip_value.min(0.1), spl_token::native_mint::DECIMALS);
        let tip_ix = system_instruction::transfer(&keypair.pubkey(), &tip_account, lamports);

        // ii) weave it into a fresh txn
        let mut pe_ixs = Vec::with_capacity(instructions.len() + 1);
        pe_ixs.push(tip_ix);
        pe_ixs.extend(instructions);
        let pe_tx = Transaction::new_signed_with_payer(
            &pe_ixs,
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );

        // iii) encode + HTTP‐POST to NextBlock or Jito
        let encoded = base64::encode(bincode::serialize(&pe_tx)?);
        let payload = json!({
            "transaction": { "content": encoded },
            "frontRunningProtection": true,
        });

        let engine_is_nextblock = env::var("BLOCK_ENGINE_PROVIDER")
            .unwrap_or_default()
            .to_lowercase()
            == "nextblock";

        if engine_is_nextblock {
            // — NextBlock —
            let auth = env::var("NEXTBLOCK_AUTH").unwrap_or_default();
            let resp = HTTP_CLIENT
                .post("https://ny.nextblock.io/api/v2/submit")
                .header("Content-Type", "application/json")
                .header("Authorization", auth)
                .json(&payload)
                .send()
                .await?;

            if !resp.status().is_success() {
                return Err(anyhow!(
                    "NextBlock error {}: {}",
                    resp.status(),
                    resp.text().await?
                ));
            }
            let body = resp.json::<serde_json::Value>().await?;
            if let Some(sig) = body.get("signature").and_then(|v| v.as_str()) {
                sigs.push(sig.to_string());
            } else {
                return Err(anyhow!("Missing signature in NextBlock response"));
            }
        } else {
            // — Jito —
            let url = format!("{}/api/v1/transactions", *jito::BLOCK_ENGINE_URL);
            let rpc_resp = HTTP_CLIENT
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&json!({
                    "id": 1,
                    "jsonrpc": "2.0",
                    "method": "sendTransaction",
                    "params": [encoded, { "encoding": "base64" }]
                }))
                .send()
                .await?;

            if !rpc_resp.status().is_success() {
                return Err(anyhow!(
                    "Jito error {}: {}",
                    rpc_resp.status(),
                    rpc_resp.text().await?
                ));
            }
            let body = rpc_resp.json::<serde_json::Value>().await?;
            if let Some(sig) = body.get("result").and_then(|v| v.as_str()) {
                sigs.push(sig.to_string());
            } else {
                return Err(anyhow!("Missing signature in Jito response"));
            }
        }

        logger.log(format!("✅ Sent via block‐engine in {:?}", start.elapsed()));
    } else {
        // — standard RPC path —
        let std_tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );
        let sig = rpc::send_txn(client, &std_tx, true)?;
        sigs.push(sig.to_string());
        logger.log("✅ Sent via standard RPC".into());
    }

    Ok(sigs)
}
