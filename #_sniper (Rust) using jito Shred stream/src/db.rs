// src/db.rs
use mongodb::{Client, Collection, options::ClientOptions};
use mongodb::bson::{doc, oid::ObjectId};
use crate::models::{UserRecord, WalletEntry};
use anyhow::{Result, Context};
use futures::TryStreamExt;

pub struct Database {
    users: Collection<UserRecord>,
}

impl Database {
    /// Initialize MongoDB client and get `users` collection.
    pub async fn new(mongo_uri: &str, db_name: &str) -> Result<Self> {
        let mut client_options = ClientOptions::parse(mongo_uri).await?;
        client_options.app_name = Some("TelegramWalletBot".to_string());
        let client = Client::with_options(client_options)?;
        let db = client.database(db_name);
        let users = db.collection::<UserRecord>("users");
        // Ensure index on telegram_id
        users.create_index(
            mongodb::IndexModel::builder()
                .keys(doc! { "telegram_id": 1 })
                .options(Some(mongodb::options::IndexOptions::builder().unique(true).build()))
                .build(),
            None
        ).await?;
        Ok(Self { users })
    }

    /// Fetch or create a user record. If not exist, create with is_whitelisted=false.
    pub async fn get_or_create_user(&self, telegram_id: i64) -> Result<UserRecord> {
        if let Some(user) = self.users
            .find_one(doc! { "telegram_id": telegram_id }, None)
            .await?
        {
            Ok(user)
        } else {
            let new = UserRecord {
                id: ObjectId::new(),
                telegram_id,
                is_whitelisted: false,
                wallets: Vec::new(),
                trading_wallet_id: None,
                min_tokens_to_buy: None,
                slippage_pct: None,
                tip_amount: None,
            };
            self.users.insert_one(&new, None).await?;
            Ok(new)
        }
    }

    /// Check whitelist
    pub async fn is_whitelisted(&self, telegram_id: i64) -> Result<bool> {
        if let Some(user) = self.users.find_one(doc! { "telegram_id": telegram_id }, None).await? {
            Ok(user.is_whitelisted)
        } else {
            Ok(false)
        }
    }

    /// Set whitelist status (admin only).
    pub async fn set_whitelist(&self, telegram_id: i64, status: bool) -> Result<()> {
        let filter = doc! { "telegram_id": telegram_id };
        let update = doc! { "$set": { "is_whitelisted": status } };
        self.users.update_one(filter, update, None).await?;
        Ok(())
    }

    /// Add a wallet (limit 5). Returns error if >5.
    pub async fn add_wallet(&self, telegram_id: i64, name: String, address: String, private_enc: String) -> Result<WalletEntry> {
        // Load user
        let mut user = self.get_or_create_user(telegram_id).await?;
        if user.wallets.len() >= 5 {
            anyhow::bail!("wallet limit reached (5)");
        }
        // Create entry
        let entry = WalletEntry {
            id: ObjectId::new(),
            name,
            address,
            private_key_enc: private_enc,
        };
        user.wallets.push(entry.clone());
        // Update in DB
        self.users.update_one(
            doc! { "telegram_id": telegram_id },
            doc! { "$set": { "wallets": bson::to_bson(&user.wallets)? } },
            None
        ).await?;
        Ok(entry)
    }

    /// List wallets
    pub async fn list_wallets(&self, telegram_id: i64) -> Result<Vec<WalletEntry>> {
        if let Some(user) = self.users.find_one(doc! { "telegram_id": telegram_id }, None).await? {
            Ok(user.wallets)
        } else {
            Ok(Vec::new())
        }
    }

    /// Set trading wallet by wallet id
    pub async fn set_trading_wallet(&self, telegram_id: i64, wallet_id: ObjectId) -> Result<()> {
        // Verify wallet belongs to user
        if let Some(user) = self.users.find_one(doc! { "telegram_id": telegram_id }, None).await? {
            if user.wallets.iter().any(|w| w.id == wallet_id) {
                self.users.update_one(
                    doc! { "telegram_id": telegram_id },
                    doc! { "$set": { "trading_wallet_id": wallet_id } },
                    None
                ).await?;
                Ok(())
            } else {
                anyhow::bail!("wallet not found");
            }
        } else {
            anyhow::bail!("user not found");
        }
    }

    /// Update a setting (min_tokens_to_buy, slippage_pct, tip_amount)
    pub async fn update_setting_min_tokens(&self, telegram_id: i64, min_tokens: f64) -> Result<()> {
        self.users.update_one(
            doc! { "telegram_id": telegram_id },
            doc! { "$set": { "min_tokens_to_buy": min_tokens } },
            None
        ).await?;
        Ok(())
    }
    pub async fn update_setting_slippage(&self, telegram_id: i64, slippage_pct: f64) -> Result<()> {
        self.users.update_one(
            doc! { "telegram_id": telegram_id },
            doc! { "$set": { "slippage_pct": slippage_pct } },
            None
        ).await?;
        Ok(())
    }
    pub async fn update_setting_tip(&self, telegram_id: i64, tip: u64) -> Result<()> {
        self.users.update_one(
            doc! { "telegram_id": telegram_id },
            doc! { "$set": { "tip_amount": tip } },
            None
        ).await?;
        Ok(())
    }

    /// Remove wallet by id
    pub async fn remove_wallet(&self, telegram_id: i64, wallet_id: ObjectId) -> Result<()> {
        // Pull from array
        self.users.update_one(
            doc! { "telegram_id": telegram_id },
            doc! { "$pull": { "wallets": { "id": wallet_id } } },
            None
        ).await?;
        // If trading_wallet_id was this, clear it
        self.users.update_one(
            doc! { "telegram_id": telegram_id, "trading_wallet_id": wallet_id },
            doc! { "$set": { "trading_wallet_id": bson::Bson::Null } },
            None
        ).await?;
        Ok(())
    }

    /// Fetch user record
    pub async fn get_user(&self, telegram_id: i64) -> Result<Option<UserRecord>> {
        Ok(self.users.find_one(doc! { "telegram_id": telegram_id }, None).await?)
    }
}
