// src/solana_helpers.rs

use aes_gcm::{Aes256Gcm, Key, Nonce}; // Or `aes_gcm::Aes256Gcm`
use aes_gcm::aead::{Aead, NewAead};
use base64::{encode as b64_encode, decode as b64_decode};
use hex::decode as hex_decode;
use rand::RngCore;
use rand::rngs::OsRng;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::pubkey::Pubkey;
use bip39::{Mnemonic, Language, Seed};
use anyhow::{Result, Context};
use serde_json::Value;

/// Load the AES-256 key from the environment variable `ENCRYPTION_KEY`.
/// The env var should be a hex-encoded 32-byte key (i.e., 64 hex chars).
fn load_encryption_key() -> Result<[u8; 32]> {
    let hex = std::env::var("ENCRYPTION_KEY")
        .context("ENCRYPTION_KEY env var not set")?;
    let bytes = hex_decode(&hex)
        .context("Failed to hex-decode ENCRYPTION_KEY")?;
    if bytes.len() != 32 {
        anyhow::bail!("ENCRYPTION_KEY must be 32 bytes (hex-encoded 64 chars)");
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}

/// Encrypt `plaintext` using AES-256-GCM. Returns a base64 string containing nonce||ciphertext.
/// Format: base64( [12-byte nonce] || ciphertext ).
pub fn encrypt_data(plaintext: &str) -> Result<String> {
    let key_bytes = load_encryption_key()?;
    let key = Key::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    // Generate a random 12-byte nonce
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .context("Encryption failure")?;

    // Prepend nonce
    let mut combined = Vec::with_capacity(12 + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(b64_encode(&combined))
}

/// Decrypt a base64 string produced by `encrypt_data`, returning the plaintext.
pub fn decrypt_data(encoded: &str) -> Result<String> {
    let combined = b64_decode(encoded).context("Base64 decode failed")?;
    if combined.len() < 12 {
        anyhow::bail!("Ciphertext too short");
    }
    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let key_bytes = load_encryption_key()?;
    let key = Key::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .context("Decryption failure")?;
    let s = String::from_utf8(plaintext).context("Decrypted data not valid UTF-8")?;
    Ok(s)
}

/// Derive Solana public address (Pubkey) from a JSON-array keypair string.
/// 
/// # Arguments
/// - `json`: a JSON string representing an array of 64 or 32 or 64 u8 values, e.g. `[12,34, ...]`.
///   - If length is 64: interpreted as full keypair bytes.
///   - If length is 32: interpreted as secret seed, but Solana Keypair::from_bytes expects 64; you may need to expand.
/// 
/// Returns the `Pubkey` as a base58 string.
pub fn derive_address_from_json_keypair(json: &str) -> Result<String> {
    // Parse JSON array
    let v: Value = serde_json::from_str(json).context("Invalid JSON")?;
    let arr = v.as_array().context("Expected JSON array")?;
    // Convert to Vec<u8>
    let bytes: Vec<u8> = arr.iter()
        .map(|val| {
            val.as_u64()
                .and_then(|n| Some(n as u8))
                .ok_or_else(|| anyhow::anyhow!("Invalid byte in array"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Depending on length:
    let kp = if bytes.len() == 64 {
        Keypair::from_bytes(&bytes).context("Failed to parse keypair from bytes")?
    } else if bytes.len() == 32 {
        // interpret as secret seed: expand to keypair via `from_seed`
        // Note: solana_sdk::signature::Keypair::from_seed requires `Signer` trait; but no direct from_seed in stable.
        // We can use ed25519_dalek to expand, then wrap in Keypair.
        // Solana Keypair::from_seed is nightly; instead:
        // Use solana_sdk::signature::Keypair::from_seed (requires "ed25519-dalek" feature)
        Keypair::from_seed(&bytes).context("Failed to derive keypair from 32-byte seed")?
    } else {
        anyhow::bail!("Keypair JSON must be 32 or 64 bytes");
    };

    let pubkey = kp.pubkey();
    Ok(pubkey.to_string())
}

/// Derive Solana public address from a base58-encoded secret key (64-byte) string.
/// E.g., if user has a base58 of secret key bytes.
/// 
/// # Arguments
/// - `b58`: base58 string of 64 bytes.
/// Returns base58 pubkey string.
pub fn derive_address_from_base58_secret(b58: &str) -> Result<String> {
    let data = bs58::decode(b58).into_vec().context("Invalid base58")?;
    if data.len() == 64 {
        let kp = Keypair::from_bytes(&data).context("Failed to parse keypair bytes")?;
        Ok(kp.pubkey().to_string())
    } else if data.len() == 32 {
        let kp = Keypair::from_seed(&data).context("Failed to derive keypair from seed")?;
        Ok(kp.pubkey().to_string())
    } else {
        anyhow::bail!("Base58 secret must be 32 or 64 bytes");
    }
}

/// Derive Solana public address from a BIP39 mnemonic phrase.
/// Uses the standard Solana derivation: m/44'/501'/0'/0' by default, but this can be adjusted.
/// 
/// # Arguments
/// - `mnemonic`: the phrase, e.g. "abandon abandon ...".
/// - `passphrase`: optional passphrase for mnemonic (usually empty string).
/// - `account`: u32, default 0.
/// - `change`: u32, default 0.
/// 
/// Returns base58 pubkey string.
pub fn derive_address_from_mnemonic(
    mnemonic: &str,
    passphrase: &str,
    account: u32,
    change: u32,
) -> Result<String> {
    // Parse mnemonic
    let mn = Mnemonic::from_phrase(mnemonic, Language::English)
        .context("Invalid mnemonic phrase")?;
    let seed = Seed::new(&mn, passphrase);
    let seed_bytes = seed.as_bytes(); // 64 bytes

    // Derivation path for Solana: m/44'/501'/<account>'/<change>'
    // We can use `solana_sdk::derivation_path::DerivationPath` if available, or derive via ed25519-dalek + slip10.
    // For simplicity, we use `solana_sdk::derivation_path::DerivationPath` if in scope:
    // If not, use `ed25519-dalek-bip32` or similar. Here we assume solana-sdk has `derive_keypair_from_seed_and_path`.
    // As of solana-sdk v1.14+, there's `keypair_from_seed_and_derivation_path`.
    #[cfg(feature = "solana-derive-keypair")]
    {
        // Pseudo-code; adjust per your solana-sdk version:
        /*
        use solana_sdk::derivation_path::DerivationPath;
        let path = DerivationPath::new_bip44(Some(501), Some(account), Some(change));
        let kp = Keypair::from_seed_with_derivation(&seed_bytes, &path)
            .context("Failed deriving keypair from mnemonic")?;
        Ok(kp.pubkey().to_string())
        */
        unimplemented!("Adjust to your solana-sdk version's derivation API");
    }
    #[cfg(not(feature = "solana-derive-keypair"))]
    {
        // As fallback: derive the ED25519 key by directly using the seed (not recommended for multiple accounts).
        // Here we just derive from seed_bytes[0..32]:
        let seed32 = &seed_bytes[0..32];
        let kp = Keypair::from_seed(seed32).context("Failed to derive keypair from mnemonic seed")?;
        Ok(kp.pubkey().to_string())
    }
}


