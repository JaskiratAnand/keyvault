<script lang="ts">
import { LoaderCircle, ShieldAlert, ShieldCheck } from 'lucide-svelte';
import { onMount } from 'svelte';
import HelpView from './components/HelpView.svelte';
// Import split modular components
import LockScreen from './components/LockScreen.svelte';
import PasswordGenerator from './components/PasswordGenerator.svelte';
import SettingsView from './components/SettingsView.svelte';
import Sidebar from './components/Sidebar.svelte';
import SyncView from './components/SyncView.svelte';
import TrashView from './components/TrashView.svelte';
import VaultView from './components/VaultView.svelte';

import type { VaultItem } from './lib/types.js';
import { setVaultContext, vaultState } from './lib/vault-state.svelte.js';

// Share the reactive vaultState singleton via Context API for Svelte 5
setVaultContext(vaultState);

const { invoke } = (window as any).__TAURI__?.core || {};

// Tab Navigation State
let currentTab = $state<
  'vault' | 'generator' | 'trash' | 'sync' | 'settings' | 'help'
>('vault');

// Mismatch remote password
let mismatchPassword = $state('');

// Re-authentication states
let lastAuthTime = 0;
const AUTH_GRACE_PERIOD = 2 * 60 * 1000;
let showReauthModal = $state(false);
let isReauthLoading = $state(false);
let reauthPassword = $state('');
let reauthError = $state('');
let reauthCallback = $state<((value: boolean) => void) | null>(null);

// Trash list active selected item
let selectedTrashEntry = $state<VaultItem | null>(null);

// Confirmation Modal states
let confirmModal = $state<{
  show: boolean;
  title: string;
  message: string;
}>({
  show: false,
  title: '',
  message: '',
});
let confirmCallback: (() => void | Promise<void>) | null = null;

function showConfirm(
  title: string,
  message: string,
  onConfirm: () => void | Promise<void>,
) {
  confirmCallback = onConfirm;
  confirmModal = {
    show: true,
    title,
    message,
  };
}

// Auto-init state check
onMount(() => {
  if (
    typeof window !== 'undefined' &&
    navigator.userAgent.includes('Windows')
  ) {
    document.documentElement.classList.add('is-windows');
  }
  vaultState.checkGDriveAuth();
  vaultState.checkVaultExists();
  vaultState.checkBiometricsSupport();
});

// Re-authentication handling
function requestReauth(): Promise<boolean> {
  const now = Date.now();
  if (now - lastAuthTime < AUTH_GRACE_PERIOD) {
    return Promise.resolve(true);
  }

  return new Promise((resolve) => {
    const run = async () => {
      // Attempt biometrics first if enabled
      if (vaultState.bioEnabled && invoke) {
        try {
          const success = await invoke('authenticate_biometrics');
          if (success) {
            lastAuthTime = Date.now();
            resolve(true);
            return;
          }
        } catch (e) {
          console.warn('Biometrics failed:', e);
        }
      }

      // Show modal fallback
      reauthPassword = '';
      reauthError = '';
      isReauthLoading = false;
      reauthCallback = (val) => {
        showReauthModal = false;
        resolve(val);
      };
      showReauthModal = true;
    };
    run();
  });
}

async function handleConfirmReauth() {
  if (isReauthLoading) return;
  isReauthLoading = true;
  reauthError = '';
  try {
    // Enforce a minimum verification delay of 600ms so the loading spinner is visibly shown and feels deliberate
    const [ok] = await Promise.all([
      vaultState.verifyPassword(reauthPassword),
      new Promise((resolve) => setTimeout(resolve, 600)),
    ]);
    if (ok) {
      lastAuthTime = Date.now();
      if (reauthCallback) reauthCallback(true);
    } else {
      reauthError = 'Incorrect master password.';
    }
  } catch (e) {
    console.error(e);
    reauthError = 'An error occurred during verification.';
  } finally {
    isReauthLoading = false;
  }
}

function handleCancelReauth() {
  if (reauthCallback) reauthCallback(false);
}

function handleLock() {
  vaultState.lock();
}

async function resolveMismatch() {
  const success = await vaultState.resolveSyncSaltMismatch(mismatchPassword);
  if (success) {
    mismatchPassword = '';
  }
}

function handleTabChange(tab: string) {
  if (tab === 'trash') {
    vaultState.activeTab = 'trash';
    selectedTrashEntry = null;
  } else {
    if (vaultState.activeTab === 'trash') {
      vaultState.activeTab = 'all';
    }
  }
}
</script>

<div class="h-screen w-screen flex flex-col bg-[#09090b] text-[#fafafa] overflow-hidden select-none font-sans antialiased">
  {#if !vaultState.isUnlocked}
    <!-- Locked state / Setup Wizard -->
    <LockScreen onHelpTab={() => currentTab = 'help'} />
  {:else}
    <!-- Main Application Layout with Navigation Sidebar -->
    <div class="grow flex overflow-hidden">
      <!-- Sidebar Navigation -->
      <Sidebar
        bind:currentTab
        onLock={handleLock}
        onTabChange={handleTabChange}
      />

      <!-- Right Main Content Panel -->
      <main class="grow flex overflow-hidden bg-[#09090b]">
        {#if currentTab === 'vault'}
          <VaultView
            {requestReauth}
            {showConfirm}
          />
        {:else if currentTab === 'generator'}
          <div class="grow flex flex-col items-center justify-center p-8 max-w-xl mx-auto w-full">
            <div class="w-full space-y-4">
              <div>
                <h2 class="text-lg font-bold text-white mb-1">Password Generator</h2>
                <p class="text-xs text-zinc-400">Generate strong, random credentials locally on your machine.</p>
              </div>
              <div class="h-145 max-h-[70vh] w-full">
                <PasswordGenerator />
              </div>
            </div>
          </div>
        {:else if currentTab === 'trash'}
          <TrashView
            {showConfirm}
            bind:selectedTrashEntry
          />
        {:else if currentTab === 'sync'}
          <SyncView
            {showConfirm}
          />
        {:else if currentTab === 'settings'}
          <SettingsView
            bind:currentTab
          />
        {:else if currentTab === 'help'}
          <HelpView />
        {/if}
      </main>
    </div>
  {/if}

  <!-- Unified Re-authentication Modal (Sleek Inline Design) -->
  {#if showReauthModal}
    <div class="fixed inset-0 bg-[#09090b]/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      <div class="w-full max-w-sm border border-[#27272a] bg-[#18181b] p-6 rounded-lg shadow-2xl space-y-4">
        <div class="flex items-center gap-2 text-[#06b6d4]">
          <ShieldCheck class="w-5 h-5" />
          <h2 class="text-sm font-bold text-[#fafafa]">Re-authentication Required</h2>
        </div>

        <p class="text-xs text-[#a1a1aa] leading-relaxed">
          Please enter your Master Password to authorize this protected action.
        </p>

        <form onsubmit={(e) => { e.preventDefault(); handleConfirmReauth(); }} class="space-y-3">
          <input
            type="password"
            placeholder="Master Password"
            bind:value={reauthPassword}
            disabled={isReauthLoading}
            class="w-full bg-[#09090b] border border-[#27272a] rounded-lg px-3 py-2 text-xs text-[#fafafa] placeholder-[#a1a1aa] outline-none focus:border-[#d4d4d8] focus:ring-1 focus:ring-[#06b6d4]/40 transition-all disabled:opacity-50"
            autocapitalize="none"
            autocorrect="off"
            spellcheck="false"
          />

          {#if reauthError}
            <div class="text-[11px] text-[#ef4444] bg-[#7f1d1d]/10 border border-[#7f1d1d]/20 p-2 rounded-lg">
              {reauthError}
            </div>
          {/if}

          <div class="flex justify-end gap-2 pt-2">
            <button
              type="button"
              onclick={handleCancelReauth}
              disabled={isReauthLoading}
              class="text-xs bg-[#27272a] border-[#27272a] px-3.5 py-1.5 rounded-lg text-[#fafafa] hover:bg-[#3f3f46] transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed border-0"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isReauthLoading}
              class="bg-[#fafafa] text-[#18181b] text-xs font-semibold px-4 py-1.5 rounded-lg hover:bg-[#fafafa]/90 transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5 border-0"
            >
              {#if isReauthLoading}
                <LoaderCircle class="h-3 w-3 animate-spin" />
                Authorizing...
              {:else}
                Authorize
              {/if}
            </button>
          </div>
        </form>
      </div>
    </div>
  {/if}

  <!-- Google Drive Mismatch Password Prompt Modal -->
  {#if vaultState.syncNeedsPassword}
    <div class="fixed inset-0 bg-[#09090b]/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      <div class="w-full max-w-sm border border-[#27272a] bg-[#18181b] p-6 rounded-lg shadow-2xl space-y-4">
        <div class="flex items-center gap-2 text-[#ef4444]">
          <ShieldAlert class="w-5 h-5" />
          <h2 class="text-sm font-bold text-white">Sync Salt Mismatch</h2>
        </div>

        <p class="text-xs text-[#a1a1aa] leading-relaxed">
          The backup file on Google Drive was encrypted with a different master key or salt. To merge or restore it, enter the Master Password of that remote backup.
        </p>

        <form onsubmit={(e) => { e.preventDefault(); resolveMismatch(); }} class="space-y-3">
          <input
            type="password"
            placeholder="Remote Vault Password"
            bind:value={mismatchPassword}
            class="w-full bg-[#09090b] border border-[#27272a] rounded-lg px-3 py-1.5 text-xs text-[#fafafa] outline-none"
            autocapitalize="none"
            autocorrect="off"
            spellcheck="false"
          />

          {#if vaultState.syncError}
            <div class="text-[11px] text-[#ef4444] bg-[#7f1d1d]/10 border border-destructive/10 p-2 rounded-lg">
              {vaultState.syncError}
            </div>
          {/if}

          <div class="flex justify-end gap-2 pt-2">
            <button
              type="button"
              onclick={() => {
                vaultState.syncNeedsPassword = false;
                vaultState.pendingRemotePayload = null;
                vaultState.pendingRemoteMetadata = null;
                vaultState.pendingRemoteSalt = null;
              }}
              class="text-xs border border-[#27272a] px-3 py-1.5 rounded-lg text-white transition-colors cursor-pointer bg-transparent"
            >
              Cancel
            </button>
            <button
              type="submit"
              class="bg-[#fafafa] text-[#18181b] text-xs font-semibold px-4 py-1.5 rounded-lg hover:bg-white transition-colors cursor-pointer border-0"
            >
              Unlock Sync
            </button>
          </div>
        </form>
      </div>
    </div>
  {/if}

  <!-- Custom Confirmation Modal -->
  {#if confirmModal.show}
    <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-[#09090b]/80 backdrop-blur-sm transition-all duration-300">
      <div class="bg-[#18181b] border border-[#27272a] rounded-xl max-w-sm w-full p-6 shadow-2xl space-y-4">
        <div class="flex items-start gap-3">
          <div class="p-2 rounded-lg bg-red-950/30 border border-red-900/30 text-red-400 shrink-0">
            <ShieldAlert class="w-5 h-5" />
          </div>
          <div>
            <h3 class="text-sm font-bold text-white leading-none mb-1">{confirmModal.title}</h3>
            <p class="text-xs text-[#a1a1aa] leading-normal">{confirmModal.message}</p>
          </div>
        </div>

        <div class="flex justify-end gap-2.5 pt-2">
          <button
            onclick={() => { confirmModal.show = false; confirmCallback = null; }}
            class="px-4 py-2 border border-[#27272a] text-xs font-semibold rounded-lg hover:bg-zinc-800 text-white transition-colors cursor-pointer bg-transparent"
          >
            Cancel
          </button>
          <button
            onclick={async () => {
              console.log('[ConfirmModal] Confirm clicked, storing callback reference');
              const cb = confirmCallback;
              if (cb) {
                console.log('[ConfirmModal] Executing stored callback');
                try {
                  await cb();
                } catch (e) {
                  console.error('[ConfirmModal] Error in callback:', e);
                }
              }
              console.log('[ConfirmModal] Callback execution complete, hiding modal');
              confirmModal.show = false;
              confirmCallback = null;
            }}
            class="bg-red-650 hover:bg-red-650/90 text-white text-xs font-bold px-4 py-2 rounded-lg transition-colors cursor-pointer border-0"
          >
            Confirm
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>
