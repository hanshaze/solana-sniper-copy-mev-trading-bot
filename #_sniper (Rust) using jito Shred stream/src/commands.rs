// src/commands.rs
use teloxide::prelude::*;
use teloxide::utils::command::BotCommand;
use mongodb::bson::oid::ObjectId;
use crate::db::Database;
use anyhow::Result;

#[derive(BotCommand, Clone)]
#[command(rename = "lowercase", description = "These commands are supported:")]
pub enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "start the bot.")]
    Start,
    #[command(description = "create a new wallet: /create_wallet <name> <private_key_or_mnemonic>")]
    CreateWallet { name: String, key: String },
    #[command(description = "list your wallets.")]
    ListWallets,
    #[command(description = "remove a wallet: /remove_wallet <wallet_id>")]
    RemoveWallet { wallet_id: String },
    #[command(description = "set trading wallet: /set_trading_wallet <wallet_id>")]
    SetTradingWallet { wallet_id: String },
    #[command(description = "set min tokens to buy: /set_min_tokens <amount>")]
    SetMinTokens { amount: f64 },
    #[command(description = "set slippage percent: /set_slippage <percent>")]
    SetSlippage { percent: f64 },
    #[command(description = "set tip amount: /set_tip <amount>")]
    SetTip { amount: u64 },
    #[command(description = "withdraw: /withdraw <wallet_id> <dest_address> <amount>")]
    Withdraw { wallet_id: String, dest: String, amount: f64 },
    #[command(description = "transfer: /transfer <wallet_id> <dest_address> <amount>")]
    Transfer { wallet_id: String, dest: String, amount: f64 },
    // Admin only:
    #[command(description = "whitelist a user: /whitelist <telegram_id>")]
    Whitelist { telegram_id: i64 },
    #[command(description = "remove from whitelist: /unwhitelist <telegram_id>")]
    Unwhitelist { telegram_id: i64 },
}

pub async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    db: Database,
    admin_ids: Vec<i64>,
) -> Result<()> {
    let chat_id = msg.chat.id;
    let user_id = msg.from().map(|u| u.id.0 as i64).unwrap_or(0);

    match cmd {
        Command::Help => {
            bot.send_message(chat_id, Command::descriptions()).await?;
        }
        Command::Start => {
            // Check whitelist
            let is_whitelisted = db.is_whitelisted(user_id).await?;
            if !is_whitelisted {
                bot.send_message(chat_id, "You are not whitelisted. Please contact admin.").await?;
            } else {
                bot.send_message(chat_id, "Welcome! Use /help to see commands.").await?;
            }
        }
        Command::CreateWallet { name, key } => {
            if !db.is_whitelisted(user_id).await? {
                bot.send_message(chat_id, "Not whitelisted.").await?;
                return Ok(());
            }
            // TODO: encrypt `key` before storing, e.g. AES with ENCRYPTION_KEY
            let private_enc = encrypt_key(&key)?;
            match db.add_wallet(user_id, name.clone(), derive_address(&key)?, private_enc).await {
                Ok(entry) => {
                    bot.send_message(chat_id, format!("Wallet created with ID: {}", entry.id)).await?;
                }
                Err(e) => {
                    bot.send_message(chat_id, format!("Error creating wallet: {}", e)).await?;
                }
            }
        }
        Command::ListWallets => {
            if !db.is_whitelisted(user_id).await? {
                bot.send_message(chat_id, "Not whitelisted.").await?;
                return Ok(());
            }
            let wallets = db.list_wallets(user_id).await?;
            if wallets.is_empty() {
                bot.send_message(chat_id, "No wallets. Use /create_wallet.").await?;
            } else {
                let mut text = String::from("Your wallets:\n");
                for w in wallets {
                    text.push_str(&format!("- ID: {}, name: {}, address: {}\n", w.id, w.name, w.address));
                }
                bot.send_message(chat_id, text).await?;
            }
        }
        Command::RemoveWallet { wallet_id } => {
            if !db.is_whitelisted(user_id).await? {
                bot.send_message(chat_id, "Not whitelisted.").await?;
                return Ok(());
            }
            match ObjectId::parse_str(&wallet_id) {
                Ok(oid) => {
                    if let Err(e) = db.remove_wallet(user_id, oid).await {
                        bot.send_message(chat_id, format!("Error removing wallet: {}", e)).await?;
                    } else {
                        bot.send_message(chat_id, "Wallet removed.").await?;
                    }
                }
                Err(_) => {
                    bot.send_message(chat_id, "Invalid wallet ID format.").await?;
                }
            }
        }
        Command::SetTradingWallet { wallet_id } => {
            if !db.is_whitelisted(user_id).await? {
                bot.send_message(chat_id, "Not whitelisted.").await?;
                return Ok(());
            }
            match ObjectId::parse_str(&wallet_id) {
                Ok(oid) => {
                    if let Err(e) = db.set_trading_wallet(user_id, oid).await {
                        bot.send_message(chat_id, format!("Error: {}", e)).await?;
                    } else {
                        bot.send_message(chat_id, "Trading wallet set.").await?;
                    }
                }
                Err(_) => {
                    bot.send_message(chat_id, "Invalid wallet ID.").await?;
                }
            }
        }
        Command::SetMinTokens { amount } => {
            if !db.is_whitelisted(user_id).await? {
                bot.send_message(chat_id, "Not whitelisted.").await?;
                return Ok(());
            }
            if let Err(e) = db.update_setting_min_tokens(user_id, amount).await {
                bot.send_message(chat_id, format!("Error: {}", e)).await?;
            } else {
                bot.send_message(chat_id, format!("min_tokens_to_buy set to {}", amount)).await?;
            }
        }
        Command::SetSlippage { percent } => {
            if !db.is_whitelisted(user_id).await? {
                bot.send_message(chat_id, "Not whitelisted.").await?;
                return Ok(());
            }
            if let Err(e) = db.update_setting_slippage(user_id, percent).await {
                bot.send_message(chat_id, format!("Error: {}", e)).await?;
            } else {
                bot.send_message(chat_id, format!("slippage_pct set to {}%", percent)).await?;
            }
        }
        Command::SetTip { amount } => {
            if !db.is_whitelisted(user_id).await? {
                bot.send_message(chat_id, "Not whitelisted.").await?;
                return Ok(());
            }
            if let Err(e) = db.update_setting_tip(user_id, amount).await {
                bot.send_message(chat_id, format!("Error: {}", e)).await?;
            } else {
                bot.send_message(chat_id, format!("tip_amount set to {}", amount)).await?;
            }
        }
        Command::Withdraw { wallet_id, dest, amount } |
        Command::Transfer { wallet_id, dest, amount } => {
            if !db.is_whitelisted(user_id).await? {
                bot.send_message(chat_id, "Not whitelisted.").await?;
                return Ok(());
            }
            // Parse wallet ID
            let oid = match ObjectId::parse_str(&wallet_id) {
                Ok(o) => o,
                Err(_) => {
                    bot.send_message(chat_id, "Invalid wallet ID.").await?;
                    return Ok(());
                }
            };
            // Fetch user & wallet
            if let Some(user) = db.get_user(user_id).await? {
                if let Some(wallet) = user.wallets.iter().find(|w| w.id == oid) {
                    // Decrypt private key: 
                    let private_key = decrypt_key(&wallet.private_key_enc)?;
                    // Then implement withdraw/transfer logic, e.g., call chain RPC:
                    // withdraw: send transaction moving `amount` tokens from wallet.address to dest.
                    // Here just stub:
                    bot.send_message(chat_id, format!(
                        "{} from wallet {} to {} of amount {}: not implemented yet",
                        if matches!(cmd, Command::Withdraw {..}) { "Withdraw" } else { "Transfer" },
                        wallet.name, dest, amount
                    )).await?;
                } else {
                    bot.send_message(chat_id, "Wallet not found.").await?;
                }
            } else {
                bot.send_message(chat_id, "User not found.").await?;
            }
        }
        Command::Whitelist { telegram_id: target } => {
            if !admin_ids.contains(&user_id) {
                bot.send_message(chat_id, "You are not admin.").await?;
                return Ok(());
            }
            if let Err(e) = db.set_whitelist(target, true).await {
                bot.send_message(chat_id, format!("Error whitelisting: {}", e)).await?;
            } else {
                bot.send_message(chat_id, format!("User {} whitelisted.", target)).await?;
            }
        }
        Command::Unwhitelist { telegram_id: target } => {
            if !admin_ids.contains(&user_id) {
                bot.send_message(chat_id, "You are not admin.").await?;
                return Ok(());
            }
            if let Err(e) = db.set_whitelist(target, false).await {
                bot.send_message(chat_id, format!("Error removing whitelist: {}", e)).await?;
            } else {
                bot.send_message(chat_id, format!("User {} unwhitelisted.", target)).await?;
            }
        }
    }
    Ok(())
}

// Placeholder encryption/decryption. In real use, implement AES or other.
fn encrypt_key(raw: &str) -> Result<String> {
    // e.g. base64 or AES-encrypt with env key
    Ok(base64::encode(raw))
}
fn decrypt_key(enc: &str) -> Result<String> {
    let bytes = base64::decode(enc)?;
    Ok(String::from_utf8(bytes)?)
}

// Placeholder derive address from private key/mnemonic: implement per-chain.
fn derive_address(key: &str) -> Result<String> {
    // stub: in Solana, parse keypair and extract pubkey:
    // ...
    Ok("public_address_stub".to_string())
}
