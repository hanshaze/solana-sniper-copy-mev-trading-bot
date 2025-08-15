// src/tginterface.rs
use teloxide::prelude::*;
use std::env;
use dotenv::dotenv;
use crate::db::Database;
use crate::commands::{handle_command, Command};

mod db;
mod models;
mod commands;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let bot_token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
    let mongo_uri = env::var("MONGODB_URI").expect("MONGODB_URI not set");
    let mongo_db = env::var("MONGODB_DB").unwrap_or_else(|_| "telegram_wallet_bot".to_string());
    let admin_ids: Vec<i64> = env::var("ADMIN_IDS")
        .unwrap_or_default()
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    let db = Database::new(&mongo_uri, &mongo_db)
        .await
        .expect("failed to initialize database");

    let bot = Bot::new(bot_token).auto_send();

    log::info!("Starting bot...");

    // Use a cloned Database in handler
    teloxide::commands_repl(bot.clone(), move |bot: AutoSend<Bot>, msg: Message, cmd: Command| {
        let db = db.clone(); // Database should be Cloneable or use Arc
        let admin_ids = admin_ids.clone();
        async move {
            if let Err(e) = handle_command(bot.clone().into_inner(), msg, cmd, db.clone(), admin_ids.clone()).await {
                log::error!("Error handling command: {:?}", e);
            }
        }
    }, Command::ty()).await;
}
