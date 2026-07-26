# 0011. Re-authentication for Protected Actions

## Status
Accepted

## Context
When the vault is unlocked, the decrypted credentials are held in memory. If the user leaves their device unattended while the extension is unlocked, another person could view, copy, or autofill passwords. To protect sensitive credentials, we need a mechanism to enforce re-authentication before exposing password data.

## Decision
We will implement re-authentication for Protected Actions using the following architectural design:

1. **Re-authentication Mode**:
   We will support **Device Biometrics** (Touch ID, Windows Hello) as the primary verification method, utilizing the browser's WebAuthn API. We will use **Master Password Re-entry** as a fallback if biometrics are unavailable, fail, or are cancelled.

2. **Active View Grace Period**:
   Re-authentication remains valid only for the duration that the current view (the popup window or options tab) remains open. Closing the popup or navigating away resets the authorization state immediately.

3. **Verification Scope**:
   Re-authentication is required for all **Protected Actions**:
   *   Revealing a password (clicking the "eye" icon)
   *   Copying a password to the clipboard
   *   Autofilling credentials into a webpage (from the popup or the inline dropdown)

4. **Biometric Settings Page Setup**:
   An explicit setting toggle will be added to the options settings page: *"Require Touch ID / Windows Hello to reveal/copy passwords"*. Activating this toggle registers a local WebAuthn platform credential under the extension's origin.

5. **Direct Biometric UI Trigger**:
   Triggering a Protected Action will immediately fire the OS-level biometric prompt. If biometrics are not configured or are cancelled/fail, the extension will display a fallback modal dialog requesting the Master Password.

6. **Local Verification via Promise Resolution**:
   The biometric check is verified locally by the successful resolution of the `navigator.credentials.get()` promise.

7. **Dropdown Fallback inside Iframe**:
   If the biometric prompt is blocked or fails inside the cross-origin Inline Autofill Overlay iframe, the iframe will display an inline Master Password input prompt to authorize the autofill action safely within the extension context.

## Consequences
- **Security**: Significantly reduces risk of physical compromise by ensuring that unlocked extensions still require user verification to expose or inject passwords.
- **UX**: Retains a low-friction workflow on devices with Touch ID/Windows Hello, with a secure fallback for other systems.
- **Portability**: Works across macOS, Windows, and Linux fallback environments.
