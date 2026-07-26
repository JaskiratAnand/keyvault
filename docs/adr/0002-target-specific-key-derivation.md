# 0002. Target-Specific Key Derivation

## Status
Accepted

## Context
Argon2id is memory and CPU intensive. In browser extensions (Manifest V3), the JavaScript/WASM environment is single-threaded by default because enabling multi-threading via `SharedArrayBuffer` requires strict COOP/COEP headers that are difficult to guarantee. Running high-parallelism parameters (`p=4`) sequentially in single-threaded WASM causes unnecessary overhead and blocks the UI thread or extension service worker.

## Decision
We will use target-specific Argon2id profiles for key derivation:
- **WASM / Browser Extension:** Set parallelism `p=1` to align with the single-threaded environment, offloading key derivation to a Web Worker to keep the UI/Service Worker responsive.
- **Native / Tauri & Mobile:** Set parallelism `p=4` to leverage native multi-threading.
The parameters used to derive a vault's key will be stored in the vault metadata so the decrypting client knows which parameters to use.

## Consequences
- Prevents UI freezing in the browser extension during unlock.
- Avoids complexity and security headers issue of `SharedArrayBuffer` in extensions.
- Native apps still get maximum brute-force resistance using multi-core processors.
