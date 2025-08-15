#[derive(Debug, Clone, Copy)]
pub enum DetectionStrategy {
    TokenCreation,
    FirstBuy,
    SuspectedCreatorSell,
}


pub async fn handle_detection_event(
    txn: &ConfirmedTransactionWithStatusMeta,
    slot: u64,
    strategy: DetectionStrategy,
    mint_override: Option<String>, // Only used for creation strategy
) {
    match strategy {
        DetectionStrategy::TokenCreation => {
            if let Some(mint) = mint_override {
                let creation_time = current_unix_timestamp();
                let mut creations = TOKEN_CREATIONS.lock().await;
                creations.insert(mint.clone(), TokenCreationInfo {
                    mint,
                    creation_time,
                });
            }
        }

        DetectionStrategy::FirstBuy => {
            if TOKEN_CREATIONS.lock().await.is_empty() {
                return;
            }

            if let TransactionWithStatusMeta::Complete(versioned_tx_with_meta) = &txn.tx_with_meta {
                if let (Some(pre_token_balances), Some(post_token_balances)) =
                    (&versioned_tx_with_meta.meta.pre_token_balances, &versioned_tx_with_meta.meta.post_token_balances)
                {
                    for (i, post_tb) in post_token_balances.iter().enumerate() {
                        if let Some(post_ui) = post_tb.ui_token_amount.ui_amount {
                            if i >= pre_token_balances.len() { continue; }

                            let pre_tb = &pre_token_balances[i];
                            if let Some(pre_ui) = pre_tb.ui_token_amount.ui_amount {
                                let delta = post_ui - pre_ui;
                                if delta >= 0.0 { continue; }

                                let buy_amount = -delta;
                                let mint = pre_tb.mint.clone();
                                let buyer = pre_tb.owner.clone();

                                const DEFAULT_TOTAL_SUPPLY: f64 = SUPPLY_THRESHOLD_UI;
                                const MIN_BUY_FRACTION: f64 = 0.000001;
                                const MAX_BUY_FRACTION: f64 = 0.10;

                                if pre_ui < DEFAULT_TOTAL_SUPPLY * 0.98 { continue; }
                                if buy_amount < DEFAULT_TOTAL_SUPPLY * MIN_BUY_FRACTION
                                    || buy_amount > DEFAULT_TOTAL_SUPPLY * MAX_BUY_FRACTION
                                {
                                    continue;
                                }

                                let mut creations = TOKEN_CREATIONS.lock().await;
                                if !creations.contains_key(&mint) {
                                    println!("[EVENT] No creation event for {}; skipping first buy.", mint);
                                    continue;
                                }

                                creations.remove(&mint);
                                drop(creations);

                                let mut map = TRACKED_TOKENS.lock().await;
                                if let Some(entry) = map.get_mut(&mint) {
                                    if entry.suspected_creators.insert(buyer.clone()) {
                                        entry.sell_split_ratio = entry.suspected_creators.len();
                                        println!(
                                            "[FOLLOW-UP CREATOR] Mint: {} | New suspected creator: {} | Total creators: {}",
                                            mint, buyer, entry.sell_split_ratio
                                        );
                                    }
                                } else {
                                    let mut suspected = HashSet::new();
                                    suspected.insert(buyer.clone());

                                    map.insert(mint.clone(), TokenTrackingState {
                                        mint: mint.clone(),
                                        first_buyer: Some(buyer.clone()),
                                        first_buy_amount: buy_amount,
                                        suspected_creators: suspected,
                                        sell_detected: false,
                                        sell_split_ratio: 0,
                                        has_bought: false,
                                        bought_amount: None,
                                        has_sold: false,
                                        open_trade_permit: None,
                                        detected_buy_slot: Some(slot),
                                    });

                                    println!("[NEW TOKEN TRACKED] Mint: {} | First Buyer: {} | Buy Amount: {} | Slot: {}",
                                        mint, buyer, buy_amount, slot);
                                    log::info!("New token tracked: {} bought by {}", mint, buyer);

                                    let notify = get_notify_handle().await;
                                    notify.notify_one();
                                }
                            }
                        }
                    }
                }
            }
        }

        DetectionStrategy::SuspectedCreatorSell => {
            if let TransactionWithStatusMeta::Complete(versioned_tx_with_meta) = &txn.tx_with_meta {
                if let (Some(pre_token_balances), Some(post_token_balances)) =
                    (&versioned_tx_with_meta.meta.pre_token_balances, &versioned_tx_with_meta.meta.post_token_balances)
                {
                    for (i, post_tb) in post_token_balances.iter().enumerate() {
                        if let Some(post_ui) = post_tb.ui_token_amount.ui_amount {
                            if i < pre_token_balances.len() {
                                let pre_tb = &pre_token_balances[i];
                                if let Some(pre_ui) = pre_tb.ui_token_amount.ui_amount {
                                    let delta = post_ui - pre_ui;
                                    if delta >= 0.0 { continue; }

                                    let mint = pre_tb.mint.clone();
                                    let seller = pre_tb.owner.clone();

                                    let mut map = TRACKED_TOKENS.lock().await;
                                    if let Some(entry) = map.get_mut(&mint) {
                                        if !entry.has_bought || entry.bought_amount.unwrap_or(0) == 0 {
                                            log::info!(
                                                "[SELL IGNORED] Mint {}: Suspected creator {} sold but we haven't bought yet",
                                                mint, seller
                                            );
                                            return;
                                        }

                                        if entry.suspected_creators.contains(&seller) && !entry.sell_detected {
                                            entry.sell_detected = true;
                                            entry.sell_split_ratio = entry.suspected_creators.len();

                                            log::info!(
                                                "[SELL DETECTED] Mint {}: Seller {} triggered sell | Total suspected creators: {}",
                                                mint, seller, entry.sell_split_ratio
                                            );

                                            let notify = get_notify_handle().await;
                                            notify.notify_one();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
