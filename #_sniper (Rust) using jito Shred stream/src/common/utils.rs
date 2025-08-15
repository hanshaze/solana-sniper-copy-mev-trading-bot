// utils.rs
use crate::engine::swap::SwapDirection;
use anyhow::Result;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Keypair};
use std::{env, sync::Arc};

#[derive(Clone)]
pub struct AppState {
    pub rpc_client: Arc<solana_client::rpc_client::RpcClient>,
    pub rpc_nonblocking_client: Arc<solana_client::nonblocking::rpc_client::RpcClient>,
    pub wallet: Arc<Keypair>,
}

pub struct ParseTx {
    pub type_tx: String,
    pub direction: Option<String>,
    pub amount_in: f64,
    pub amount_out: f64,
    pub mint: String,
}

pub fn import_env_var(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("Environment variable {} is not set", key))
}

pub fn create_rpc_client() -> Result<Arc<solana_client::rpc_client::RpcClient>> {
    let rpc_https = import_env_var("RPC_ENDPOINT");
    let rpc_client = solana_client::rpc_client::RpcClient::new_with_commitment(
        rpc_https,
        CommitmentConfig::processed(),
    );
    Ok(Arc::new(rpc_client))
}

pub async fn create_nonblocking_rpc_client(
) -> Result<Arc<solana_client::nonblocking::rpc_client::RpcClient>> {
    let rpc_https = import_env_var("RPC_ENDPOINT");
    let rpc_client = solana_client::nonblocking::rpc_client::RpcClient::new_with_commitment(
        rpc_https,
        CommitmentConfig::processed(),
    );
    Ok(Arc::new(rpc_client))
}

pub fn import_wallet() -> Result<Arc<Keypair>> {
    let priv_key = import_env_var("PRIVATE_KEY");
    let wallet: Keypair = Keypair::from_base58_string(priv_key.as_str());

    Ok(Arc::new(wallet))
}

#[derive(Copy, Clone, Debug, Default)]
pub enum ComputeUnitLimits {
    #[default]
    Dynamic,
    Fixed(u64),
}

#[derive(Copy, Clone, Debug)]
pub enum PriorityFeeConfig {
    DynamicMultiplier(u64),
    FixedCuPrice(u64),
    JitoTip(u64),
}

#[derive(Clone, Debug)]
pub struct SwapConfig {
    pub priority_fee: Option<PriorityFeeConfig>,
    pub cu_limits: Option<ComputeUnitLimits>,
    pub wrap_and_unwrap_sol: Option<bool>,
    pub as_legacy_transaction: Option<bool>,
    pub slippage: u64,
    pub swap_direction: SwapDirection,
    pub use_jito: bool,
    pub use_priority_tip: bool,
}

#[derive(Clone, Debug, Default)]
pub struct SwapConfigOverrides {
    pub priority_fee: Option<PriorityFeeConfig>,
    pub cu_limits: Option<ComputeUnitLimits>,
    pub wrap_and_unwrap_sol: Option<bool>,
    pub destination_token_account: Option<Pubkey>,
    pub as_legacy_transaction: Option<bool>,
}

#[derive(Copy, Clone, Debug)]
pub struct SwapInput {
    pub input_token_mint: Pubkey,
    pub output_token_mint: Pubkey,
    pub slippage_bps: u16,
    pub amount: u64,
    pub mode: SwapExecutionMode,
    pub market: Option<Pubkey>,
    pub creator_vault: Option<Pubkey>,
}

#[derive(Copy, Clone, Debug)]
pub enum SwapExecutionMode {
    ExactIn,
    ExactOut,
}
impl SwapExecutionMode {
    pub fn amount_specified_is_input(&self) -> bool {
        matches!(self, SwapExecutionMode::ExactIn)
    }
}
