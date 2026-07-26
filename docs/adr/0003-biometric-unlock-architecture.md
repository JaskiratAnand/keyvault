# 0003. Biometric Unlock Architecture

## Status
Accepted

## Context
Browser extensions are sandboxed and cannot directly access native OS secure enclaves or trigger Touch ID / Windows Hello APIs. The v1 release target is a standalone browser extension, while v2 is a Tauri desktop application. We need a secure, hardware-backed biometric unlock mechanism that works for both standalone extension usage and integrated desktop/extension usage.

## Decision
We will support two biometric unlock mechanisms:
1. **v1 (Standalone Extension):** Use the **WebAuthn PRF (Pseudo-Random Function) Extension**. This allows the browser extension to derive a hardware-backed symmetric key directly from a biometric authenticator (Touch ID/Windows Hello) without needing a companion app.
2. **v2 (Integrated Desktop):** Introduce **Native Messaging** support. When the Tauri desktop app is active, the extension will communicate with it via native messaging, enabling shared unlock states and delegating secure enclave operations to the desktop app.

## Consequences
- The v1 extension is fully self-contained and secure, not requiring a companion app for biometric unlock.
- Requires using modern browser WebAuthn PRF features.
- Prepares the extension architecture for native integration in v2.
