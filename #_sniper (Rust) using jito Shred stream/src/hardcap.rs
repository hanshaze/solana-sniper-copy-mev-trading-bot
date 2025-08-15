pub async fn handle_buy_and_sell_logic(
    txn: &ConfirmedTransactionWithStatusMeta,
    sell_sender: &Sender<(String, u64)>,
) {
    let logger = Logger::new("[BUY/SELL LOGIC] => ".to_string());
    let our_pubkey = env::var("WALLET_PUBKEY").unwrap_or_default();
    let min_inflow: f64 = env::var("MIN_SOL_INFLOW_TRIGGER").unwrap_or("0.5".to_string()).parse().unwrap_or(0.5);
    let min_outflow: f64 = env::var("MIN_SOL_OUTFLOW_TRIGGER").unwrap_or("0.5".to_string()).parse().unwrap_or(0.5);

    // Extract our on-chain slot for comparisons
    let current_slot = CURRENT_SLOT.load(std::sync::atomic::Ordering::Relaxed);

    let TransactionWithStatusMeta::Complete(vtx) = &txn.tx_with_meta else { return };
    let Some(post_balances) = &vtx.meta.post_token_balances else { return };
    let Some(pre_balances) = &vtx.meta.pre_token_balances else { return };

    let mut tokens = BOUGHT_TOKENS.lock().await;

    for (i, post_tb) in post_balances.iter().enumerate() {
        if i >= pre_balances.len() { continue; }

        let pre_tb = &pre_balances[i];
        let mint = &post_tb.mint;
        let owner = post_tb.owner.clone();
        let ata = post_tb.account_index.to_string();

        let pre_amount = pre_tb.ui_token_amount.ui_amount.unwrap_or(0.0);
        let post_amount = post_tb.ui_token_amount.ui_amount.unwrap_or(0.0);
        let delta = post_amount - pre_amount;

        if let Some(entry) = tokens.get_mut(mint) {
            if entry.buy_slot() == 0 {
                continue;
            }
            if entry.sell_triggered_at().is_some() {
                continue;
            }

            let is_from_our_ata = Some(ata.clone()) == entry.ata().cloned();
            let is_owner_ours = owner == our_pubkey;


            // Time to slot delta instead of wall-clock
            let slot_delta = current_slot.saturating_sub(entry.buy_slot());

            if delta > 0.0 && !is_owner_ours && !is_from_our_ata {
                if entry.add_unique_buyer(owner.clone()) {
                    logger.log(format!(
                        "🧍 Unique buyer for {}: {} | Total: {}",
                        mint,
                        owner,
                        entry.unique_buyers().len()
                    ));

                    if entry.unique_buyers().len() == 1 && entry.suspected_creator().is_none() {
                        entry.set_suspected_creator(owner.clone());
                        info!("{}", format!("🧠 Suspected creator flagged: {} on {}", owner, mint));
                    }
                }

                // Early rug detection: <2 buyers after X slots
                if entry.unique_buyers().len() < 10 && slot_delta > 12 {
                    entry.set_sell_triggered(true);
                    entry.set_sell_triggered_at(Instant::now());
                    info!("{}", format!("🛑 Rug suspected — Forcing sell on {} after {} slots", mint, slot_delta));
                    if sell_sender.send((mint.clone(), entry.amount())).await.is_ok() {
                        info!("{}", format!("📤 Sell sent for {}", mint));
                    }
                    continue;
                }

                // Exhausted follow-ups
                if entry.unique_buyers().len() >= 5 {
                    entry.set_sell_triggered(true);
                    entry.set_sell_triggered_at(Instant::now());
                    info!("{}", format!("📉 Follow-up exhaustion — Selling {} after 3 buyers", mint));
                    if sell_sender.send((mint.clone(), entry.amount())).await.is_ok() {
                        info!("{}", format!("📤 Sell sent for {}", mint));
                    }
                    continue;
                }

            }

            // SOL inflow/outflow logic (unchanged)
            if let Ok(pid) = Pubkey::from_str(PUMP_PROGRAM) {
                if let Ok(bonding_curve) = get_pda(&Pubkey::from_str(mint).unwrap(), &pid) {
                    let message = &vtx.transaction.message;
                    let mut keys = match message {
                        VersionedMessage::Legacy(m) => m.account_keys.clone(),
                        VersionedMessage::V0(v0) => {
                            let mut k = v0.account_keys.clone();
                            k.extend(vtx.meta.loaded_addresses.writable.clone());
                            k.extend(vtx.meta.loaded_addresses.readonly.clone());
                            k
                        }
                    };
                    if let Some(idx) = keys.iter().position(|k| k == &bonding_curve) {
                        let pre_sol = vtx.meta.pre_balances[idx];
                        let post_sol = vtx.meta.post_balances[idx];
                        let delta_sol = (post_sol as i64 - pre_sol as i64) as f64 / 1e9;
                        logger.log(format!(
                            "🔍 Mint: {} | ΔSOL: {:.6} | Slot: {}", mint, delta_sol, txn.slot
                        ));

                        if delta_sol > 0.0 {
                            entry.add_sol_inflow(delta_sol);
                        } else if delta_sol < 0.0 {
                            entry.add_sol_outflow(-delta_sol);
                            if slot_delta <= 1 {
                                entry.set_sell_triggered(true);
                                entry.set_sell_triggered_at(Instant::now());
                                info!("{}", format!("💣 Early SOL drain! Fast sell on {}", mint));
                                if sell_sender.send((mint.clone(), entry.amount())).await.is_ok() {
                                    info!("{}", format!("📤 Sell sent for {}", mint));
                                }
                                continue;
                            }
                        }

                        if delta_sol < -min_outflow {
                            entry.set_sell_triggered(true);
                            entry.set_sell_triggered_at(Instant::now());
                            info!("{}", format!("💣 Outflow sell on {} | ΔSOL: {:.6}", mint, delta_sol));
                            if sell_sender.send((mint.clone(), entry.amount())).await.is_ok() {
                                info!("{}", format!("📤 Sell sent for {}", mint));
                            }
                            continue;
                        }

                        if entry.sol_inflow() >= min_inflow {
                            entry.set_sell_triggered(true);
                            entry.set_sell_triggered_at(Instant::now());
                            info!("{}", format!("🚀 Inflow sell on {} | Total {:.6} SOL", mint, entry.sol_inflow()));
                            let mint_c = mint.clone(); let amt = entry.amount(); let tx_s = sell_sender.clone();
                            let target_slot = current_slot + 1;
                            tokio::spawn(async move {
                                while CURRENT_SLOT.load(Ordering::Relaxed) < target_slot {
                                    tokio::time::sleep(Duration::from_millis(10)).await;
                                }
                                if tx_s.send((mint_c.clone(), amt)).await.is_ok() {
                                    info!("📤 Delayed sell sent for {}", mint_c);
                                }
                            });
                        }
                    }
                }
            }
        }
    }
}
