# 0004. OAuth Token Management Boundary

## Status
Accepted

## Context
Google OAuth requires interactive authentication flows (opening tabs, webviews, and handling redirects) and secure storage of credential tokens (access and refresh tokens). The implementation details of these operations differ drastically across the v1, v2, and v3 platform targets (Browser Extension, Tauri Desktop, React Native). Implementing this directly in `vault-core` would introduce heavy platform-specific dependencies into the Rust library.

## Decision
We will separate OAuth token management and storage from the sync engine:
1. **Platform Responsibility:** Each platform UI wrapper is responsible for initiating OAuth flows, capturing tokens, securely storing them (e.g. macOS Keychain, Android Keystore, Chrome secure storage), and refreshing tokens when expired.
2. **Core Responsibility:** `vault-core` defines a `TokenProvider` trait/interface. The sync engine accepts an implementation of this trait and uses it to obtain valid OAuth access tokens when calling the Google Drive API.

## Consequences
- Keeps `vault-core` lightweight, platform-agnostic, and easy to compile to WebAssembly and native FFI.
- Allows each platform to use standard, native SDKs and patterns for OAuth authentication and secure token storage.
- Simplifies testing of `vault-core` sync logic by mocking the `TokenProvider`.
