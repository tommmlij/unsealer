use anyhow::anyhow;
use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use serde_json::Value;
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::{PublicKey, SecretKey};
use crate::cli::{PublicKeyValue, SecretKeyValue};

// We have this as a utility function to create the secrets. 
// It is not used in the normal unsealer run
#[allow(dead_code)]
pub fn encode_payload(
    config_json: &Value,
    manager_private_key: &SecretKey,
    server_public_key: &PublicKey,
) -> anyhow::Result<String> {
    
    let plaintext = serde_json::to_vec(&config_json)
        .map_err(|e| anyhow!("Failed to encode json payload: {}", e))?;

    // Encrypt the message
    let nonce = box_::gen_nonce();
    let ciphertext = box_::seal(
        &plaintext, &nonce, &server_public_key, &manager_private_key);

    // Concatenate nonce + ciphertext and base64url encode
    let mut full_message = nonce.0.to_vec();
    full_message.extend_from_slice(&ciphertext);
    
    Ok(URL_SAFE.encode(&full_message))

}


pub fn decode_payload(
    payload: &str,
    server_private_key: &SecretKeyValue,
    manager_public_key: &PublicKeyValue,
) -> anyhow::Result<String> {
    let decoded = URL_SAFE.decode(payload)
        .map_err(|e| anyhow!("Failed to decode base64 payload: {}", e))?;

    if decoded.len() < box_::NONCEBYTES {
        return Err(anyhow!("Invalid encrypted payload: too short"));
    }

    let nonce_bytes = &decoded[..box_::NONCEBYTES];
    let nonce = box_::Nonce::from_slice(nonce_bytes)
        .ok_or_else(|| anyhow!("Failed to create nonce from slice: invalid length"))?;

    let ciphertext = &decoded[box_::NONCEBYTES..];

    let decrypted = box_::open(ciphertext, &nonce, manager_public_key, server_private_key)
        .map_err(|_| anyhow!("Decryption failed"))?;

    String::from_utf8(decrypted)
        .map_err(|e| anyhow!("Failed to convert decrypted bytes to UTF-8 string: {}", e))
}
