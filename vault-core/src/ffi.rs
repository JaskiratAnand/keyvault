use crate::vault::{Vault, VaultItem};
use wasm_bindgen::prelude::*;

/// Derives a key using Argon2id.
#[wasm_bindgen]
pub fn wasm_derive_key(
    password: &str,
    salt: &[u8],
    m_cost: Option<u32>,
    t_cost: Option<u32>,
    p_cost: Option<u32>,
) -> Result<Vec<u8>, JsValue> {
    let mut params = crate::crypto::Argon2Params::default();
    if let Some(m) = m_cost {
        params.memory_cost = m;
    }
    if let Some(t) = t_cost {
        params.time_cost = t;
    }
    if let Some(p) = p_cost {
        params.parallelism = p;
    }
    let key = crate::crypto::derive_key(password, salt, params)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(key.inner.clone())
}

/// Encrypts a plaintext payload using AES-256-GCM with a random nonce.
/// The returned vector prepends the 12-byte nonce: [12-byte Nonce] + [Ciphertext]
#[wasm_bindgen]
pub fn wasm_encrypt_vault(key_bytes: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, JsValue> {
    // Generate random 12-byte nonce
    let mut nonce = [0u8; 12];
    crate::crypto::generate_random_bytes(&mut nonce)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Encrypt
    let key = crate::crypto::EncryptionKey {
        inner: key_bytes.to_vec(),
    };
    let ciphertext = crate::crypto::encrypt(&key, plaintext, &nonce)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Package: Prepend nonce to ciphertext
    let mut payload = nonce.to_vec();
    payload.extend(ciphertext);

    Ok(payload)
}

/// Decrypts a vault payload. Expects [12-byte Nonce] + [Ciphertext]
#[wasm_bindgen]
pub fn wasm_decrypt_vault(key_bytes: &[u8], payload: &[u8]) -> Result<Vec<u8>, JsValue> {
    if payload.len() < 12 {
        return Err(JsValue::from_str(
            "Invalid payload length (must be at least 12 bytes)",
        ));
    }

    // Split the nonce and ciphertext
    let (nonce, ciphertext) = payload.split_at(12);

    // Decrypt
    let key = crate::crypto::EncryptionKey {
        inner: key_bytes.to_vec(),
    };
    let plaintext = crate::crypto::decrypt(&key, ciphertext, nonce)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(plaintext)
}

/// Creates a new empty vault and returns its JSON representation.
#[wasm_bindgen]
pub fn wasm_vault_new() -> String {
    let vault = Vault::new();
    serde_json::to_string(&vault).unwrap()
}

/// Adds an item to a vault JSON, returning the updated vault JSON.
#[wasm_bindgen]
pub fn wasm_vault_add_entry(vault_json: &str, item_json: &str) -> Result<String, JsValue> {
    let mut vault: Vault =
        serde_json::from_str(vault_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let item: VaultItem =
        serde_json::from_str(item_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    vault.add_item(item);

    serde_json::to_string(&vault).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Updates an item in a vault JSON, returning the updated vault JSON.
#[wasm_bindgen]
pub fn wasm_vault_update_entry(vault_json: &str, item_json: &str) -> Result<String, JsValue> {
    let mut vault: Vault =
        serde_json::from_str(vault_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let item: VaultItem =
        serde_json::from_str(item_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    vault.update_item(item).map_err(JsValue::from_str)?;

    serde_json::to_string(&vault).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Deletes an item and records a tombstone, returning the updated vault JSON.
#[wasm_bindgen]
pub fn wasm_vault_delete_entry(
    vault_json: &str,
    id: &str,
    deleted_at: &str,
) -> Result<String, JsValue> {
    let mut vault: Vault =
        serde_json::from_str(vault_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    vault
        .delete_item(id, deleted_at)
        .map_err(JsValue::from_str)?;

    serde_json::to_string(&vault).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Restores an item from the trash, returning the updated vault JSON.
#[wasm_bindgen]
pub fn wasm_vault_restore_entry(
    vault_json: &str,
    id: &str,
    restored_at: &str,
) -> Result<String, JsValue> {
    let mut vault: Vault =
        serde_json::from_str(vault_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    vault
        .restore_item(id, restored_at)
        .map_err(JsValue::from_str)?;

    serde_json::to_string(&vault).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Permanently deletes (purges) an item, returning the updated vault JSON.
#[wasm_bindgen]
pub fn wasm_vault_purge_entry(
    vault_json: &str,
    id: &str,
    purged_at: &str,
) -> Result<String, JsValue> {
    let mut vault: Vault =
        serde_json::from_str(vault_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    vault.purge_item(id, purged_at).map_err(JsValue::from_str)?;

    serde_json::to_string(&vault).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Merges two vault JSON structures, returning the merged vault JSON.
#[wasm_bindgen]
pub fn wasm_vault_merge(vault_a_json: &str, vault_b_json: &str) -> Result<String, JsValue> {
    let mut vault_a: Vault =
        serde_json::from_str(vault_a_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let vault_b: Vault =
        serde_json::from_str(vault_b_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    vault_a.merge(vault_b);

    serde_json::to_string(&vault_a).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Generates a credential (password, passphrase, or PIN) based on the JSON configuration.
/// Returns JSON: { "credential": "...", "entropy": 51.7 }
#[wasm_bindgen]
pub fn wasm_generate(config_json: &str) -> Result<String, JsValue> {
    let config: crate::generator::GeneratorConfig =
        serde_json::from_str(config_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let (credential, entropy) = crate::generator::generate(&config).map_err(JsValue::from_str)?;

    let result = serde_json::json!({
        "credential": credential,
        "entropy": entropy
    });

    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Exports the vault entries as a CSV string.
#[wasm_bindgen]
pub fn wasm_vault_export_csv(vault_json: &str) -> Result<String, JsValue> {
    let vault: Vault =
        serde_json::from_str(vault_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(vault.to_csv())
}

/// Parses CSV data and imports entries into the vault, returning the updated vault JSON.
#[wasm_bindgen]
pub fn wasm_vault_import_csv(
    vault_json: &str,
    csv_text: &str,
    current_time: &str,
) -> Result<String, JsValue> {
    let mut vault: Vault =
        serde_json::from_str(vault_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    vault
        .import_csv(csv_text, current_time)
        .map_err(|e| JsValue::from_str(&e))?;
    serde_json::to_string(&vault).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Encrypts vault plaintext and prepends [KV01 magic] + [16-byte Salt] + [12-byte Nonce] + [Ciphertext]
#[wasm_bindgen]
pub fn wasm_encrypt_vault_packaged(
    key_bytes: &[u8],
    salt: &[u8],
    plaintext: &[u8],
) -> Result<Vec<u8>, JsValue> {
    // Generate random 12-byte nonce
    let mut nonce = [0u8; 12];
    crate::crypto::generate_random_bytes(&mut nonce)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Encrypt
    let key = crate::crypto::EncryptionKey {
        inner: key_bytes.to_vec(),
    };
    let ciphertext = crate::crypto::encrypt(&key, plaintext, &nonce)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Package: [KV01] + [Salt] + [Nonce] + [Ciphertext]
    let mut payload = vec![75, 86, 48, 49]; // "KV01"
    payload.extend_from_slice(salt);
    payload.extend_from_slice(&nonce);
    payload.extend(ciphertext);

    Ok(payload)
}

/// Extracts the 16-byte salt from a packaged payload.
/// Returns Err if the magic header is not present.
#[wasm_bindgen]
pub fn wasm_vault_extract_salt(payload: &[u8]) -> Result<Vec<u8>, JsValue> {
    if payload.len() >= 20
        && payload[0] == 75
        && payload[1] == 86
        && payload[2] == 48
        && payload[3] == 49
    {
        Ok(payload[4..20].to_vec())
    } else {
        Err(JsValue::from_str("Missing magic header KV01"))
    }
}

/// Decrypts a packaged payload. Strictly requires the magic header 'KV01'.
#[wasm_bindgen]
pub fn wasm_decrypt_vault_packaged(key_bytes: &[u8], payload: &[u8]) -> Result<Vec<u8>, JsValue> {
    if payload.len() >= 20
        && payload[0] == 75
        && payload[1] == 86
        && payload[2] == 48
        && payload[3] == 49
    {
        // KV01 Format: Skip Magic (4 bytes) + Salt (16 bytes)
        let encrypted_part = &payload[20..];
        wasm_decrypt_vault(key_bytes, encrypted_part)
    } else {
        Err(JsValue::from_str(
            "Invalid vault backup file: missing magic header KV01",
        ))
    }
}

/// Generates a TOTP code.
#[wasm_bindgen]
pub fn wasm_generate_totp(secret: &str, timestamp: u64) -> Result<String, JsValue> {
    crate::totp::generate_totp(secret, timestamp).map_err(|e| JsValue::from_str(&e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_wasm_json_deserialization_failures() {
        let malformed_json = "{ invalid }";
        let valid_vault = wasm_vault_new();

        assert!(wasm_vault_add_entry(&valid_vault, malformed_json).is_err());
        assert!(wasm_vault_update_entry(&valid_vault, malformed_json).is_err());
        assert!(wasm_vault_merge(&valid_vault, malformed_json).is_err());
        assert!(wasm_vault_merge(malformed_json, &valid_vault).is_err());
        assert!(wasm_generate(malformed_json).is_err());
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_wasm_decrypt_bounds_and_magic() {
        let key = vec![0u8; 32];
        let short_payload = vec![0u8; 11];
        let result = wasm_decrypt_vault(&key, &short_payload);
        assert!(result.is_err());

        // Validate error message if possible
        if let Some(msg) = result.unwrap_err().as_string() {
            assert!(msg.contains("Invalid payload length"));
        }

        let invalid_magic_payload = vec![0u8; 25];
        assert!(wasm_vault_extract_salt(&invalid_magic_payload).is_err());
        assert!(wasm_decrypt_vault_packaged(&key, &invalid_magic_payload).is_err());
    }

    #[test]
    fn test_wasm_packaged_vault_flow() {
        let salt = b"salt1234salt1234";
        let key = wasm_derive_key("mypassword", salt, Some(1024), Some(1), Some(1)).unwrap();
        assert_eq!(key.len(), 32);

        let vault_json = wasm_vault_new();
        let item_json = r#"{
            "type": "SecureNote",
            "id": "uuid-123",
            "title": "My FFI Note",
            "notes": "FFI content",
            "tags": [],
            "created_at": "2026-07-01T12:00:00Z",
            "updated_at": "2026-07-01T12:00:00Z"
        }"#;

        let updated_vault_json = wasm_vault_add_entry(&vault_json, item_json).unwrap();
        let plaintext = updated_vault_json.as_bytes();

        // Package and encrypt
        let packaged = wasm_encrypt_vault_packaged(&key, salt, plaintext).unwrap();
        assert!(packaged.len() >= 20 + 12); // magic (4) + salt (16) + nonce (12)

        // Extract salt
        let extracted_salt = wasm_vault_extract_salt(&packaged).unwrap();
        assert_eq!(extracted_salt, salt);

        // Decrypt and verify
        let decrypted_bytes = wasm_decrypt_vault_packaged(&key, &packaged).unwrap();
        let decrypted_vault_json = String::from_utf8(decrypted_bytes).unwrap();
        assert_eq!(decrypted_vault_json, updated_vault_json);
    }

    #[test]
    fn test_wasm_generate_totp() {
        let secret = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
        let token = wasm_generate_totp(secret, 1234567890).unwrap();
        assert_eq!(token, "005924");
    }
}
