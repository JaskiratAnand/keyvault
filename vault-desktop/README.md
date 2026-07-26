# KeyVault Desktop

KeyVault Desktop is a zero-knowledge, local-first secure password manager shell built using **Tauri v2**, **Svelte 5**, **TypeScript**, and **Tailwind CSS v4**. It interfaces natively with the `vault-core` Rust cryptographic library for secure, audited local data storage.

---

## 🛠️ Prerequisites

Ensure you have the following installed on your system before running or building the app:

### Common Dependencies
- **Node.js**: Version 18+ (LTS recommended)
- **pnpm**: Version 9+ (Package Manager)
- **Rust Toolchain**: [Rustup](https://rustup.rs/) (Tauri's native backend driver/compiler)

### Windows-Specific Dependencies
To compile and package the native installer on Windows:
1. **C++ Build Tools**: Run the [Visual Studio Installer](https://visualstudio.microsoft.com/downloads/) and check the **"Desktop development with C++"** workload (installs `cl.exe` and MSVC SDKs).
2. **NSIS (Nullsoft Scriptable Install System)** (Recommended): Used by default in this project to compile `.exe` installers. It requires no additional system dependencies.
3. **WiX Toolset v3** (Optional): Only required if you configure the build targets in `tauri.conf.json` to compile `.msi` installers. If needed, you can install it using:
   ```powershell
   winget install WiXToolset.WiXToolset
   ```
   *Note: WiX v3 requires administrator privileges to install and depends on the `.NET Framework 3.5 (NetFx3)` Windows feature.*

### Linux-Specific Dependencies (Ubuntu/Debian)
To compile and build packages on Linux:
- Install the core compilers and WebKitGTK headers via `apt`:
  ```bash
  sudo apt update
  sudo apt install -y \
    libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
  ```

---

## 🚀 Development Commands

From the `vault-desktop` directory, you can run the following package scripts:

### Running the Dev Environment
Starts the Vite dev server with Hot Module Replacement (HMR) and boots the native Tauri window:
```bash
pnpm tauri dev
# or
pnpm tauri:dev
```

### Building the Production Application
To compile the frontend assets and package them into a native installer, run the build script from your target OS:
```bash
pnpm tauri build
```

#### 🍎 macOS (Output: `.app`, `.dmg`)
- Run the build command on macOS. It utilizes native Xcode Command Line Tools automatically.
- Output files are generated in `src-tauri/target/release/bundle/dmg/KeyVault_0.1.0_aarch64.dmg` (or `_x64.dmg` when compiling on Intel Macs).

#### 🪟 Windows (Output: `.exe` / `.msi`)
- Run the build command on Windows (requires C++ Build Tools).
- The default setup targets **NSIS** and outputs a standalone installer at `src-tauri/target/release/bundle/nsis/KeyVault_0.1.0_x64-setup.exe`.
- (Optional) If WiX is configured, `.msi` output will be generated in `src-tauri/target/release/bundle/msi/KeyVault_0.1.0_x64_en-US.msi`.

#### 🐧 Linux (Output: `.deb`, `.AppImage`)
- Run the build command on Linux (requires WebKitGTK system headers installed).
- Output files are generated in:
  - `src-tauri/target/release/bundle/deb/keyvault_0.1.0_amd64.deb` (Ubuntu/Debian package)
  - `src-tauri/target/release/bundle/appimage/keyvault_0.1.0_amd64.AppImage` (Universal AppImage)

---

## 🧼 Code Quality & Formatting

The desktop codebase is integrated with **Biome.js** and **oxlint** for blazing-fast linting and formatting, and **svelte-check** for TypeScript verification:

- **Typecheck**: Verify TypeScript compile states and Svelte component bindings:
  ```bash
  pnpm run check
  ```
- **Lint**: Run static code analysis checks across scripts, styles, and configurations:
  ```bash
  pnpm run lint
  ```
- **Format**: Run Biome formatter to automatically write standard code layout:
  ```bash
  pnpm run fmt
  ```

---

## 🔐 Onboarding & Vault Reset

KeyVault Desktop runs fully client-side and derives 256-bit encryption keys directly from your Master Password.

- **Initial Setup**: On first launch (when no local vault exists), the app guides you to **Setup Master Password** and requires typing it twice to prevent password typos.
- **Wiping/Resetting Vault**: If you need to clear the local database or forgot your Master Password during testing, click the red **"Forgot Master Password? Reset Vault"** button at the bottom of the Lock Screen. This opens a custom, WebView-safe confirmation modal to wipe the local file and let you initialize a fresh vault.

---

## 🧪 Testing

The project has two independent test suites — one for the Rust backend and one for the Svelte frontend.

### Frontend Tests (Vitest + JSDOM)

Runs all `*.test.ts` / `*.spec.ts` files in `src/` using a simulated browser DOM. Tauri IPC (`window.__TAURI__`) is mocked via `vi.fn()` — no real Tauri backend required.

```bash
# Run all frontend tests once
pnpm test

# Watch mode (re-runs on file changes)
pnpm exec vitest

# Run a single test file
pnpm exec vitest src/lib/vault-state.test.ts

# Verbose output (show individual test names)
pnpm exec vitest run --reporter=verbose
```

### Rust Backend Tests (cargo test)

Runs all `#[test]` functions inside `src-tauri/src/` using the `tauri::test` mock runtime. Tests are fully parallelized and use thread-local file path overrides for isolation — no real `AppLocalData` directory is written to.

```bash
# Standard run (from repo root or vault-desktop/src-tauri/)
cargo test -p vault-desktop

# macOS — must link Swift runtime explicitly
RUSTFLAGS="-C link-arg=-Wl,-rpath,/usr/lib/swift" cargo test -p vault-desktop

# Run a specific test by name (substring match)
RUSTFLAGS="-C link-arg=-Wl,-rpath,/usr/lib/swift" cargo test -p vault-desktop test_unlock_new_vault

# Show println!/dbg! output even for passing tests
RUSTFLAGS="-C link-arg=-Wl,-rpath,/usr/lib/swift" cargo test -p vault-desktop -- --nocapture
```

### Run All Tests

```bash
# Frontend
pnpm test

# Backend (macOS)
RUSTFLAGS="-C link-arg=-Wl,-rpath,/usr/lib/swift" cargo test -p vault-desktop
```

### Test Coverage Summary

| Module | Source File | Tests |
|---|---|---|
| VaultSession (unlock / lock / CRUD) | `src-tauri/src/vault_session.rs` | 7 |
| Crypto commands (credential gen, TOTP, recovery) | `src-tauri/src/commands/crypto.rs` | 3 |
| Sync commands (merge, overwrite, decrypt) | `src-tauri/src/commands/sync.rs` | 3 |
| Frontend state (filteredItems, allTags, unlock, lock) | `src/lib/vault-state.test.ts` | 20 |
| **Total** | | **33** |

---

## 📁 Project Structure

```
vault-desktop/
├── src/                            # Svelte frontend
│   ├── lib/
│   │   ├── vault-state.svelte.ts   # Central reactive state (VaultState class)
│   │   ├── vault-state.test.ts     # Frontend unit tests (Vitest)
│   │   ├── tauri-ipc.ts            # Typed Tauri invoke wrapper + error helpers
│   │   ├── types.ts                # Shared TypeScript types (Vault, VaultItem, etc.)
│   │   └── sync/                   # Google Drive sync provider
│   ├── components/                 # UI components
│   └── App.svelte                  # Root Svelte component
├── src-tauri/                      # Rust / Tauri backend
│   ├── src/
│   │   ├── main.rs                 # App entrypoint + Tauri command registration
│   │   ├── vault_session.rs        # In-memory vault session + 7 unit tests
│   │   ├── storage.rs              # Encrypted file I/O + test path override
│   │   ├── error.rs                # AppError enum (thiserror)
│   │   └── commands/
│   │       ├── crypto.rs           # Credential gen, TOTP, recovery key + 3 tests
│   │       ├── sync.rs             # Merge, overwrite, remote decrypt + 3 tests
│   │       └── vault.rs            # Vault CRUD IPC commands
│   └── Cargo.toml
├── vite.config.ts                  # Vite build + Vitest test configuration
├── package.json                    # Scripts and dependencies
└── README.md                       # This file
```

