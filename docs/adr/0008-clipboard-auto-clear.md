# 0008. Clipboard Auto-Clear

## Status
Accepted

## Context
When users copy passwords, usernames, or TOTP codes from the KeyVault UI to paste them into other applications, the sensitive plaintext data remains in the OS clipboard indefinitely. This exposes credentials to clipboard hijacking by other applications or scripts running on the system.

## Decision
We will implement an automatic clipboard clearing mechanism on all platform UIs (Browser Extension, Tauri Desktop, Mobile):
1. **Timeout:** After a user copies a credential field, the clipboard will be automatically cleared after a configurable duration (default: 30 seconds).
2. **Safety Check:** The clearing operation will be cancelled if the user copies a different piece of content before the timeout expires, preventing unexpected loss of unrelated clipboard data.

## Consequences
- Protects plaintext credentials from lingering in the OS clipboard.
- Requires platform-specific clipboard write permissions and APIs in the UI layers.
- Minor UX trade-off: users must paste within the timeout window.
