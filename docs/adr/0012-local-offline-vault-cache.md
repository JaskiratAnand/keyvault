# 0012. Local Offline Vault Cache

## Status
Accepted

## Context
A password manager must remain functional when the user does not have internet access (e.g. on a plane or in areas with poor connectivity). Since the primary vault storage is a file hosted on Google Drive, we need a local mechanism to allow decrypting and reading credentials offline.

## Decision
We will maintain an encrypted local cache of the vault on the host device:
1. **Storage:** The cached file is stored in platform-specific local storage (`chrome.storage.local` for browser extension, local disk for Tauri, secure file storage for mobile).
2. **Encryption:** The local cache is encrypted using the same Master Key (MK) as the remote Google Drive vault, using AES-256-GCM.
3. **Write Behavior:** Every successful pull from Google Drive updates the local cache. Every local mutation writes to the local cache immediately before debouncing and pushing to Google Drive.
4. **Offline Access:** If Google Drive is unreachable, the client decrypts and reads from the local cache. Writes made offline are cached locally and marked for sync once a connection is re-established.

## Consequences
- Fast, instant-on load times by reading from local cache first.
- Secure offline availability of credentials.
- Requires offline sync state tracking (sync pending flag) to push changes once online.
