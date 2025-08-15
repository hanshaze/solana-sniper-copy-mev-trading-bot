// swap.rs
use crate::common::utils::{AppState, SwapConfig, SwapInput};
use crate::dex::pump_fun::Pump;
use anyhow::Result;
use clap::ValueEnum;
use serde::Deserialize;

#[derive(ValueEnum, Copy, Debug, Clone, Deserialize, PartialEq)]
pub enum SwapDirection {
    #[serde(rename = "buy")]
    Buy,
    #[serde(rename = "sell")]
    Sell,
}
impl From<SwapDirection> for u8 {
    fn from(value: SwapDirection) -> Self {
        match value {
            SwapDirection::Buy => 0,
            SwapDirection::Sell => 1,
        }
    }
}

#[derive(ValueEnum, Debug, Clone, Deserialize)]
pub enum SwapInType {
    #[serde(rename = "qty")]
    Qty,
    #[serde(rename = "pct")]
    Pct,
}

pub async fn pump_swap(
    state: AppState,
    swap_input: SwapInput,
    swap_direction: &str,
    use_jito: bool,
    urgent: bool,
) -> Result<Vec<String>> {
    if swap_input.amount == 0 {
        return Err(anyhow::anyhow!(
            "Skipping swap: provided amount is 0 lamports"
        ));
    }

    // Convert the provided swap_direction string to our enum.
    let swap_direction_enum = match swap_direction {
        "buy" => SwapDirection::Buy,
        "sell" => SwapDirection::Sell,
        _ => anyhow::bail!("Invalid swap direction"),
    };

    // Default to the provided slippage...
    let mut slippage = swap_input.slippage_bps as u64;

    // 🔧 Add more slippage for sells to prevent 6003 errors
    if swap_direction_enum == SwapDirection::Sell {
        slippage += 50; // Add 10bps extra
    }

    let swap_config = SwapConfig {
        slippage,
        swap_direction: swap_direction_enum,
        use_jito,
        use_priority_tip: urgent,
        cu_limits: None,
        priority_fee: None,
        wrap_and_unwrap_sol: Some(true),
        as_legacy_transaction: Some(false),
    };

    let swapx = Pump::new(state.rpc_nonblocking_client, state.rpc_client, state.wallet);

    let mint = if swap_direction_enum == SwapDirection::Buy {
        swap_input.output_token_mint.to_string()
    } else {
        swap_input.input_token_mint.to_string()
    };

    let amount_in = swap_input.amount;

    let res = swapx
        .swap(&mint, swap_config, swap_input, amount_in)
        .await?;
    Ok(res)
}
