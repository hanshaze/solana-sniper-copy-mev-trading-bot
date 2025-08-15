use anyhow::{anyhow, Result};
use rand::{seq::IteratorRandom, thread_rng};
use solana_sdk::pubkey::Pubkey;
use std::{str::FromStr, sync::LazyLock};
use tokio::sync::RwLock;

use crate::common::utils::import_env_var;

// Endpoint and auth token from env
pub static NEXTBLOCK_API_URL: LazyLock<String> =
    LazyLock::new(|| import_env_var("NEXTBLOCK_API_URL"));
pub static NEXTBLOCK_AUTH_HEADER: LazyLock<String> =
    LazyLock::new(|| import_env_var("NEXTBLOCK_AUTH_HEADER"));
pub static NEXTBLOCK_TIP_VALUE: LazyLock<String> =
    LazyLock::new(|| import_env_var("NEXTBLOCK_TIP_VALUE"));

// List of hardcoded tip accounts
pub static TIP_ACCOUNTS: LazyLock<RwLock<Vec<String>>> = LazyLock::new(|| RwLock::new(vec![]));

#[derive(Debug)]
pub struct TipAccountResult {
    pub accounts: Vec<String>,
}

pub async fn init_tip_accounts() -> Result<()> {
    let accounts = TipAccountResult {
        accounts: vec![
            "NextbLoCkVtMGcV47JzewQdvBpLqT9TxQFozQkN98pE".to_string(),
            "NexTbLoCkWykbLuB1NkjXgFWkX9oAtcoagQegygXXA2".to_string(),
            "NeXTBLoCKs9F1y5PJS9CKrFNNLU1keHW71rfh7KgA1X".to_string(),
            "NexTBLockJYZ7QD7p2byrUa6df8ndV2WSd8GkbWqfbb".to_string(),
            "neXtBLock1LeC67jYd1QdAa32kbVeubsfPNTJC1V5At".to_string(),
            "nEXTBLockYgngeRmRrjDV31mGSekVPqZoMGhQEZtPVG".to_string(),
            "NEXTbLoCkB51HpLBLojQfpyVAMorm3zzKg7w9NFdqid".to_string(),
            "nextBLoCkPMgmG8ZgJtABeScP35qLa2AMCNKntAP7Xc".to_string(),
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
            println!("nextblock: failed to parse Pubkey: {:?}", err);
        })?),
        None => Err(anyhow!("nextblock: no tip accounts available")),
    }
}

// unit sol
pub async fn get_tip_value() -> Result<f64> {
    // If TIP_VALUE is set, use it
    if let Ok(tip_value) = std::env::var("NEXTBLOCK_TIP_VALUE") {
        match f64::from_str(&tip_value) {
            Ok(value) => Ok(value),
            Err(_) => {
                println!(
                    "Invalid NEXTBLOCK_TIP_VALUE in environment variable: '{}'. Falling back to percentile calculation.",
                    tip_value
                );
                Err(anyhow!("Invalid TIP_VALUE in environment variable"))
            }
        }
    } else {
        Err(anyhow!("NEXTBLOCK_TIP_VALUE environment variable not set"))
    }
}
