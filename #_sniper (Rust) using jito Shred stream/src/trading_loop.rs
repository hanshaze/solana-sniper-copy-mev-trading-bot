// trading_loop.rs

use crate::common::utils::{
    create_nonblocking_rpc_client, create_rpc_client, import_env_var, import_wallet, AppState,
    SwapExecutionMode, SwapInput,
};
use crate::core::tx;
use crate::dex::pump_fun::PUMP_PROGRAM;
use crate::engine::swap::pump_swap;
use dotenv::dotenv;
use shared_state::{BuyOrder, SellOrder};
use solana_sdk::{native_token, pubkey::Pubkey};
use std::{
    env,
    str::FromStr,
    sync::{Arc, OnceLock},
};
use tokio::sync::{mpsc::Receiver, Notify, OnceCell};

static TRIGGER_NOTIFY: OnceCell<Arc<Notify>> = OnceCell::const_new();

/// Exposed so `pub use trading_loop::get_notify_handle;` still works.
pub async fn get_notify_handle() -> Arc<Notify> {
    TRIGGER_NOTIFY
        .get_or_init(|| async { Arc::new(Notify::new()) })
        .await
        .clone()
}

// Pre-parse these exactly once at program load:
static PUMP_PROGRAM_KEY: OnceLock<Pubkey> = OnceLock::new();
static LAMPORTS_PER_SOL_U64: OnceLock<u64> = OnceLock::new();

pub async fn start_trading_loop(mut mint_rx: Receiver<BuyOrder>, mut sell_rx: Receiver<SellOrder>) {
    // load .env if present
    dotenv().ok();

    // one-time RPC & wallet setup
    let rpc = create_rpc_client().unwrap();
    let rpc_nonblocking = create_nonblocking_rpc_client().await.unwrap();
    let wallet = import_wallet().unwrap();

    let app = AppState {
        rpc_client: rpc.clone(),
        rpc_nonblocking_client: rpc_nonblocking.clone(),
        wallet: wallet.clone(),
    };

    // ─── SPAWN BLOCKHASH REFRESHER ───
    // This will keep a fresh recent‐blockhash in the background so sends never stall
    // ─── ONE-TIME TIP‐ENGINE INITIALIZATION ───
    // Read your env vars just once
    let block_engine = env::var("BLOCK_ENGINE_PROVIDER").unwrap_or_default();
    let use_nextblock = block_engine.eq_ignore_ascii_case("nextblock");
    let use_priority_tip = env::var("USE_PRIORITY_TIP")
        .unwrap_or_default()
        .eq_ignore_ascii_case("true");
    // This will create/fetch your tip‐accounts and pull the first tip value
    tx::init_tip_state(use_nextblock, use_priority_tip)
        .await
        .expect("failed to init tip engine");
    // stash env vars once
    let slippage_bps = import_env_var("SLIPPAGE").parse::<u16>().unwrap_or(15);
    let buy_amount_sol = import_env_var("BUY_AMOUNT_SOL")
        .parse::<f64>()
        .unwrap_or(0.01);

    // initialize our OnceLocks
    let lamports_per_sol = *LAMPORTS_PER_SOL_U64.get_or_init(|| native_token::LAMPORTS_PER_SOL);
    let buy_amount_lamports = (buy_amount_sol * lamports_per_sol as f64) as u64;
    let pump_program_key =
        *PUMP_PROGRAM_KEY.get_or_init(|| Pubkey::from_str(PUMP_PROGRAM).unwrap());

    loop {
        tokio::select! {
            // ───────── BUY ORDERS ─────────
            Some(BuyOrder { mint, creator, use_jito, urgent }) = mint_rx.recv() => {
                let mint_pk = Pubkey::from_str(&mint).unwrap();
                // derive creator vault address
                let (creator_vault, _) =
                    Pubkey::find_program_address(&[b"creator-vault", creator.as_ref()], &pump_program_key);

                let input = SwapInput {
                    input_token_mint:  spl_token::native_mint::ID,
                    output_token_mint: mint_pk,
                    slippage_bps,
                    amount:            buy_amount_lamports,
                    mode:              SwapExecutionMode::ExactIn,
                    market:            None,
                    creator_vault:     Some(creator_vault),
                };

                let app_clone = app.clone();
                tokio::spawn(async move {
                    let _ = pump_swap(app_clone, input, "buy", use_jito, urgent).await;
                });
            }

            // ───────── SELL ORDERS ────────
            Some(SellOrder { mint, amount, use_jito, urgent }) = sell_rx.recv() => {
                let mint_pk = Pubkey::from_str(&mint).unwrap();

                let input = SwapInput {
                    input_token_mint:  mint_pk,
                    output_token_mint: spl_token::native_mint::ID,
                    slippage_bps,
                    amount,
                    mode:              SwapExecutionMode::ExactOut,
                    market:            None,
                    creator_vault:     None,
                };

                let app_clone = app.clone();
                tokio::spawn(async move {
                    let _ = pump_swap(app_clone, input, "sell", use_jito, urgent).await;
                });
            }
        }
    }
}
