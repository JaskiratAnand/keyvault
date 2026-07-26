# Chrome Web Store Listing — KeyVault

> Last Updated: 2026-07-03

## Store Listing

**Extension Name**  
KeyVault

**Short Description**  
Secure local-first password manager powered by Rust and WebAssembly, with safe Google Drive synchronization.

**Detailed Description**  
KeyVault is a secure, local-first password manager that puts you in complete control of your credentials. By leveraging a high-performance cryptographic core written in Rust and compiled directly to WebAssembly, KeyVault handles key derivation and database encryption locally on your machine with near-native efficiency.

Key Features:
- Strong Local Cryptography: Key derivation using Argon2id and authenticated AEAD encryption via AES-256-GCM, executing securely inside a compiled WebAssembly sandbox.
- Private Cloud Sync: Securely synchronize and merge your credentials across browsers using your personal Google Drive AppData folder. KeyVault never sees or stores your data.
- Flexible Password Generator: Generate secure character-based passwords, multi-word passphrases using the EFF wordlist, or numeric PINs with custom complexity rules.
- Safe Form Autofill: Speed up your logins by securely filling credentials directly into the active browser page.
- Cleartext JSON Export: Fully export your decrypted vault payload anytime to maintain complete data ownership.

How to Use:
1. Open KeyVault from the extension bar.
2. Initialize your vault by choosing a strong Master Password. This password is used to derive the local encryption keys and is never saved or transmitted.
3. Manage your logins under the Vault tab, or generate new credentials using the Generator.
4. Set up Google Drive sync under Settings to enable silent background merging and database backup.

Privacy & Security:
KeyVault operates under a zero-knowledge model. The extension does not collect analytics, track your web history, or communicate with developer-owned servers. All cryptographic operations occur strictly on your device.

**Category**  
Productivity

**Single Purpose**  
Locally generates, stores, and synchronizes secure credentials via private Google Drive folders.

**Primary Language**  
English

---

## Graphics & Assets

| Asset | Dimensions | Status | Filename / Path |
|-------|-----------|--------|-----------------|
| Store Icon [REQUIRED] | 128×128 PNG | ✅ Ready | `vault-extension/public/icon/128.png` |
| Screenshot 1 [REQUIRED] | 1280×800 or 640×400 | ⬜ Not created | (To be generated: Vault list/search view) |
| Screenshot 2 [RECOMMENDED] | 1280×800 or 640×400 | ⬜ Not created | (To be generated: Password generator sliders) |
| Screenshot 3 [RECOMMENDED] | 1280×800 or 640×400 | ⬜ Not created | (To be generated: Sync connection panel) |

### Screenshot Notes
- **Screenshot 1 (Vault list)**: Showcases search filter, entries, and the clean master lock indicator.
- **Screenshot 2 (Generator)**: Shows Svelte range sliders, custom passphrase word counts, and reactive entropy estimations.
- **Screenshot 3 (Settings)**: Shows the sync dashboard displaying connected account emails, reset options, and backup configurations.

---

## Permissions Justification

| Permission | Type | Justification |
|------------|------|---------------|
| `storage` | permissions | Used to persist the Master Password salt and the locally encrypted vault database payload locally inside the extension sandbox. |
| `activeTab` | permissions | Grants temporary, secure permission to the current active tab only when the user clicks the "Autofill on Page" button. |
| `scripting` | permissions | Used to inject a helper script into the active page DOM to locate form elements and fill credential values. |
| `identity` | permissions | Used to authorize the user and manage access tokens for Google Drive vault synchronization. |

---

## Privacy & Data Use

### Data Collection

**Does the extension collect user data?** Yes (processed locally and transmitted to the user's private Google Drive folder for sync; no developer servers or third parties are involved).

| Data Type | Collected? | Transmitted Off-Device? | Purpose | Shared with Third Parties? |
|-----------|-----------|------------------------|---------|---------------------------|
| Authentication info | Yes | Yes (to Google APIs) | To log in and sync credentials | No |
| Website content | Yes | Yes (to Google Drive) | To back up the encrypted vault | No |

### Data Use Certification
- [x] Data is NOT sold to third parties
- [x] Data is NOT used for purposes unrelated to the extension's core functionality
- [x] Data is NOT used for creditworthiness or lending purposes

---

## Privacy Policy

**Privacy Policy URL**  
`https://github.com/[username]/passwd-manager/blob/main/PRIVACY.md` *(To be hosted live upon submission)*

---

## Distribution

**Visibility**: Public  
**Regions**: All regions  
**Pricing**: Free  

---

## Developer Info

**Publisher Name**: KeyVault Developer  
**Contact Email**: developer@keyvault.local  
**Support URL**: `https://github.com/[username]/passwd-manager/issues`  

---

## Version History

| Version | Date | Changes | Status |
|---------|------|---------|--------|
| 1.0.0 | 2026-07-03 | Initial release featuring Rust/WASM core, Google Drive sync, and Svelte UI. | Draft |

---

## Review Notes

### Known Issues / Limitations
- WebAssembly requires `'wasm-unsafe-eval'` CSP configuration in Chrome Manifest V3, which may prompt automated scanning alerts. The Wasm engine is bundled locally.
- Google Drive synchronization requires setting up a Google Cloud Console OAuth consent screen under the matching Chrome Web Store extension ID.
