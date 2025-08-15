// pump_fun.rs
use crate::common::utils::SwapInput;
use crate::{
    common::{logger::Logger, utils::SwapConfig},
    core::tx,
    engine::swap::SwapDirection,
};
use anyhow::{anyhow, Result};
use borsh::{from_slice, to_vec, BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use shared_state::BoughtTokenInfo;
use shared_state::BOUGHT_TOKENS;
use shared_state::CURRENT_SLOT;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    system_program,
};
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account as create_ata;
use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::time::Instant;
use std::{str::FromStr, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tracing::info;
// Define the 8-byte discriminator for the pump swap instruction.
// (These bytes must match what the on‑chain program expects.)
const PUMP_SWAP_DISCRIMINATOR: [u8; 8] = [102, 6, 61, 18, 1, 218, 235, 234];

pub const TEN_THOUSAND: u64 = 10000;
pub const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
pub const RENT_PROGRAM: &str = "SysvarRent111111111111111111111111111111111";
pub const ASSOCIATED_TOKEN_PROGRAM: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
pub const PUMP_GLOBAL: &str = "4wTV1YmiEkRvAtNtsSGPtUrqRYQMe5SKy2uB4Jjaxnjf";
pub const PUMP_FEE_RECIPIENT: &str = "CebN5WGQ4jvEPvsVU4EoHEpgzq1VV7AbicfhtW4xC9iM";
pub const PUMP_PROGRAM: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
// pub const PUMP_FUN_MINT_AUTHORITY: &str = "TSLvdd1pWpHVjahSpsvCXUbgwsL3JAcvokwaKt1eokM";
pub const PUMP_ACCOUNT: &str = "Ce6TQqeHC9p8KetsN6JsjHK7UTZk7nasjjnr7XxXp9F1";
pub const PUMP_BUY_METHOD: u64 = 16927863322537952870;
pub const PUMP_SELL_METHOD: u64 = 12502976635542562355;

/// This struct will be serialized (after the discriminator) as the instruction data.
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct PumpBuyInstruction {
    /// The minimum number of tokens the buyer expects to receive.
    pub amount: u64,
    /// The maximum SOL (in lamports) the buyer is willing to spend.
    pub max_sol_cost: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct PumpSellInstruction {
    /// Amount of tokens to sell
    pub amount: u64,
    /// Minimum SOL to receive
    pub min_sol_output: u64,
}

pub struct Pump {
    pub rpc_nonblocking_client: Arc<solana_client::nonblocking::rpc_client::RpcClient>,
    pub keypair: Arc<Keypair>,
    pub rpc_client: Option<Arc<solana_client::rpc_client::RpcClient>>,
    created_atas: Arc<Mutex<HashSet<Pubkey>>>,
}

impl Pump {
    pub fn new(
        rpc_nonblocking_client: Arc<solana_client::nonblocking::rpc_client::RpcClient>,
        rpc_client: Arc<solana_client::rpc_client::RpcClient>,
        keypair: Arc<Keypair>,
    ) -> Self {
        Self {
            rpc_nonblocking_client,
            keypair,
            rpc_client: Some(rpc_client),
            created_atas: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub async fn swap(
        &self,
        mint: &str,
        swap_config: SwapConfig,
        swap_input: SwapInput,
        amount_in_lamports: u64,
    ) -> Result<Vec<String>> {
        match swap_config.swap_direction {
            SwapDirection::Buy => {
                self.swap_buy(mint, swap_config, &swap_input, amount_in_lamports)
                    .await
            }
            SwapDirection::Sell => self.swap_sell(mint, swap_config).await,
        }
    }

    async fn swap_buy(
        &self,
        _mint: &str,
        swap_config: SwapConfig,
        swap_input: &SwapInput,
        amount_in_lamports: u64,
    ) -> Result<Vec<String>> {
        let creator_vault = swap_input
            .creator_vault
            .ok_or_else(|| anyhow!("creator_vault not provided in SwapInput"))?;

        // still need a Logger for the lower‐level TX helper
        let logger = Logger::new("[BUY SWAP] => ".to_string());
        let owner = self.keypair.pubkey();
        let mint_pubkey = swap_input.output_token_mint;
        let pump_program = Pubkey::from_str(PUMP_PROGRAM)?;

        // derive PDAs
        let bonding_curve = get_pda(&mint_pubkey, &pump_program)?;
        let associated_bonding_curve = get_associated_token_address(&bonding_curve, &mint_pubkey);

        // derive ATA locally
        let out_ata =
            spl_associated_token_account::get_associated_token_address(&owner, &mint_pubkey);

        // 1) ensure user ATA exists via cache
        let mut instructions = Vec::new();
        {
            let cache = self.created_atas.lock().await;
            if !cache.contains(&out_ata) {
                instructions.push(create_ata(&owner, &owner, &mint_pubkey, &spl_token::ID));
            }
        }

        // 2) build the buy instruction data
        let min_tokens: u64 = std::env::var("MIN_TOKENS_EXPECTED")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8_000_000_000_000);
        let mut data = PUMP_SWAP_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&to_vec(&PumpBuyInstruction {
            amount: min_tokens,
            max_sol_cost: amount_in_lamports,
        })?);

        // 3) assemble accounts in *exact* IDL order
        let global = Pubkey::find_program_address(&[b"global"], &pump_program).0;
        let fee_recipient = Pubkey::from_str(PUMP_FEE_RECIPIENT)?;
        let event_authority = Pubkey::from_str(PUMP_ACCOUNT)?;
        let accounts = vec![
            AccountMeta::new_readonly(global, false),             //  1
            AccountMeta::new(fee_recipient, false),               //  2
            AccountMeta::new_readonly(mint_pubkey, false),        //  3
            AccountMeta::new(bonding_curve, false),               //  4
            AccountMeta::new(associated_bonding_curve, false),    //  5
            AccountMeta::new(out_ata, false),                     //  6
            AccountMeta::new(owner, true),                        //  7
            AccountMeta::new_readonly(system_program::ID, false), //  8
            AccountMeta::new_readonly(spl_token::ID, false),      //  9
            AccountMeta::new(creator_vault, false),               // 10
            AccountMeta::new_readonly(event_authority, false),    // 11
            AccountMeta::new_readonly(pump_program, false),       // 12
        ];

        info!(
            "⛓ Preparing BUY for {}: min_tokens={}, max_sol={}; accounts={:?}",
            mint_pubkey,
            min_tokens,
            amount_in_lamports,
            accounts
                .iter()
                .map(|m| m.pubkey.to_string())
                .collect::<Vec<_>>()
        );

        // 4) push your swap instruction
        instructions.push(Instruction {
            program_id: pump_program,
            accounts,
            data,
        });

        // 5) dispatch
        let tx_result = tx::new_signed_and_send(
            self.rpc_client.as_ref().unwrap(),
            &self.keypair,
            instructions,
            swap_config.use_jito,
            swap_config.use_priority_tip,
            &logger,
        )
        .await;

        // 6) record on success
        if let Ok(sigs) = &tx_result {
            let buy_slot = CURRENT_SLOT.load(std::sync::atomic::Ordering::Relaxed);
            let mut tokens = BOUGHT_TOKENS.lock().await;
            let mut info = BoughtTokenInfo::new(
                mint_pubkey.to_string(),
                Some(out_ata.to_string()),
                min_tokens,
                Instant::now(),
                sigs.first().cloned(),
                buy_slot,
                Some(creator_vault),
            );
            info.set_buy_executed_at(Instant::now());
            tokens.insert(mint_pubkey.to_string(), info);

            info!(
                "✅ BUY sent for {}: sigs={:?} @ slot={}",
                mint_pubkey, sigs, buy_slot
            );
        }

        tx_result
    }

    async fn swap_sell(&self, mint: &str, swap_config: SwapConfig) -> Result<Vec<String>> {
        // logger still used for the helper
        let logger = Logger::new("[SELL SWAP] => ".to_string());
        let owner = self.keypair.pubkey();
        let mint_pubkey = Pubkey::from_str(mint)?;
        let pump_program = Pubkey::from_str(PUMP_PROGRAM)?;

        // derive PDAs
        let global_pda = Pubkey::find_program_address(&[b"global"], &pump_program).0;
        let fee_recipient = Pubkey::from_str(PUMP_FEE_RECIPIENT)?;
        let bonding_curve = get_pda(&mint_pubkey, &pump_program)?;
        let associated_bonding_curve = get_associated_token_address(&bonding_curve, &mint_pubkey);
        let event_authority = Pubkey::from_str(PUMP_ACCOUNT)?;

        // fetch stored info including creator_vault
        let (in_ata, sell_amount, creator_vault) = {
            let tokens = BOUGHT_TOKENS.lock().await;
            let info = tokens
                .get(mint)
                .ok_or_else(|| anyhow!("Missing buy record for {}", mint))?;
            let ata_str = info
                .ata()
                .ok_or_else(|| anyhow!("Missing ATA for {}", mint))?;
            let tracked = info.amount();
            let fallback = info.fallback_amount();
            let sell_amount = tracked.max(fallback);
            let vault = *info
                .creator_vault()
                .ok_or_else(|| anyhow!("Missing stored creator_vault for {}", mint))?;
            (Pubkey::from_str(ata_str)?, sell_amount, vault)
        };

        // build instruction data
        const PUMP_SELL_DISCRIMINATOR: [u8; 8] = [51, 230, 133, 164, 1, 127, 131, 173];
        let mut data = PUMP_SELL_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&to_vec(&PumpSellInstruction {
            amount: sell_amount,
            min_sol_output: 1_000,
        })?);

        // assemble accounts in IDL order
        let accounts = vec![
            AccountMeta::new_readonly(global_pda, false),
            AccountMeta::new(fee_recipient, false),
            AccountMeta::new_readonly(mint_pubkey, false),
            AccountMeta::new(bonding_curve, false),
            AccountMeta::new(associated_bonding_curve, false),
            AccountMeta::new(in_ata, false),
            AccountMeta::new(owner, true),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new(creator_vault, false),
            AccountMeta::new_readonly(Pubkey::from_str(TOKEN_PROGRAM)?, false),
            AccountMeta::new_readonly(event_authority, false),
            AccountMeta::new_readonly(pump_program, false),
        ];

        info!(
            "⛓ Preparing SELL for {}: amount={}, creator_vault={}; accounts={:?}",
            mint_pubkey,
            sell_amount,
            creator_vault,
            accounts
                .iter()
                .map(|m| m.pubkey.to_string())
                .collect::<Vec<_>>()
        );

        let ix = Instruction {
            program_id: pump_program,
            accounts,
            data,
        };
        let mut last: Result<Vec<String>> = Err(anyhow!("no attempts"));
        for attempt in 1..=3 {
            let res = tx::new_signed_and_send(
                self.rpc_client.as_ref().unwrap(),
                &self.keypair,
                vec![ix.clone()],
                swap_config.use_jito,
                swap_config.use_priority_tip,
                &logger,
            )
            .await;

            match res {
                Ok(sigs) => {
                    let sell_slot = CURRENT_SLOT.load(Ordering::Relaxed);
                    info!(
                        "💰 SELL succeeded for {}: sigs={:?} @ slot={}",
                        mint_pubkey, sigs, sell_slot
                    );

                    // ** record the signature so your gRPC‐stream confirmation can see it **
                    let mut tokens = BOUGHT_TOKENS.lock().await;
                    if let Some(entry) = tokens.get_mut(&mint_pubkey.to_string()) {
                        entry.set_sell_signature(sigs[0].clone());
                    }

                    last = Ok(sigs);
                    break;
                }

                Err(err) => {
                    info!(
                        "❌ SELL attempt {} failed for {}: {}",
                        attempt, mint_pubkey, err
                    );
                    if attempt < 3 {
                        tokio::time::sleep(Duration::from_millis(600 * attempt)).await;
                    }
                    last = Err(err);
                }
            }
        }

        last
    }
}

pub async fn wait_for_bonding_curve_account(
    rpc_client: Arc<solana_client::rpc_client::RpcClient>,
    mint: &Pubkey,
    program_id: &Pubkey,
    max_retries: usize,
    delay_ms: u64,
) -> Result<(Pubkey, Pubkey, BondingCurveAccount)> {
    let bonding_curve = get_pda(mint, program_id)?;
    let associated_bonding_curve = get_associated_token_address(&bonding_curve, mint);

    for attempt in 0..max_retries {
        match rpc_client.get_account_data(&bonding_curve) {
            Ok(data) => {
                let bonding_curve_account: BondingCurveAccount =
                    from_slice::<BondingCurveAccount>(&data).map_err(|e| {
                        anyhow!("Failed to deserialize bonding curve account: {}", e)
                    })?;
                return Ok((
                    bonding_curve,
                    associated_bonding_curve,
                    bonding_curve_account,
                ));
            }
            Err(err) => {
                if attempt == max_retries - 1 {
                    return Err(anyhow!(
                        "Bonding curve account not found after retries: {}",
                        err
                    ));
                }
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }
        }
    }

    Err(anyhow!(
        "Bonding curve account retry logic failed unexpectedly"
    ))
}

/// Derives the bonding curve PDA and its associated token account for a given mint.
pub async fn get_bonding_curve_account(
    rpc_client: Arc<solana_client::rpc_client::RpcClient>,
    mint: &Pubkey,
    program_id: &Pubkey,
) -> Result<(Pubkey, Pubkey, BondingCurveAccount)> {
    let bonding_curve = get_pda(mint, program_id)?;
    let associated_bonding_curve =
        spl_associated_token_account::get_associated_token_address(&bonding_curve, mint);
    let bonding_curve_data = rpc_client
        .get_account_data(&bonding_curve)
        .inspect_err(|err| {
            println!(
                "Failed to get bonding curve account data: {}, err: {}",
                bonding_curve, err
            );
        })?;

    let bonding_curve_account =
        from_slice::<BondingCurveAccount>(&bonding_curve_data).map_err(|e| {
            anyhow!(
                "Failed to deserialize bonding curve account: {}",
                e.to_string()
            )
        })?;

    Ok((
        bonding_curve,
        associated_bonding_curve,
        bonding_curve_account,
    ))
}

/// Derives the PDA for the bonding curve from the mint and program id.
pub fn get_pda(mint: &Pubkey, program_id: &Pubkey) -> Result<Pubkey> {
    let seeds = [b"bonding-curve".as_ref(), mint.as_ref()];
    let (bonding_curve, _bump) = Pubkey::find_program_address(&seeds, program_id);
    Ok(bonding_curve)
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct BondingCurveAccount {
    pub discriminator: u64,
    pub virtual_token_reserves: u64,
    pub virtual_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub token_total_supply: u64,
    pub complete: bool,
    pub creator: Pubkey,
}

/// Retrieves pump info for a given token mint.
pub async fn get_pump_info(
    rpc_client: Arc<solana_client::rpc_client::RpcClient>,
    mint: &str,
) -> Result<PumpInfo> {
    let mint = Pubkey::from_str(mint)?;
    let program_id = Pubkey::from_str(PUMP_PROGRAM)?;
    let (bonding_curve, associated_bonding_curve, bonding_curve_account) =
        get_bonding_curve_account(rpc_client, &mint, &program_id).await?;

    let pump_info = PumpInfo {
        mint: mint.to_string(),
        bonding_curve: bonding_curve.to_string(),
        associated_bonding_curve: associated_bonding_curve.to_string(),
        raydium_pool: None,
        raydium_info: None,
        complete: bonding_curve_account.complete,
        virtual_sol_reserves: bonding_curve_account.virtual_sol_reserves,
        virtual_token_reserves: bonding_curve_account.virtual_token_reserves,
        total_supply: bonding_curve_account.token_total_supply,
    };
    Ok(pump_info)
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RaydiumInfo {
    pub base: f64,
    pub quote: f64,
    pub price: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PumpInfo {
    pub mint: String,
    pub bonding_curve: String,
    pub associated_bonding_curve: String,
    pub raydium_pool: Option<String>,
    pub raydium_info: Option<RaydiumInfo>,
    pub complete: bool,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub total_supply: u64,
}
