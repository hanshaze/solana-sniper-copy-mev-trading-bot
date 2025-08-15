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

pub struct Pump {
    pub rpc_nonblocking_client: Arc<solana_client::nonblocking::rpc_client::RpcClient>,
    pub keypair: Arc<Keypair>,
    pub rpc_client: Option<Arc<solana_client::rpc_client::RpcClient>>,
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
        }
    }

    pub async fn swap(&self, mint: &str, swap_config: SwapConfig) -> Result<Vec<String>> {
        let logger = Logger::new("[SWAP IN PUMP.FUN] => ".to_string());
        let slippage_bps = swap_config.slippage * 100;
        let owner = self.keypair.pubkey();
        let mint =
            Pubkey::from_str(mint).map_err(|e| anyhow!("failed to parse mint pubkey: {}", e))?;
        let program_id = spl_token::ID;
        let native_mint = spl_token::native_mint::ID;

        let (token_in, token_out, pump_method) = match swap_config.swap_direction {
            SwapDirection::Buy => (native_mint, mint, PUMP_BUY_METHOD),
            SwapDirection::Sell => (mint, native_mint, PUMP_SELL_METHOD),
        };
        let pump_program = Pubkey::from_str(PUMP_PROGRAM)?;
        let (bonding_curve, associated_bonding_curve, bonding_curve_account) =
            get_bonding_curve_account(self.rpc_client.clone().unwrap(), &mint, &pump_program)
                .await?;

        let in_ata = token::get_associated_token_address(
            self.rpc_nonblocking_client.clone(),
            self.keypair.clone(),
            &token_in,
            &owner,
        );
        let out_ata = token::get_associated_token_address(
            self.rpc_nonblocking_client.clone(),
            self.keypair.clone(),
            &token_out,
            &owner,
        );

        let mut create_instruction = None;
        let mut close_instruction = None;

        tx::new_signed_and_send(
            &client,
            &self.keypair,
            instructions,
            swap_config.use_jito,
            &logger,
        )
        .await
    }
}
impl Raydium {
    pub fn new(
        rpc_nonblocking_client: Arc<solana_client::nonblocking::rpc_client::RpcClient>,
        rpc_client: Arc<solana_client::rpc_client::RpcClient>,
        keypair: Arc<Keypair>,
    ) -> Self {
        Self {
            rpc_nonblocking_client,
            keypair,
            rpc_client: Some(rpc_client),
            pool_id: None,
        }
    }

    pub async fn swap(
        &self,
        swap_config: SwapConfig,
        amm_pool_id: Pubkey,
        pool_state: AmmInfo,
    ) -> Result<Vec<String>> {
        let logger = Logger::new(format!(
            "[SWAP IN RAYDIUM]({}) => ",
            chrono::Utc::now().timestamp()
        ));
        let slippage_bps = swap_config.slippage * 100;
        let owner = self.keypair.pubkey();
        let program_id = spl_token::ID;
        let native_mint = spl_token::native_mint::ID;
        let mint = pool_state.coin_vault_mint;

        let (token_in, token_out, user_input_token, swap_base_in) = match (
            swap_config.swap_direction.clone(),
            pool_state.coin_vault_mint == native_mint,
        ) {
            (SwapDirection::Buy, true) => (native_mint, mint, pool_state.coin_vault, true),
            (SwapDirection::Buy, false) => (native_mint, mint, pool_state.pc_vault, true),
            (SwapDirection::Sell, true) => (mint, native_mint, pool_state.pc_vault, true),
            (SwapDirection::Sell, false) => (mint, native_mint, pool_state.coin_vault, true),
        };

        logger.log(format!(
            "token_in:{}, token_out:{}, user_input_token:{}, swap_base_in:{}",
            token_in, token_out, user_input_token, swap_base_in
        ));

        let in_ata = get_associated_token_address(
            self.rpc_nonblocking_client.clone(),
            self.keypair.clone(),
            &token_in,
            &owner,
        );
        let out_ata = get_associated_token_address(
            self.rpc_nonblocking_client.clone(),
            self.keypair.clone(),
            &token_out,
            &owner,
        );

        let mut create_instruction = None;
        let mut close_instruction = None;

        tx::new_signed_and_send(
            &client,
            &self.keypair,
            instructions,
            swap_config.use_jito,
            &logger,
        )
        .await
    }
}