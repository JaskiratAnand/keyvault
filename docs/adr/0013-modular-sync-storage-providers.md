# 0013. Modular Sync Storage Providers

## Status
Accepted

## Context
We need to support multiple backup and synchronization targets (starting with Google Drive, with future expansion to Microsoft OneDrive, Apple iCloud, etc.). Implementing each target directly in the state store or inside the Rust WASM core would lead to coupling, bloating, and code duplication.

## Decision
We will define a clean, abstract TypeScript interface `StorageProvider` under `src/lib/sync/types.ts` and use a factory registry to swap between implementations. 
Key design components:
1. **Separation of Concerns**: JavaScript handles browser OAuth boundaries and HTTP REST request loops; the Rust WASM core handles cryptography and merge reconciliation.
2. **Explicit Metadata Wrapper**: Provider methods return and consume an explicit metadata object (containing ETag and last-modified fields) to allow the sync controller to handle skip-caching uniformly.
3. **Encapsulated Authentication**: Authentication states, tokens, and credential storage are managed internally by each provider class, exposing only `signIn()` and `isAuthenticated()` bounds.

## Consequences
- Clean, decoupled codebase allowing new storage providers to be added by writing a single adapter class and registering it in the factory registry.
- Clear separation between UI state management, cloud storage communication, and secure cryptography.
- Restricting token details to provider files ensures zero exposure of API scopes to the core state store.
