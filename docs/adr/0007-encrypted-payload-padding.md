# 0007. Encrypted Payload Padding

## Status
Accepted

## Context
When the vault is encrypted with AES-256-GCM, the ciphertext size matches the plaintext size. An attacker with read access to the user's Google Drive sync file can monitor changes to the file size to estimate the number of credentials stored or track when entries are modified. This constitutes a side-channel metadata leak.

## Decision
We will pad the serialized vault payload before encryption:
1. **Length Header:** Prepend a 4-byte big-endian integer to the serialized JSON string containing the exact length of the JSON payload.
2. **Padding:** Append random padding bytes to the payload to align the total size to the next **4 KB** boundary (e.g. 4096 bytes, 8192 bytes, etc.).
3. **Encryption:** Encrypt the combined length header + JSON payload + padding bytes as a single block using AES-256-GCM.
4. **Decryption:** Decrypt the block, read the first 4 bytes to obtain the payload length, and extract exactly that many bytes to parse the JSON.

## Consequences
- File sizes on Google Drive will only grow in 4 KB increments, obscuring the exact number of credentials and small edits.
- Prevents side-channel tracking of vault changes.
- Minimal storage and performance overhead.
