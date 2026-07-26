# 0009. Autofill Phishing Prevention

## Status
Accepted

## Context
Automated credential autofilling can be exploited by malicious pages using invisible/hidden forms or nested cross-origin iframes to extract credentials without the user's knowledge. We need to define strict browser extension heuristics to prevent these credential-harvesting attacks.

## Decision
We will enforce the following security rules in the browser extension's content scripts during autofill:
1. **User Action Bound:** Autofilling is strictly event-driven and will only occur when the user explicitly triggers it (e.g. clicking the entry in the popup or using the keyboard shortcut `Ctrl+Shift+L`). There will be no automatic autofill on page load.
2. **Visibility Check:** Only input elements that are visible (i.e. not `type="hidden"`, not styled with `display: none` or `visibility: hidden`, and having non-zero width/height) will be eligible for autofilling.
3. **Origin Isolation:** Injected scripts will only fill inputs inside frames (including iframes) whose active document origin matches the domain of the selected credential.

## Consequences
- Protects users from hidden forms and iframe-based credential-harvesting techniques.
- Limits autofill compatibility with websites that use highly non-standard hidden inputs for authentication (a reasonable security trade-off).
