# Security Policy

## Security Overview

KeyVault is built with a zero-knowledge architecture. Security and user privacy are foundational design principles of this workspace.

### Cryptographic Foundation (`vault-core`)
- **Key Derivation**: Argon2id (memory-hard password hashing algorithm).
- **Symmetric Encryption**: AES-256-GCM with un-reused, cryptographically random nonces.
- **Zeroization**: Sensitive memory regions holding keys, unencrypted passwords, and intermediate derivation states derive from `zeroize::Zeroize` to ensure immediate wiping upon drop.
- **Local-First Architecture**: Cryptographic operations occur strictly client-side within the compiled WebAssembly module or native Rust core. Master passwords and unencrypted vault payloads are never sent to external servers or cloud storage providers.

---

## Reporting Vulnerabilities

We take security issues seriously. If you discover a security vulnerability in KeyVault (such as cryptographic flaws, memory safety issues, extension injection risks, or credential leakage), please **do not report it publicly via GitHub Issues**.

### Preferred Reporting Channel
- **GitHub Private Vulnerability Reporting**: Submit a report directly via the repository's **Security > Advisories > Report a vulnerability** tab.
- **Email Contact**: If you prefer direct email communication, please send your report to `security@keyvault.app` (or contact the repository maintainers via GitHub profile security contact).

### What to Include in Your Report
1. Detailed description of the vulnerability.
2. Steps to reproduce or proof-of-concept (PoC) exploit code.
3. Affected components (`vault-core`, `vault-extension`, or `vault-desktop`).
4. Any proposed fix or mitigation suggestions.

### Disclosure Process
- **Acknowledgement**: We will acknowledge receipt of your vulnerability report within **48 hours**.
- **Assessment & Patching**: We will assess the severity and develop a fix in a private advisory.
- **Public Disclosure**: Once a security patch is released across releases, we will publish a GitHub Security Advisory crediting your discovery (unless requested otherwise).
