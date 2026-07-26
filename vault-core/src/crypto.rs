use argon2::{Algorithm, Argon2, Params, Version};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct EncryptionKey {
    pub inner: Vec<u8>,
}

pub fn generate_random_bytes(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    getrandom::fill(buf)
}

pub fn encrypt(
    key: &EncryptionKey,
    plaintext: &[u8],
    nonce: &[u8],
) -> Result<Vec<u8>, aes_gcm::Error> {
    use aes_gcm::{Aes256Gcm, KeyInit, aead::Aead};

    let cipher = Aes256Gcm::new_from_slice(&key.inner).map_err(|_| aes_gcm::Error)?;

    let nonce_ref = aes_gcm::Nonce::try_from(nonce).map_err(|_| aes_gcm::Error)?;

    let ciphertext = cipher.encrypt(&nonce_ref, plaintext)?;

    Ok(ciphertext)
}

pub fn decrypt(
    key: &EncryptionKey,
    ciphertext: &[u8],
    nonce: &[u8],
) -> Result<Vec<u8>, aes_gcm::Error> {
    use aes_gcm::{Aes256Gcm, KeyInit, aead::Aead};

    let cipher = Aes256Gcm::new_from_slice(&key.inner).map_err(|_| aes_gcm::Error)?;

    let nonce_ref = aes_gcm::Nonce::try_from(nonce).map_err(|_| aes_gcm::Error)?;

    let plaintext = cipher.decrypt(&nonce_ref, ciphertext)?;

    Ok(plaintext)
}

#[derive(Debug, Clone, Copy)]
pub struct Argon2Params {
    pub memory_cost: u32,
    pub time_cost: u32,
    pub parallelism: u32,
}

impl Default for Argon2Params {
    fn default() -> Self {
        Self {
            memory_cost: 65536,
            time_cost: 3,
            parallelism: 1,
        }
    }
}

pub fn derive_key(
    passwd: &str,
    salt: &[u8],
    params: Argon2Params,
) -> Result<EncryptionKey, argon2::Error> {
    let argon2_params = Params::new(
        params.memory_cost,
        params.time_cost,
        params.parallelism,
        Some(32),
    )?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon2_params);
    let mut key = vec![0u8; 32];

    argon2.hash_password_into(passwd.as_bytes(), salt, &mut key)?;

    Ok(EncryptionKey { inner: key })
}

pub fn derive_key_default(passwd: &str, salt: &[u8]) -> Result<EncryptionKey, argon2::Error> {
    derive_key(passwd, salt, Argon2Params::default())
}

pub fn mock_derive_key(password: &str, salt: &[u8]) -> Vec<u8> {
    let mut key = Vec::new();

    // Convert the password characters to bytes and append them
    key.extend_from_slice(password.as_bytes());
    key.extend_from_slice(salt); // Append the salt bytes

    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_derive_key() {
        let password = "super-secret-password";
        let salt = [1, 2, 3, 4]; // Fixed-size stack array of bytes

        // Pass salt as a borrowed slice using the "&" operator
        let key = mock_derive_key(password, &salt);

        // Assert that the output key matches the concatenation
        let expected_prefix = password.as_bytes();
        assert_eq!(&key[0..expected_prefix.len()], expected_prefix);
        assert_eq!(&key[expected_prefix.len()..], &salt);
    }

    #[test]
    fn test_real_derive_key() {
        let password = "my-secure-master-password";
        let salt = b"saltysaltysaltyy";

        let key = derive_key_default(password, salt).expect("Failed to derive key");

        // Assert length is exactly 32 bytes (256 bits)
        assert_eq!(key.inner.len(), 32);

        // Ensure same password and salt produces the same key (deterministic)
        let key2 = derive_key_default(password, salt).expect("Failed to derive key");
        assert_eq!(key.inner, key2.inner);

        // Ensure different password produces different key
        let key3 = derive_key_default("different-password", salt).expect("Failed to derive key");
        assert_ne!(key.inner, key3.inner);
    }

    #[test]
    fn test_aes_gcm_roundtrip() {
        let password = "master-password";
        let salt = b"saltysaltysaltyy";

        let key = derive_key_default(password, salt).unwrap();
        let message = b"my super secret credentials!";

        // 12-byte nonce generated using our CSPRNG helper
        let mut nonce = [0u8; 12];
        generate_random_bytes(&mut nonce).unwrap();

        // Encrypt
        let ciphertext = encrypt(&key, message, &nonce).expect("Encryption failed");
        assert_ne!(ciphertext, message); // Ciphertext must look random

        // Decrypt
        let decrypted = decrypt(&key, &ciphertext, &nonce).expect("Decryption failed");
        assert_eq!(decrypted, message); // Must match original message

        // Tamper test: modify one byte of ciphertext
        let mut tampered = ciphertext.clone();
        tampered[0] ^= 1; // Flip one bit

        // Decryption MUST fail due to authentication failure
        let tamper_result = decrypt(&key, &tampered, &nonce);
        assert!(tamper_result.is_err());
    }
}
