# KeyVault Versioning & Release Guide

KeyVault uses an **Independent Package Versioning** strategy based on [Semantic Versioning (SemVer 2.0.0)](https://semver.org/).

Because KeyVault is a monorepo containing distinct distribution targets (Chrome Extension, Desktop App, and Rust Core Engine), each package tracks its version number independently. This allows you to release urgent desktop patches or core updates without needing to bump or resubmit the browser extension to the Chrome Web Store.

---

## Package Version Summary & Manifest Locations

| Package | Initial Version | Tag Pattern | Config Files Updated |
| :--- | :--- | :--- | :--- |
| **`vault-extension`** | `0.1.0` | `extension-v*` | `vault-extension/package.json` |
| **`vault-desktop`** | `0.1.0` | `desktop-v*` | `vault-desktop/package.json`<br>`vault-desktop/src-tauri/tauri.conf.json`<br>`vault-desktop/src-tauri/Cargo.toml` |
| **`vault-core`** | `0.1.0` | `core-v*` | `vault-core/Cargo.toml` |

---

## Version Bumping CLI Commands

We provide automated helper scripts to bump version strings cleanly across all associated package manifests in a single command.

Run these commands from the root directory of the repository:

### 1. Bump Extension Version
```bash
pnpm run bump:extension <new_version>

# Example:
pnpm run bump:extension 0.1.1
```

### 2. Bump Desktop Version
```bash
pnpm run bump:desktop <new_version>

# Example:
pnpm run bump:desktop 0.1.1
```

### 3. Bump Core Rust Library Version
```bash
pnpm run bump:core <new_version>

# Example:
pnpm run bump:core 0.1.1
```

---

## Publishing a Release to GitHub

Whenever you push a package-specific version tag to GitHub, automated GitHub Actions workflows build the binary artifacts and publish an official GitHub Release automatically.

### Step-by-Step Release Workflow

#### A. Releasing a Browser Extension Update (`extension-v*`)

1. **Bump Version:**
   ```bash
   pnpm run bump:extension 0.1.1
   ```
2. **Commit & Tag:**
   ```bash
   git add .
   git commit -m "chore(extension): bump version to 0.1.1"
   git tag extension-v0.1.1
   ```
3. **Push Tag to GitHub:**
   ```bash
   git push origin main
   git push origin extension-v0.1.1
   ```
4. **What Happens:**
   GitHub Actions automatically compiles `vault-core` to WebAssembly, packages the browser extension into `.output/vault-extension-0.1.1-chrome.zip`, and creates a new GitHub Release with the `.zip` attached.

---

#### B. Releasing a Desktop Application Update (`desktop-v*`)

1. **Bump Version:**
   ```bash
   pnpm run bump:desktop 0.1.1
   ```
2. **Commit & Tag:**
   ```bash
   git add .
   git commit -m "chore(desktop): bump version to 0.1.1"
   git tag desktop-v0.1.1
   ```
3. **Push Tag to GitHub:**
   ```bash
   git push origin main
   git push origin desktop-v0.1.1
   ```
4. **What Happens:**
   GitHub Actions spins up macOS and Windows cloud build runners, builds the Tauri v2 desktop app, and attaches `.dmg` (macOS), `.msi` (Windows), and `.exe` (Windows) installers to a new GitHub Release.

---

#### C. Releasing a Core Engine Update (`core-v*`)

1. **Bump Version:**
   ```bash
   pnpm run bump:core 0.1.1
   ```
2. **Commit & Tag:**
   ```bash
   git add .
   git commit -m "chore(core): bump version to 0.1.1"
   git tag core-v0.1.1
   git push origin main
   git push origin core-v0.1.1
   ```

---

## Semantic Versioning Rules (SemVer)

- **MAJOR (`X.0.0`)**: Incompatible API or breaking storage schema changes.
- **MINOR (`0.X.0`)**: Backward-compatible new features (e.g. adding TOTP generator, passkey support).
- **PATCH (`0.0.X`)**: Backward-compatible bug fixes or security patches.
