struct TradeDispatcher {
    db:   Database,
    bot:  AutoSend<Bot>,
    user_ids:          Vec<i64>,      // all whitelisted telegram IDs
    next_idx:          AtomicUsize,   // for round-robin
}

impl TradeDispatcher {
    pub async fn new(db: Database, bot: AutoSend<Bot>) -> Self {
        let ids = db
            .users
            .find(doc! { "is_whitelisted": true }, None)
            .await
            .unwrap()
            .try_collect::<Vec<UserRecord>>()
            .await
            .unwrap()
            .into_iter()
            .map(|u| u.telegram_id)
            .collect();
        TradeDispatcher { db, bot, user_ids: ids, next_idx: AtomicUsize::new(0) }
    }

    /// Round-robin pick your next telegram user
    fn pop_next_user(&self) -> i64 {
        let len = self.user_ids.len();
        let i = self.next_idx.fetch_add(1, Ordering::Relaxed) % len;
        self.user_ids[i]
    }

    pub async fn dispatch_buy(&self, buy: BuyOrder) {
        // 1) pick user
        let telegram_id = self.pop_next_user();
        let user = self.db.get_user(telegram_id).await.unwrap()
            .expect("should exist and be whitelisted");

        // 2) decrypt & load their Keypair
        let keypair = decrypt_keypair(&user
            .wallets
            .iter()
            .find(|w| w.id == user.trading_wallet_id.unwrap())
            .unwrap()
            .private_key_enc
        );

        // 3) build per-user SwapConfig
        let mut cfg = SwapConfig::default();
        if let Some(sl) = user.slippage_pct      { cfg.slippage_bps = (sl * 100.0) as u16 }
        if let Some(tip) = user.tip_amount       { cfg.use_priority_tip = tip > 0 }
        if let Some(min_tok) = user.min_tokens_to_buy { cfg.min_tokens = Some(min_tok as u64) }

        // 4) hand off to pump_swap
        let input = /* build your SwapInput exactly as before */;
        let result = pump_swap(
            AppState {
                rpc_client:            /* shared */,
                rpc_nonblocking_client: /* shared */,
                wallet: Arc::new(keypair),
            },
            input,
            "buy",
            buy.use_jito,
            buy.urgent,
        )
        .await;

        // 5) notify on Telegram
        let msg = match result {
            Ok(sigs) => format!("✅ [{}] buy for `{}` succeeded: {}", telegram_id, buy.mint, sigs[0]),
            Err(e)   => format!("❌ [{}] buy for `{}` failed: {}", telegram_id, buy.mint, e),
        };
        let _ = self.bot.send_message(telegram_id, msg).await;
    }

    pub async fn dispatch_sell(&self, sell: SellOrder) {
        // Look up who originally bought this mint
        let user_id = {
            let tokens = shared_state::BOUGHT_TOKENS.lock().await;
            let info   = tokens.get(&sell.mint).unwrap();
            info.telegram_id().unwrap()  
        };

        // same as buy: fetch user, decrypt, build cfg, call pump_swap("sell", ...) and notify
        /* … */
    }
}
