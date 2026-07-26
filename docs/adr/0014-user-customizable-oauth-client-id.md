# 0014. User Customizable OAuth Client ID

## Status
Accepted

## Context
KeyVault is designed to be an open-source, local-first, decentralized password manager. To support secure cloud synchronization, it connects directly to the user's personal Google Drive folder. However, accessing Google's Drive APIs requires registering an OAuth Client ID in the Google Cloud Console.

Hardcoding only a single developer-owned Client ID would prevent users who fork or build custom releases of KeyVault from using the Sync feature, as Google restricts authentication requests to matching Extension IDs. Users would be forced to modify the codebase to use their own credentials.

## Decision
We will ship KeyVault with a default pre-registered Client ID for immediate convenience in official builds. Additionally, we will introduce a "Custom Google Client ID" configuration field in the Settings tab of the extension options.

If a user specifies a custom Client ID, the extension will save it in local storage (`browser.storage.local`) and use it dynamically at runtime to request OAuth tokens instead of the default hardcoded Client ID.

## Consequences
- Open-source users and forks can fully deploy their own custom builds of KeyVault and sync with Google Drive by inputting their own Google Cloud Client ID.
- Aligns with the project's decentralized principles by removing dependency on the developer's Google Cloud project registration.
- Requires saving and retrieving the custom Client ID dynamically inside the `GoogleDriveProvider` class.
