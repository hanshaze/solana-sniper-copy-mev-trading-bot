use std::{future::Future, str::FromStr, sync::LazyLock, time::Duration};

use anyhow::{anyhow, Result};
use indicatif::{ProgressBar, ProgressStyle};
use rand::{seq::IteratorRandom, thread_rng};
use serde::Deserialize;
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;
use tokio::{
    sync::RwLock,
    time::{sleep, Instant},
};

use crate::common::utils::import_env_var;

pub static BLOCK_ENGINE_URL: LazyLock<String> =
    LazyLock::new(|| import_env_var("ZEROSLOT_BLOCK_ENGINE_URL"));
pub static TIP_STREAM_URL: LazyLock<String> =
    LazyLock::new(|| import_env_var("ZEROSLOT_TIP_STREAM_URL"));
pub static TIP_PERCENTILE: LazyLock<String> =
    LazyLock::new(|| import_env_var("ZEROSLOT_TIP_PERCENTILE"));

pub static TIP_ACCOUNTS: LazyLock<RwLock<Vec<String>>> = LazyLock::new(|| RwLock::new(vec![]));

#[derive(Debug)]
pub struct TipAccountResult {
    pub accounts: Vec<String>,
}

pub async fn init_tip_accounts() -> Result<()> {
    let accounts = TipAccountResult {
        accounts: vec![
            "Eb2KpSC8uMt9GmzyAEm5Eb1AAAgTjRaXWFjKyFXHZxF3".to_string(),
            "FCjUJZ1qozm1e8romw216qyfQMaaWKxWsuySnumVCCNe".to_string(),
            "ENxTEjSQ1YabmUpXAdCgevnHQ9MHdLv8tzFiuiYJqa13".to_string(),
            "6rYLG55Q9RpsPGvqdPNJs4z5WTxJVatMB8zV3WJhs5EK".to_string(),
            "Cix2bHfqPcKcM233mzxbLk14kSggUUiz2A87fJtGivXr".to_string(),
        ],
    };
    let mut tip_accounts = TIP_ACCOUNTS.write().await;

    accounts
        .accounts
        .iter()
        .for_each(|account| tip_accounts.push(account.to_string()));
    Ok(())
}

pub async fn get_tip_account() -> Result<Pubkey> {
    let accounts = TIP_ACCOUNTS.read().await;
    let mut rng = thread_rng();
    match accounts.iter().choose(&mut rng) {
        Some(acc) => Ok(Pubkey::from_str(acc).inspect_err(|err| {
            println!("jito: failed to parse Pubkey: {:?}", err);
        })?),
        None => Err(anyhow!("jito: no tip accounts available")),
    }
}
// unit sol
pub async fn get_tip_value() -> Result<f64> {
    // If TIP_VALUE is set, use it
    if let Ok(tip_value) = std::env::var("ZEROSLOT_TIP_VALUE") {
        match f64::from_str(&tip_value) {
            Ok(value) => Ok(value),
            Err(_) => {
                println!(
                    "Invalid ZEROSLOT_TIP_VALUE in environment variable: '{}'. Falling back to percentile calculation.",
                    tip_value
                );
                Err(anyhow!(
                    "Invalid ZEROSLOT_TIP_VALUE in environment variable"
                ))
            }
        }
    } else {
        Err(anyhow!("ZEROSLOT_TIP_VALUE environment variable not set"))
    }
}

pub async fn fetch_bundle_status(bundle_id: String) -> Result<Vec<serde_json::Value>> {
    // Example implementation using reqwest:
    use reqwest::Client;
    let url = format!(
        "{}/bundle/{}",
        crate::common::utils::import_env_var("JITO_BLOCK_ENGINE_URL"),
        bundle_id
    );
    let client = Client::new();
    let resp = client.get(&url).send().await?;
    let json: Vec<serde_json::Value> = resp.json().await?;
    Ok(json)
}

#[derive(Deserialize, Debug)]
pub struct BundleStatus {
    pub bundle_id: String,
    pub transactions: Vec<String>,
    pub slot: u64,
    pub confirmation_status: String,
    pub err: ErrorStatus,
}

#[derive(Deserialize, Debug)]
pub struct ErrorStatus {
    #[serde(rename = "Ok")]
    pub ok: Option<()>,
}

pub async fn wait_for_bundle_confirmation<F, Fut>(
    fetch_statuses: F,
    bundle_id: String,
    interval: Duration,
    timeout: Duration,
) -> Result<Vec<String>>
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = Result<Vec<Value>>>,
{
    let progress_bar = new_progress_bar();
    let start_time = Instant::now();

    loop {
        let statuses = fetch_statuses(bundle_id.clone()).await?;

        if let Some(status) = statuses.first() {
            let bundle_status: BundleStatus =
                serde_json::from_value(status.clone()).inspect_err(|err| {
                    println!(
                        "Failed to parse JSON when get_bundle_statuses, err: {}",
                        err,
                    );
                })?;

            println!("{:?}", bundle_status);
            match bundle_status.confirmation_status.as_str() {
                "finalized" | "confirmed" => {
                    progress_bar.finish_and_clear();
                    println!(
                        "Finalized bundle {}: {}",
                        bundle_id, bundle_status.confirmation_status
                    );
                    // print tx
                    bundle_status
                        .transactions
                        .iter()
                        .for_each(|tx| println!("https://solscan.io/tx/{}", tx));
                    return Ok(bundle_status.transactions);
                }
                _ => {
                    progress_bar.set_message(format!(
                        "Finalizing bundle {}: {}",
                        bundle_id, bundle_status.confirmation_status
                    ));
                }
            }
        } else {
            progress_bar.set_message(format!("Finalizing bundle {}: {}", bundle_id, "None"));
        }

        // check loop exceeded 1 minute,
        if start_time.elapsed() > timeout {
            println!("Loop exceeded {:?}, breaking out.", timeout);
            return Err(anyhow!("Bundle status get timeout"));
        }

        // Wait for a certain duration before retrying
        sleep(interval).await;
    }
}
pub fn new_progress_bar() -> ProgressBar {
    let progress_bar = ProgressBar::new(42);
    progress_bar.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {wide_msg}")
            .expect("ProgressStyle::template direct input to be correct"),
    );
    progress_bar.enable_steady_tick(Duration::from_millis(100));
    progress_bar
}
