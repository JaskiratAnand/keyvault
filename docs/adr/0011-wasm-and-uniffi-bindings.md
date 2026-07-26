# 0011. WASM and UniFFI Bindings

## Status
Accepted

## Context
KeyVault targets multiple platforms (Browser Extension in v1, Tauri Desktop in v2, iOS/Android Mobile in v3). Re-implementing the cryptographic and sync logic in JavaScript, Swift, and Kotlin would lead to security audits across three separate codebases and likely result in diverging behavior. We need a way to build the core once in Rust and distribute it safely to all target environments.

## Decision
We will expose the `vault-core` Rust library using two binding generators:
1. **`wasm-bindgen`:** Used to compile the core into a WebAssembly (WASM) module for the v1 Browser Extension popup and background scripts.
2. **`uniffi`:** Used to generate Swift bindings (for iOS) and Kotlin bindings (for Android) for the v3 mobile app.
For the v2 Tauri app, we will link the Rust crate directly as a native dependency using Tauri command handlers.

## Consequences
- Single source of truth for cryptography, sync, and matching logic.
- Reduced surface area for security audits.
- Eliminates platform-specific implementation bugs.
- Builds require compilation tooling for WASM and native mobile libraries.
