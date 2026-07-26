# KeyVault

Zero-knowledge, local-first password manager with a Rust cryptographic core and client-side Google Drive sync.

## Language

**Vault Item**:
The top-level secure item stored inside the vault, which can be either a Domain Group or a Secure Note.
_Avoid_: Vault Entry, login record, credential item

**Domain Group**:
A collection of one or more Accounts associated with a single domain or application. Match-grouped automatically upon new account creation using the URL's base domain (or item title if empty) against either the Domain Group's title or any of its associated URLs, excluding items in the Trash.
_Avoid_: Domain container, website group

**Account**:
A single credential set (username, password, TOTP secret, history, etc.) belonging to a Domain Group.
_Avoid_: Login credential, credentials

**Secure Note**:
A standalone secure text-based note stored as a Vault Item.
_Avoid_: Note entry, safe note

**Master Password**:
The user's secret passphrase used to derive the vault encryption key.
_Avoid_: Vault password, main password

**Vault Key**:
The symmetric AES-256-GCM key derived from the Master Password via Argon2id.
_Avoid_: Encryption key, master key

**Sync Merging**:
The client-side process of merging two divergent vault files based on entry UUIDs and update timestamps.
_Avoid_: Conflict resolution, sync overwrite

**Storage Provider**:
A modular service implementing authentication and read/write REST interactions for a cloud storage repository (e.g., Google Drive, OneDrive, iCloud) to store the encrypted vault payload.
_Avoid_: Sync client, Drive service, API wrapper.

**OAuth Client ID**:
The public identifier used to register and authenticate KeyVault with a cloud storage provider's API endpoints.
_Avoid_: App credentials, developer key.

**Custom Client ID**:
A user-specified OAuth Client ID that overrides the default developer-provided ID, enabling independent compilation and fork deployments to authorize cloud sync directly.
_Avoid_: User client ID, client ID override.

**Desktop OAuth Loopback**:
The desktop-specific OAuth authentication flow that runs a temporary local TCP listener on a loopback port (40305) to capture authorization codes from the system browser redirect.
_Avoid_: Local sync server, desktop redirect server.

**Soft Deletion**:
The action of moving an active Vault Item into the Trash, updating its update timestamp but keeping its full payload encrypted and intact for potential restoration. When the last Account of a Domain Group is deleted, the Domain Group is soft-deleted, retaining its Account payload to allow full restoration.
_Avoid_: Temp delete, delete to bin.

**Permanent Deletion (Purging)**:
A final action of destroying a Vault Item payload permanently, removing it from both active and trash collections, and writing a sync Tombstone.
_Avoid_: Hard delete, erase, format.

**Trash**:
The encrypted collection list within the Vault holding soft-deleted Vault Items awaiting user-triggered recovery or permanent deletion.
_Avoid_: Bin, deleted items list, recycle bin.

**Inline Autofill Overlay**:
An isolated iframe UI component loaded from the extension's secure origin and injected directly into target input fields on a webpage to present credentials.
_Avoid_: Injected dropdown, HTML autocomplete, float menu.

**Field Detector**:
The content script module that runs in the host webpage's context, responsible for detecting active input fields and positioning the Inline Autofill Overlay.
_Avoid_: DOM scanner, form parser.

**Autofill Badge**:
The small interactive icon injected into or placed adjacent to the input field, indicating that KeyVault has matching credentials.
_Avoid_: In-field icon, key logo, wand icon.

**Autofill Dropdown Panel**:
The floating iframe container that appears when the Autofill Badge is clicked, displaying the list of matched credentials.
_Avoid_: Suggestion list, popup menu, inline dropdown.

**Protected Action**:
An operation on a Vault Item or Account that exposes sensitive fields (such as viewing a password, copying a password, or triggering autofill) and requires explicit user verification before execution.
_Avoid_: Restricted action, secure operation.

**Re-authentication Mode**:
The verification method used to authorize a Protected Action, such as Master Password re-entry, Quick PIN, or Device Biometrics.
_Avoid_: Auth method, verification type.

**Auth Grace Period**:
A brief, configurable time window (e.g., 2 minutes) following a successful re-authentication during which the user can perform Protected Actions without being prompted again.
_Avoid_: Unlock timer, session duration.

**Remote Restore**:
The action of overwriting the local vault state entirely with the Google Drive cloud backup payload, discarding local changes.
_Avoid_: Local overwrite, cloud download.

**Remote Overwrite**:
The action of overwriting the Google Drive cloud backup completely with the local vault state, discarding cloud updates.
_Avoid_: Cloud overwrite, cloud backup upload.

**Cloud Wipe**:
The permanent deletion of the encrypted vault database file from the storage provider's remote servers during disconnection.
_Avoid_: Cloud clean, erase drive backup.




