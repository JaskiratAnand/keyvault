import { isBiometricsEnabled, verifyBiometrics } from './biometrics.js';
import { vaultState } from './vault-state.svelte.js';

class ReauthState {
  isAuthorized = $state(false);
}

export const reauthState = new ReauthState();

class ReauthController {
  showModal = $state(false);
  errorMsg = $state('');
  resolveCallback = $state<((value: boolean) => void) | null>(null);

  async requestReauth(): Promise<boolean> {
    // If already authorized in this active view session, pass immediately
    if (reauthState.isAuthorized) {
      return true;
    }

    // Attempt biometrics if configured
    const bioEnabled = await isBiometricsEnabled();
    if (bioEnabled) {
      try {
        const success = await verifyBiometrics();
        if (success) {
          reauthState.isAuthorized = true;
          return true;
        }
      } catch (e) {
        console.warn(
          'Biometric re-auth failed or cancelled, falling back to password:',
          e,
        );
      }
    }

    // Fall back to showing the master password prompt dialog
    return new Promise((resolve) => {
      this.errorMsg = '';
      this.resolveCallback = (val) => {
        this.showModal = false;
        resolve(val);
      };
      this.showModal = true;
    });
  }

  async verifyPassword(password: string): Promise<boolean> {
    try {
      const isValid = await vaultState.verifyPassword(password);
      if (isValid) {
        reauthState.isAuthorized = true;
        if (this.resolveCallback) {
          this.resolveCallback(true);
        }
        return true;
      }
      this.errorMsg = 'Incorrect master password.';
      return false;
    } catch {
      this.errorMsg = 'Verification failed.';
      return false;
    }
  }

  cancel() {
    this.errorMsg = '';
    if (this.resolveCallback) {
      this.resolveCallback(false);
    }
  }
}

export const reauthController = new ReauthController();
