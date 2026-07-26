# 0010. Secret Zeroization in JS/WASM

## Status
Accepted

## Context
While the Rust cryptographic core uses the `zeroize` crate to safely overwrite memory holding cryptographic keys and raw passwords when they are dropped, the JavaScript environment uses automated memory management. JavaScript strings are immutable and cannot be zeroed, exposing secrets to memory-dump extraction attacks if they persist in the V8 heap.

## Decision
We will enforce the following guidelines at the WASM-to-JavaScript boundary:
1. **Typed Array Transfers:** Critical inputs like the master password, recovery codes, and cryptographic keys must be passed across the WASM boundary using mutable `Uint8Array` buffers instead of JavaScript strings.
2. **Immediate Clearing:** JavaScript wrappers must zero out these `Uint8Array` buffers (using `array.fill(0)`) immediately after passing them to or receiving them from WASM.
3. **No JS Crypto:** Cryptographic key wrapping, decryption, and key derivation must occur exclusively within the Rust WASM memory space.
4. **Transient UI Strings:** Plaintext passwords retrieved for the UI or clipboard copy must be kept as short-lived variables and dereferenced immediately to allow prompt garbage collection.

## Consequences
- Mitigates the risk of sensitive keys remaining in browser heap memory dumps.
- Increases the complexity of FFI code, requiring manual buffer allocation and zeroing in TypeScript.
