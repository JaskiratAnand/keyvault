# 0010. Inline Autofill Architecture

## Status
Accepted

## Context
Google Chrome does not offer public APIs for third-party extensions to inject suggestions directly into its native autofill dropdown. To provide a seamless, native-like user experience, we must implement an **Inline Autofill Overlay** directly on the web page. We need to define a secure, high-performance, and compatible architecture for this overlay.

## Decision
We will implement the Inline Autofill Overlay using the following architectural design:

1. **Isolated `<iframe>` Injection**:
   The Autofill Badge and the Autofill Dropdown Panel will be injected as `<iframe>` elements loaded from the secure extension origin (`chrome-extension://`). This ensures absolute style and script isolation, preventing the host page's JavaScript or CSS from tampering with or stealing decrypted credentials.

2. **Direct Background Communication**:
   The injected dropdown iframe will communicate directly with the background service worker via `browser.runtime.sendMessage` to fetch matching credentials. Decrypted credentials will never enter the host page's content script context during the suggestion phase.

3. **Static Inline Badge + Dropdown Panel**:
   A small KeyVault badge icon will be placed inside target inputs. Clicking this badge will open a vertically stacked credential suggestion panel directly below the input field.

4. **Absolute Positioning via Coordinate Tracking**:
   The badge and panel iframes will be appended directly to the document `<body>` and positioned absolutely using `getBoundingClientRect()` of the input field. Coordinates will be synchronized during scrolling and resizing using `requestAnimationFrame`. This avoids breaking the host site's CSS grid or flex layouts.

5. **Advanced Prototype Setter Simulation**:
   Credentials will be injected using native setters extracted from `HTMLInputElement.prototype.value` and dispatching standard bubble events, ensuring full compatibility with single-page app (SPA) frameworks like React, Svelte, and Vue.

6. **Click to Unlock Redirection**:
   If the vault is locked, clicking the badge opens a prompt directing the user to unlock via the secure, browser-native extension popup, protecting the Master Password from being typed into webpage-injected fields.

7. **Focus Event Delegation**:
   The content script will listen to the global `focusin` event rather than performing constant DOM scans or running a `MutationObserver`. The Autofill Badge is dynamically injected only when a target field is focused.

8. **Dismissal on Click Outside**:
   The dropdown panel closes automatically when clicking outside the panel/input area or on loss of input focus.

## Consequences
- **Security**: Strong protection against malicious scripts and frame-based harvesting because credentials never cross the insecure content script boundary until the user explicitly triggers autofill.
- **Performance**: Near-zero CPU overhead on page load since monitoring is passive and event-driven.
- **UX**: Professional, non-intrusive autofill interface that works side-by-side with native browser dropdowns.
- **Complexity**: Requires careful iframe-to-background messaging and scroll-tracking positioning math.
