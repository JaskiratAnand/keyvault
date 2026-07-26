# 0016. Hierarchical Vault Storage

## Status
Accepted

## Context
KeyVault originally stored all credentials in a flat list of `VaultEntry` records. While simple, this does not match the Google Password Manager or Chrome-like UX where multiple credentials for a single domain are grouped together under a single domain card. Users want to see a list of domains on the primary screen and drill down to view or copy specific credentials.

## Decision
We will change the database structure in the Rust core to be hierarchical rather than flat:
1.  Define a top-level `VaultItem` enum representing either a `DomainGroup` (grouped by domain) or a `SecureNote`.
2.  Each `DomainGroup` has a list of sub-structs called `Account` which store the actual username, password, TOTP secrets, notes, and password history.
3.  To ensure robust client-side sync merging, each `Account` will carry its own unique UUID and `updated_at` timestamp.
4.  Deletions will be tracked via `Tombstones` both at the group level (if a whole group/note is deleted) and at the account level (if a single account within a group is deleted).
5.  CSV imports and exports will be mapped: flat CSV files are dynamically grouped by base domain on import, and nested credentials are expanded into flat CSV rows on export.
6.  Backwards Compatibility & Robustness: To handle legacy vaults or empty vault payloads gracefully without requiring explicit migration scripts, we configure deserialization to be permissive. The `items`, `tombstones`, and `trash` fields in the `Vault` struct are marked with `#[serde(default)]`, allowing the deserializer to fallback to empty collections if these fields are absent in the payload.

## Consequences
- Requires updating the Rust data models, deserialization/serialization code, and synchronization merge logic in `vault-core`.
- Requires updating the TypeScript bindings and state manager in the frontend.
- Requires redesigned UI panels in both the popup and the options page to present a domain-first overview and a drill-down detail view for nested accounts.
- Modifies inline autofill dropdown matching to query matching groups and display all nested accounts.
