# KeyVault

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![Rust](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org/)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-WASM-purple.svg)](https://webassembly.org/)
[![Tauri](https://img.shields.io/badge/Tauri-v2-blue.svg)](https://v2.tauri.app/)
[![Svelte](https://img.shields.io/badge/Svelte-v5-red.svg)](https://svelte.dev/)

Zero-knowledge, local-first password manager workspace powered by a high-performance **Rust cryptographic core**, compiled to **WebAssembly** for client-side browser extensions and native **Tauri v2** for desktop platforms.

---

## Key Features

- **Zero-Knowledge Architecture**: All key derivation and encryption occur strictly on your local device. Your master password and vault contents never leave your machine unencrypted.
- **Local-First & Multi-Platform**:
  - **Browser Extension**: Built with Svelte 5, TypeScript, and WXT.
  - **Desktop App**: Native desktop application built with Tauri v2 and Svelte.
  - **Google Drive Sync**: Encrypted vault synchronization over Google Drive API (`drive.file` / `appdata` scope) with tombstone-based CRDT merge resolution.
- **Cryptographic Rigor**:
  - **Argon2id** password key derivation.
  - **AES-256-GCM** authenticated symmetric encryption with unique nonces.
  - **Zeroization**: Secure memory wiping for plaintext keys and secrets on drop.

---

## Installation

### 🖥️ Desktop Application

#### **macOS**

**Homebrew (Recommended):**

Installing via Homebrew automatically bypasses macOS Gatekeeper quarantine warnings:

```bash
brew tap JaskiratAnand/tap
brew trust JaskiratAnand/tap
brew install --cask keyvault
```

**Download .dmg:**

Download [`KeyVault_aarch64.dmg`](https://github.com/JaskiratAnand/keyvault/releases?q=desktop) (Apple Silicon `arm64`) and drag `KeyVault.app` to `/Applications`.

> **📌 macOS Gatekeeper Note**: If macOS displays `"KeyVault is damaged and can't be opened"`, open Terminal and run: `xattr -cr /Applications/KeyVault.app`

> **Note on macOS Intel (x64)**: Pre-built binaries for Intel-based Macs (`x64`) are not available. Intel Mac users can build the application locally from source (see [Quick Start](#quick-start-development)).*  

#### **Windows & Direct Downloads**
Download pre-built installers from [Desktop Releases](https://github.com/JaskiratAnand/keyvault/releases?q=desktop):

- **Windows**: Download [`KeyVault_x64-setup.exe`](https://github.com/JaskiratAnand/keyvault/releases?q=desktop-v) and run the installer.  
  *(If Windows SmartScreen appears: Click **More info** $\rightarrow$ **Run anyway**).*

---

### **🧩 Chrome Browser Extension**

1. Download the latest `keyvault-extension-v*.zip` from [Extension Releases](https://github.com/JaskiratAnand/keyvault/releases?q=extension).
2. Unzip the downloaded file into a folder on your computer.
3. Open Chrome (or Chromium-based browsers like Edge, Brave, or Arc) and go to `chrome://extensions`.
4. Enable **Developer mode** (toggle in the top-right corner).
5. Click **Load unpacked** and select the extracted folder.

---

## Repository Architecture

```
.
├── vault-core/        # Rust cryptographic engine (Argon2id, AES-256-GCM, WASM bindings)
├── vault-extension/   # Svelte 5 + WXT browser extension (Chrome / Firefox)
└── vault-desktop/     # Tauri v2 + Svelte 5 desktop app (macOS / Windows / Linux)
```

---

## Quick Start (Development)

### 1. Build the Rust WASM Core

Ensure [Rust](https://www.rust-lang.org/tools/install) and [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) are installed:

```bash
cd vault-core
wasm-pack build --target web
cargo test
```

### 2. Browser Extension

```bash
cd vault-extension
pnpm install
cp .env.example .env  # Add your Google Cloud OAuth Client ID if testing Google Drive sync
pnpm run dev
```

### 3. Desktop Application

```bash
cd vault-desktop
pnpm install
cp .env.example .env
pnpm run tauri:dev
```

---

## Security Policy

Security reports are taken very seriously. Please read our [SECURITY.md](SECURITY.md) for vulnerability disclosure guidelines via **GitHub Private Vulnerability Reporting**.

---

## Versioning & Releases

KeyVault follows Independent Semantic Versioning for all workspace packages (`vault-core`, `vault-extension`, `vault-desktop`). For version bump CLI commands and GitHub release instructions, see [VERSIONING.md](docs/VERSIONING.md).

---

## Contributing

We welcome community contributions! Please review [CONTRIBUTING.md](CONTRIBUTING.md) for environment setup, code formatting standards, and PR procedures, as well as our [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

---

## Licensing

KeyVault uses a **Hybrid Open-Source License**:

- **[`vault-core`](vault-core)**: Dual-licensed under [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE).
- **[`vault-extension`](vault-extension)** and **[`vault-desktop`](vault-desktop)**: Licensed under [AGPL-3.0](LICENSE-AGPL).
