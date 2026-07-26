# 0005. Cryptographic Key Hierarchy

## Status
Accepted

## Context
In our initial design, the master key used to encrypt the vault was derived directly from the user's master password via Argon2id. Under this design, changing the master password would change the vault key, requiring a full vault decryption/re-encryption and invalidating the recovery code printed on the user's offline Emergency Kit.

## Decision
We will implement a two-tier cryptographic key hierarchy:
1. **Master Key (MK):** A cryptographically secure random 256-bit key generated once during onboarding. This key is used to encrypt and decrypt the actual vault entries (payload) and never changes.
2. **Key Encryption Key (KEK):** Derived via `Argon2id(master_password, salt)`. Its sole purpose is to encrypt/decrypt the Master Key. The resulting `encrypted_master_key` ciphertext is stored in the vault metadata.
3. **Recovery Key (RK):** Derived from the 128-bit random recovery code in the Emergency Kit. This is also used to encrypt the Master Key, storing the resulting `encrypted_recovery_key` ciphertext in the vault metadata.

## Consequences
- Changing the master password is fast and lightweight (only requires re-encrypting the 256-bit Master Key, not the entire vault).
- The printed Emergency Kit remains valid indefinitely across master password changes.
- Cryptographic isolation between the user's master password strength and the encryption strength of the vault payload (which is always protected by a high-entropy random 256-bit key).
