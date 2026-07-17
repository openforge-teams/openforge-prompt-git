use crate::error::{AppError, AppResult, CryptoError};
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use keyring::Entry;
use rand::RngCore;

const SERVICE: &str = "prompt-git";
const ACCOUNT: &str = "master-key";

fn get_or_create_master_key() -> AppResult<[u8; 32]> {
    let entry = Entry::new(SERVICE, ACCOUNT).map_err(|e| CryptoError::Keyring(e.to_string()))?;
    match entry.get_password() {
        Ok(encoded) => {
            let bytes = B64
                .decode(encoded)
                .map_err(|e| CryptoError::Decrypt(e.to_string()))?;
            if bytes.len() != 32 {
                return Err(AppError::Crypto(CryptoError::Decrypt(
                    "invalid master key length".into(),
                )));
            }
            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes);
            Ok(key)
        }
        Err(_) => {
            let mut key = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut key);
            entry
                .set_password(&B64.encode(key))
                .map_err(|e| CryptoError::Keyring(e.to_string()))?;
            Ok(key)
        }
    }
}

pub fn encrypt_secret(plaintext: &str) -> AppResult<String> {
    if plaintext.is_empty() {
        return Ok(String::new());
    }
    let key = get_or_create_master_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| CryptoError::Encrypt(e.to_string()))?;
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| CryptoError::Encrypt(e.to_string()))?;
    let mut out = nonce_bytes.to_vec();
    out.extend(ciphertext);
    Ok(B64.encode(out))
}

pub fn decrypt_secret(encoded: &str) -> AppResult<String> {
    if encoded.is_empty() {
        return Ok(String::new());
    }
    let raw = B64
        .decode(encoded)
        .map_err(|e| CryptoError::Decrypt(e.to_string()))?;
    if raw.len() < 13 {
        return Err(AppError::Crypto(CryptoError::Decrypt(
            "ciphertext too short".into(),
        )));
    }
    let (nonce_bytes, ciphertext) = raw.split_at(12);
    let key = get_or_create_master_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| CryptoError::Decrypt(e.to_string()))?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let plain = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| CryptoError::Decrypt(e.to_string()))?;
    String::from_utf8(plain).map_err(|e| AppError::Crypto(CryptoError::Decrypt(e.to_string())))
}

pub fn mask_api_key(key: &str) -> String {
    if key.is_empty() {
        return String::new();
    }
    if key.len() <= 8 {
        return "****".into();
    }
    format!("{}…{}", &key[..4], &key[key.len() - 4..])
}

pub fn hash_password(password: &str) -> String {
    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    hasher.update(b"prompt-git-app-password:");
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}
