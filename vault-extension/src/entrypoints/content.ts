export default defineContentScript({
  matches: ['*://*/*'],
  main() {
    let activeInput: HTMLInputElement | null = null;
    let activeBadgeIframe: HTMLIFrameElement | null = null;
    let activeDropdownIframe: HTMLIFrameElement | null = null;
    let frameUpdatePending = false;

    // Helper: Determine if element is a valid username/password input target
    function isAutofillTarget(el: HTMLElement): el is HTMLInputElement {
      if (el.tagName !== 'INPUT') return false;
      const input = el as HTMLInputElement;

      // Skip hidden, button, or structural inputs
      const skippedTypes = [
        'hidden',
        'submit',
        'button',
        'checkbox',
        'radio',
        'file',
        'image',
        'range',
      ];
      if (skippedTypes.includes(input.type)) return false;

      // Skip elements that are visibly hidden
      const style = window.getComputedStyle(input);
      if (
        style.display === 'none' ||
        style.visibility === 'hidden' ||
        input.offsetWidth === 0 ||
        input.offsetHeight === 0
      ) {
        return false;
      }

      if (input.type === 'password') return true;

      // Email/Text inputs are targets if they exist in a form containing a password field
      const form = input.form;
      if (form) {
        return form.querySelector('input[type="password"]') !== null;
      }

      // Check siblings for password inputs if form element is absent
      let sibling = input.nextElementSibling;
      while (sibling) {
        if (
          sibling.tagName === 'INPUT' &&
          (sibling as HTMLInputElement).type === 'password'
        ) {
          return true;
        }
        sibling = sibling.nextElementSibling;
      }

      return false;
    }

    // Helper: Safely inject value using prototype setters to support React/Svelte/Vue state updates
    function fillInputSecurely(input: HTMLInputElement, value: string) {
      try {
        const descriptor = Object.getOwnPropertyDescriptor(
          HTMLInputElement.prototype,
          'value',
        );
        const setter = descriptor?.set;
        if (setter) {
          setter.call(input, value);
        } else {
          input.value = value;
        }
        input.dispatchEvent(new Event('input', { bubbles: true }));
        input.dispatchEvent(new Event('change', { bubbles: true }));
      } catch (e) {
        console.error('Failed to securely set input value:', e);
        input.value = value;
      }
    }

    // Helper: Find username/password fields and inject credentials
    function fillForm(input: HTMLInputElement, user: string, pass: string) {
      if (!input) return;

      let passInput: HTMLInputElement | null = null;
      let userInput: HTMLInputElement | null = null;

      const form = input.form;
      if (form) {
        passInput = form.querySelector('input[type="password"]');
        userInput = form.querySelector(
          'input[type="email"], input[type="text"], input:not([type])',
        );
      } else {
        // Fallback sibling traversal
        if (input.type === 'password') {
          passInput = input;
          let sibling = input.previousElementSibling;
          while (sibling) {
            if (sibling.tagName === 'INPUT') {
              userInput = sibling as HTMLInputElement;
              break;
            }
            const inner = sibling.querySelector('input');
            if (inner) {
              userInput = inner;
              break;
            }
            sibling = sibling.previousElementSibling;
          }
        } else {
          userInput = input;
          let sibling = input.nextElementSibling;
          while (sibling) {
            if (
              sibling.tagName === 'INPUT' &&
              (sibling as HTMLInputElement).type === 'password'
            ) {
              passInput = sibling as HTMLInputElement;
              break;
            }
            const inner = sibling.querySelector('input[type="password"]');
            if (inner) {
              passInput = inner as HTMLInputElement;
              break;
            }
            sibling = sibling.nextElementSibling;
          }
        }
      }

      if (passInput && pass) fillInputSecurely(passInput, pass);
      if (userInput && user) fillInputSecurely(userInput, user);
    }

    // Helper: Destroy existing frames
    function destroyBadge() {
      if (activeBadgeIframe) {
        activeBadgeIframe.remove();
        activeBadgeIframe = null;
      }
    }

    function destroyDropdown() {
      if (activeDropdownIframe) {
        activeDropdownIframe.remove();
        activeDropdownIframe = null;
      }
    }

    // Position updates synchronized via requestAnimationFrame
    function updateFramesPosition() {
      if (!activeInput) return;

      const rect = activeInput.getBoundingClientRect();
      const scrollTop =
        window.pageYOffset || document.documentElement.scrollTop;
      const scrollLeft =
        window.pageXOffset || document.documentElement.scrollLeft;

      // Update badge position
      if (activeBadgeIframe) {
        const size = 20;
        const top = rect.top + scrollTop + (rect.height - size) / 2;
        const left = rect.left + scrollLeft + rect.width - size - 8;
        activeBadgeIframe.style.top = `${top}px`;
        activeBadgeIframe.style.left = `${left}px`;
      }

      // Update dropdown position
      if (activeDropdownIframe) {
        const top = rect.top + rect.height + scrollTop + 4;
        const left = rect.left + scrollLeft;
        activeDropdownIframe.style.top = `${top}px`;
        activeDropdownIframe.style.left = `${left}px`;
      }

      frameUpdatePending = false;
    }

    function requestFrameUpdate() {
      if (!frameUpdatePending) {
        frameUpdatePending = true;
        requestAnimationFrame(updateFramesPosition);
      }
    }

    // Toggle Dropdown iframe below input
    function toggleDropdown() {
      if (!activeInput) return;

      if (activeDropdownIframe) {
        destroyDropdown();
        return;
      }

      const dropdown = document.createElement('iframe');
      dropdown.src = browser.runtime.getURL(
        `/inline-dropdown.html?origin=${encodeURIComponent(window.location.origin)}`,
      );
      dropdown.style.position = 'absolute';
      dropdown.style.width = '240px';
      dropdown.style.height = '180px';
      dropdown.style.border = 'none';
      dropdown.style.borderRadius = '8px';
      dropdown.style.boxShadow =
        '0 10px 15px -3px rgba(0, 0, 0, 0.3), 0 4px 6px -2px rgba(0, 0, 0, 0.2)';
      dropdown.style.zIndex = '9999999';
      dropdown.setAttribute('scrolling', 'no');

      document.body.appendChild(dropdown);
      activeDropdownIframe = dropdown;

      updateFramesPosition();
    }

    // Listen to input focus using Event Delegation
    document.addEventListener('focusin', async (e) => {
      const target = e.target as HTMLElement;
      if (!isAutofillTarget(target)) return;

      // If switching to another input, clean up previous frames
      if (activeInput !== target) {
        destroyDropdown();
        destroyBadge();
        activeInput = target;
      }

      // Check if background worker has entries matching page origin, or is locked
      const origin = window.location.origin;
      try {
        const response = await browser.runtime.sendMessage({
          type: 'check-matches',
          origin,
        });

        // Show inline badge if we have matches OR if the vault is currently locked
        if (response.hasMatches || response.isLocked) {
          if (!activeBadgeIframe) {
            const badge = document.createElement('iframe');
            badge.src = browser.runtime.getURL('/inline-badge.html');
            badge.style.position = 'absolute';
            badge.style.width = '20px';
            badge.style.height = '20px';
            badge.style.border = 'none';
            badge.style.background = 'transparent';
            badge.style.zIndex = '999999';
            badge.setAttribute('scrolling', 'no');

            document.body.appendChild(badge);
            activeBadgeIframe = badge;
          }
          updateFramesPosition();
        }
      } catch (e) {
        console.error('Failed to communicate with background:', e);
      }
    });

    // Handle scroll/resize events
    window.addEventListener('scroll', requestFrameUpdate, { passive: true });
    window.addEventListener('resize', requestFrameUpdate, { passive: true });

    // Handle click-away dismissal
    document.addEventListener('pointerdown', (e) => {
      const target = e.target as HTMLElement;
      if (activeDropdownIframe) {
        if (
          target !== activeInput &&
          target !== activeBadgeIframe &&
          target !== activeDropdownIframe
        ) {
          destroyDropdown();
        }
      }
    });

    // Listen for postMessages sent by our iframe elements (Badge and Dropdown)
    window.addEventListener('message', (event) => {
      // Validate that message is coming from extension origin
      const extensionOrigin = browser.runtime.getURL('').slice(0, -1);
      if (event.origin !== extensionOrigin) return;

      if (event.data?.type === 'keyvault-badge-clicked') {
        toggleDropdown();
      }

      if (event.data?.type === 'keyvault-autofill') {
        const { username, password } = event.data;
        if (activeInput) {
          fillForm(activeInput, username, password);
        }
        destroyDropdown();
      }
    });
  },
});
