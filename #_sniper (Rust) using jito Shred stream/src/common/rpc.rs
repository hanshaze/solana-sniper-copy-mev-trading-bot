use anchor_lang::AccountDeserialize;
use anyhow::Result;
use base64::{prelude::BASE64_STANDARD, Engine};
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig, RpcSendTransactionConfig},
    rpc_filter::RpcFilterType,
    rpc_request::RpcRequest,
    rpc_response::{RpcResult, RpcSimulateTransactionResult},
};
use solana_sdk::{
    account::Account, commitment_config::CommitmentConfig, instruction::Instruction,
    message::Message, pubkey::Pubkey, signature::Signature, signer::signers::Signers,
    transaction::Transaction,
};
use solana_transaction_status::UiTransactionEncoding;

pub fn build_txn(
    client: &RpcClient,
    instructions: &[Instruction],
    fee_payer: &Pubkey,
    signing_keypairs: &dyn Signers,
) -> Result<Transaction> {
    let blockhash = client.get_latest_blockhash().unwrap();
    let message = Message::new_with_blockhash(instructions, Some(fee_payer), &blockhash);
    let mut transaction = Transaction::new_unsigned(message);

    transaction
        .try_partial_sign(signing_keypairs, blockhash)
        .unwrap();
    Ok(transaction)
}

pub fn send_txn(client: &RpcClient, txn: &Transaction, skip_preflight: bool) -> Result<Signature> {
    Ok(client.send_and_confirm_transaction_with_spinner_and_config(
        txn,
        CommitmentConfig::confirmed(),
        RpcSendTransactionConfig {
            skip_preflight,
            ..RpcSendTransactionConfig::default()
        },
    )?)
}

pub fn simulate_transaction(
    client: &RpcClient,
    transaction: &Transaction,
    sig_verify: bool,
    cfg: CommitmentConfig,
) -> RpcResult<RpcSimulateTransactionResult> {
    let serialized = bincode::serialize(transaction)
        .map_err(|e| (format!("Serialization failed: {e}")))
        .unwrap();
    let serialized_encoded = BASE64_STANDARD.encode(serialized);
    println!("{}", serialized_encoded);

    client.send(
        RpcRequest::SimulateTransaction,
        serde_json::json!([serialized_encoded, {
            "sigVerify": sig_verify, "commitment": cfg.commitment, "encoding": Some(UiTransactionEncoding::Base64)
        }]),
    )
}

pub fn send_without_confirm_txn(client: &RpcClient, txn: &Transaction) -> Result<Signature> {
    Ok(client.send_transaction_with_config(
        txn,
        RpcSendTransactionConfig {
            skip_preflight: true,
            ..RpcSendTransactionConfig::default()
        },
    )?)
}

pub fn get_account(client: &RpcClient, addr: &Pubkey) -> Result<Option<Vec<u8>>> {
    if let Some(account) = client
        .get_account_with_commitment(addr, CommitmentConfig::processed())?
        .value
    {
        let account_data = account.data;
        Ok(Some(account_data))
    } else {
        Ok(None)
    }
}

pub fn get_anchor_account<T: AccountDeserialize>(
    client: &RpcClient,
    addr: &Pubkey,
) -> Result<Option<T>> {
    if let Some(account) = client
        .get_account_with_commitment(addr, CommitmentConfig::processed())?
        .value
    {
        let mut data: &[u8] = &account.data;
        let ret = T::try_deserialize(&mut data).unwrap();
        Ok(Some(ret))
    } else {
        Ok(None)
    }
}

pub fn get_multiple_accounts(
    client: &RpcClient,
    pubkeys: &[Pubkey],
) -> Result<Vec<Option<Account>>> {
    Ok(client.get_multiple_accounts(pubkeys)?)
}

pub fn get_program_accounts_with_filters(
    client: &RpcClient,
    program: Pubkey,
    filters: Option<Vec<RpcFilterType>>,
) -> Result<Vec<(Pubkey, Account)>> {
    let accounts = client
        .get_program_accounts_with_config(
            &program,
            RpcProgramAccountsConfig {
                filters,
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64Zstd),
                    ..RpcAccountInfoConfig::default()
                },
                with_context: Some(false),
                sort_results: None, // <-- add this line
            },
        )
        .unwrap();
    Ok(accounts)
}
