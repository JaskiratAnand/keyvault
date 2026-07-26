import { vaultState } from '~/lib/vault-state.svelte';

export default defineBackground(() => {
  // Listen for runtime messages from content scripts and iframes
  browser.runtime.onMessage.addListener((message, _sender, sendResponse) => {
    if (message.type === 'check-matches') {
      const { origin } = message;
      if (!origin) {
        sendResponse({ hasMatches: false, count: 0 });
        return true;
      }

      (async () => {
        try {
          // Initialize cryptography engine and check session state
          await vaultState.initWasm();
          if (!vaultState.isUnlocked) {
            // Even if locked, we return matches: false (content script handles lock state via badge clicking)
            sendResponse({ hasMatches: false, count: 0, isLocked: true });
            return;
          }

          const cleanPage = origin
            .toLowerCase()
            .replace(/^https?:\/\//, '')
            .replace(/^www\./, '')
            .split('/')[0];
          const items = vaultState.vault.items || [];

          let matchCount = 0;
          for (const item of items) {
            if (item.type === 'DomainGroup') {
              const matches = (item.urls || []).some((u) => {
                const cleanEntry = u
                  .toLowerCase()
                  .replace(/^https?:\/\//, '')
                  .replace(/^www\./, '')
                  .split('/')[0];
                return (
                  cleanPage === cleanEntry ||
                  cleanPage.endsWith(`.${cleanEntry}`) ||
                  cleanEntry.endsWith(`.${cleanPage}`)
                );
              });
              if (matches) {
                matchCount += (item.accounts || []).length;
              }
            }
          }

          sendResponse({
            hasMatches: matchCount > 0,
            count: matchCount,
            isLocked: false,
          });
        } catch (e) {
          console.error('Background check-matches failed:', e);
          sendResponse({ hasMatches: false, count: 0, isLocked: true });
        }
      })();

      return true; // Keep channel open for async response
    }

    if (message.type === 'open-popup') {
      if (typeof browser !== 'undefined' && browser.action?.openPopup) {
        browser.action
          .openPopup()
          .catch((e) => console.error('Failed to open popup:', e));
      }
      sendResponse({ success: true });
      return true;
    }

    if (message.type === 'is-unlocked') {
      (async () => {
        await vaultState.initWasm();
        sendResponse({ isUnlocked: vaultState.isUnlocked });
      })();
      return true;
    }
  });
});
