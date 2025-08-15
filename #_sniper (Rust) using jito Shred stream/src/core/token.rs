use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use spl_token_2022::{
    extension::StateWithExtensionsOwned,
    state::{Account, Mint},
};
use spl_token_client::{
    client::{ProgramClient, ProgramRpcClient, ProgramRpcClientSendTransaction},
    token::{Token, TokenError, TokenResult},
};
use std::sync::Arc;

use tracing::info;

pub async fn get_token_balance(
    client: Arc<solana_client::nonblocking::rpc_client::RpcClient>,
    keypair: Arc<Keypair>,
    mint: &Pubkey,
    owner: &Pubkey,
) -> Result<u64, TokenError> {
    let ata = get_associated_token_address(client.clone(), keypair.clone(), mint, owner);
    info!(
        "🔎 Fetching token balance for ATA: {} | Mint: {} | Owner: {}",
        ata, mint, owner
    );

    match get_account_info(client.clone(), keypair, mint, &ata).await {
        Ok(account_info) => {
            info!(
                "✅ Token balance found: {} tokens (raw amount)",
                account_info.base.amount
            );
            Ok(account_info.base.amount)
        }
        Err(e) => {
            info!("❌ Failed to fetch token balance for ATA {}: {:?}", ata, e);
            Err(e)
        }
    }
}

pub fn get_associated_token_address(
    client: Arc<solana_client::nonblocking::rpc_client::RpcClient>,
    keypair: Arc<Keypair>,
    address: &Pubkey,
    owner: &Pubkey,
) -> Pubkey {
    let token_client = Token::new(
        Arc::new(ProgramRpcClient::new(
            client.clone(),
            ProgramRpcClientSendTransaction,
        )),
        &spl_token::ID,
        address,
        None,
        Arc::new(Keypair::from_bytes(&keypair.to_bytes()).expect("failed to copy keypair")),
    );
    token_client.get_associated_token_address(owner)
}

pub async fn get_account_info(
    client: Arc<solana_client::nonblocking::rpc_client::RpcClient>,
    _keypair: Arc<Keypair>,
    address: &Pubkey,
    account: &Pubkey,
) -> TokenResult<StateWithExtensionsOwned<Account>> {
    let program_client = Arc::new(ProgramRpcClient::new(
        client.clone(),
        ProgramRpcClientSendTransaction,
    ));
    let account = program_client
        .get_account(*account)
        .await
        .map_err(TokenError::Client)?
        .ok_or(TokenError::AccountNotFound)
        .inspect_err(|err| println!("get_account_info: {} {}: mint {}", account, err, address))?;

    if account.owner != spl_token::ID {
        return Err(TokenError::AccountInvalidOwner);
    }
    let account = StateWithExtensionsOwned::<Account>::unpack(account.data)?;
    if account.base.mint != *address {
        return Err(TokenError::AccountInvalidMint);
    }

    Ok(account)
}

pub async fn get_mint_info(
    client: Arc<solana_client::nonblocking::rpc_client::RpcClient>,
    _keypair: Arc<Keypair>,
    address: &Pubkey,
) -> TokenResult<StateWithExtensionsOwned<Mint>> {
    let program_client = Arc::new(ProgramRpcClient::new(
        client.clone(),
        ProgramRpcClientSendTransaction,
    ));
    let account = program_client
        .get_account(*address)
        .await
        .map_err(TokenError::Client)?
        .ok_or(TokenError::AccountNotFound)
        .inspect_err(|err| println!("{} {}: mint {}", address, err, address))?;

    if account.owner != spl_token::ID {
        return Err(TokenError::AccountInvalidOwner);
    }

    let mint_result = StateWithExtensionsOwned::<Mint>::unpack(account.data).map_err(Into::into);
    let decimals: Option<u8> = None;
    if let (Ok(mint), Some(decimals)) = (&mint_result, decimals) {
        if decimals != mint.base.decimals {
            return Err(TokenError::InvalidDecimals);
        }
    }

    mint_result
}
