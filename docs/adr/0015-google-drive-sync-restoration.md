# 0015. Google Drive Sync Restoration and Overwrite Management

## Status
Accepted

## Context
KeyVault performs automatic, silent background synchronization of the encrypted vault database to the user's private Google Drive `appDataFolder`. During KDF initialization, Argon2id derives the vault encryption key from the Master Password combined with a randomly generated local salt (`vault_salt`). Previously, this salt was stored exclusively in local storage and was never synchronized.

This created a critical structural failure: if a user reset their local vault database, reinstalled the extension, or connected to Google Drive sync from a second device, the new local vault would generate a new random salt. Even when inputting the correct master password, the derived key would differ from the key used to encrypt the remote backup on Google Drive. Consequently, the background sync operation would fail with `aead::Error` (decryption failure), leaving the user unable to restore or sync their credentials.

## Decision
To address this, we will introduce a backward-compatible salt packaging protocol and explicit manual overwrite operations within the options dashboard:

1. **Self-Contained Salt Wrapper (`KV01` Header)**:
   The `vault.db` file uploaded to Google Drive will be packaged with a binary header:
   `[4-byte Magic 'KV01'] + [16-byte Salt] + [12-byte Nonce] + [Ciphertext]`
   * *Backward Compatibility*: If a downloaded file lacks the `KV01` magic bytes, the sync client falls back to treating it as the old format (raw `[12-byte Nonce] + [Ciphertext]`) and decrypts it using the local salt.

2. **Explicit Metadata Comparison & Overwrite States**:
   Under the Sync tab, the UI will compare local and remote vault metadata:
   * **Local**: Last updated timestamp, count of active and trash entries.
   * **Remote**: Cloud file modified timestamp, file size, and remote entry count (if decryptable).

3. **Interactive Sync Overwrites**:
   We will expose three operations to the user:
   * **Sync & Merge**: Runs a two-way merge using the `wasm_vault_merge` core and updates both contexts.
   * **Restore Remote (Overwrite Local)**: Replaces local storage completely with the remote backup. If a salt/key mismatch is detected, it securely prompts the user for the remote vault's Master Password, decrypts the payload, and overwrites the local salt and payload.
   * **Back Up Local (Overwrite Remote)**: Overwrites the Google Drive file completely with the local database.

4. **Cloud Database Deletion**:
   When disconnecting sync, the user will be offered an option to delete the remote `vault.db` from their Google Drive `appDataFolder` before signing out to guarantee no residual data remains online.

## Consequences
- Restoring vault credentials from Google Drive on new devices, browser profiles, or after local resets is now fully supported.
- Eliminates silent background sync failures (`aead::Error`) due to key/salt mismatches by converting them into explicit, interactive user resolutions.
- Grants users total sovereignty over their cloud backups by providing clear merge, restore, backup, and cloud-wipe controls.
