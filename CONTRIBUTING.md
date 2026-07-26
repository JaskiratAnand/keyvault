# Contributing to KeyVault

Thank you for your interest in contributing to KeyVault! KeyVault is an open-source, zero-knowledge, local-first password manager built with Rust, WebAssembly, Svelte, WXT, and Tauri.

We welcome contributions of all kinds: bug reports, feature requests, documentation improvements, code contributions, and security audits.

---

## Workspace Structure

The repository is structured as a monorepo:

- **[`vault-core`](file:///Users/jas/Code/passwd-manager/vault-core)**: Core Rust library implementing Argon2id key derivation, AES-256-GCM encryption, tombstone sync merging, and WASM bindings (Dual MIT/Apache-2.0).
- **[`vault-extension`](file:///Users/jas/Code/passwd-manager/vault-extension)**: Browser extension built with Svelte, TypeScript, WXT framework, and WASM (AGPL-3.0).
- **[`vault-desktop`](file:///Users/jas/Code/passwd-manager/vault-desktop)**: Desktop app built with Tauri v2, Svelte, TypeScript, and Rust native bindings (AGPL-3.0).
- **[`vault-benchmark`](file:///Users/jas/Code/passwd-manager/vault-benchmark)**: Web Worker performance benchmark suite comparing WASM, Web Crypto, and JS implementations.

---

## Local Development Setup

### Prerequisites

Make sure you have the following installed on your machine:
- [Rust](https://www.rust-lang.org/tools/install) (Edition 2024 / stable toolchain)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- [Node.js](https://nodejs.org/) (v18+ or v20+)
- [pnpm](https://pnpm.io/) (`npm install -g pnpm`)
- For Desktop development: Platform dependencies for [Tauri v2](https://v2.tauri.app/start/prerequisites/)

### 1. Build the Rust WASM Core

Before running the extension or benchmark suite, compile the Rust core into WebAssembly:

```bash
cd vault-core
wasm-pack build --target web
```

To run Rust unit tests:
```bash
cargo test
```

### 2. Set Up Environment Variables (Google Drive Sync)

If you plan to work on Google Drive sync features in the extension or desktop app:

1. Copy `.env.example` to `.env` in the target package directory:
   ```bash
   cp vault-extension/.env.example vault-extension/.env
   # or
   cp vault-desktop/.env.example vault-desktop/.env
   ```
2. Create a project in [Google Cloud Console](https://console.cloud.google.com/).
3. Enable the **Google Drive API** under **APIs & Services > Library**.
4. Create **OAuth 2.0 Client IDs** (Chrome App / Extension for browser extension; Desktop App for desktop app).
5. Paste your `VITE_GDRIVE_CLIENT_ID` into `vault-extension/.env` or `vault-desktop/.env`.

### 3. Run the Browser Extension

```bash
cd vault-extension
pnpm install
pnpm run dev
```
To run tests and linters:
```bash
pnpm test
pnpm run check
pnpm run lint
```

### 4. Run the Desktop Application

```bash
cd vault-desktop
pnpm install
pnpm run tauri:dev
```

---

## Code Quality & Standards

- **Rust**: Format with `cargo fmt` and verify clean execution with `cargo clippy`.
- **TypeScript & Svelte**: Format and check with `biome` and `oxlint`:
  ```bash
  pnpm run fmt
  pnpm run lint
  pnpm run check
  ```
- **Commit Messages**: Write clear, descriptive commit messages focusing on what changed and why.

---

## Submitting Pull Requests

1. **Fork the Repository**: Create a personal fork on GitHub.
2. **Create a Feature Branch**: `git checkout -b feature/my-cool-feature` or `fix/issue-description`.
3. **Keep Commits Clean**: Ensure tests pass and no temporary debug code or secret `.env` files are committed.
4. **Open a PR**: Open a pull request against `main` on the primary repository. Provide a clear description of the change and link any relevant GitHub Issues.

---

## Licensing & Contributions

KeyVault operates under a **Hybrid License Model**:
- **`vault-core`**: Dual-licensed under **MIT** OR **Apache-2.0**.
- **`vault-extension` & `vault-desktop`**: Licensed under **AGPL-3.0**.

By submitting a pull request to this repository, you agree that your contributions to `vault-core` will be licensed under MIT/Apache-2.0 and contributions to client applications will be licensed under AGPL-3.0.
