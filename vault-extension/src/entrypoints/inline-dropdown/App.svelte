<script lang="ts">
import { Key, Lock, Sparkles } from 'lucide-svelte';
import { onMount } from 'svelte';
import { isBiometricsEnabled, verifyBiometrics } from '~/lib/biometrics.js';
import {
  type Account,
  type DomainGroup,
  vaultState,
} from '~/lib/vault-state.svelte.js';

let origin = $state('');
let matchingEntries = $state<{ account: Account; group: DomainGroup }[]>([]);

let selectedEntryToAutofill = $state<Account | null>(null);
let showPasswordPrompt = $state(false);
let reauthPassword = $state('');
let reauthError = $state('');
let isVerifying = $state(false);

onMount(async () => {
  // Parse origin from URL query params
  const params = new URLSearchParams(window.location.search);
  origin = params.get('origin') || '';

  // Initialize secure vault core
  await vaultState.initWasm();

  // Find matching credentials if unlocked
  if (vaultState.isUnlocked && origin) {
    findMatches();
  }
});

const findMatches = () => {
  const cleanPage = origin
    .toLowerCase()
    .replace(/^https?:\/\//, '')
    .replace(/^www\./, '')
    .split('/')[0];
  const items = vaultState.vault.items || [];

  const matches: { account: Account; group: DomainGroup }[] = [];
  for (const item of items) {
    if (item.type === 'DomainGroup') {
      const isMatch = (item.urls || []).some((u) => {
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
      if (isMatch) {
        for (const account of item.accounts || []) {
          matches.push({ account, group: item });
        }
      }
    }
  }
  matchingEntries = matches;
};

const doAutofill = (entry: Account) => {
  window.parent.postMessage(
    {
      type: 'keyvault-autofill',
      username: entry.username || '',
      password: entry.password || '',
    },
    '*',
  );
};

const handleSelect = async (entry: Account) => {
  selectedEntryToAutofill = entry;

  // Try biometrics first if configured
  const bioEnabled = await isBiometricsEnabled();
  if (bioEnabled) {
    try {
      const success = await verifyBiometrics();
      if (success) {
        doAutofill(entry);
        selectedEntryToAutofill = null;
        return;
      }
    } catch (e) {
      console.warn(
        'Biometric re-auth failed in iframe, falling back to password:',
        e,
      );
    }
  }

  // Show inline password prompt fallback
  reauthPassword = '';
  reauthError = '';
  showPasswordPrompt = true;
};

const handlePasswordVerify = async (e: Event) => {
  e.preventDefault();
  if (!reauthPassword.trim() || !selectedEntryToAutofill) return;

  isVerifying = true;
  reauthError = '';
  try {
    const isValid = await vaultState.verifyPassword(reauthPassword);
    if (isValid) {
      doAutofill(selectedEntryToAutofill);
      showPasswordPrompt = false;
      selectedEntryToAutofill = null;
      reauthPassword = '';
    } else {
      reauthError = 'Incorrect master password.';
    }
  } catch {
    reauthError = 'Verification failed.';
  } finally {
    isVerifying = false;
  }
};

const handleUnlock = async () => {
  // Send message to background to trigger popup opening
  await browser.runtime.sendMessage({ type: 'open-popup' });
};
</script>

<div class="h-full flex flex-col justify-between bg-zinc-950 p-1.5 text-zinc-200 select-none">
    {#if !vaultState.wasmReady}
        <div class="grow flex items-center justify-center py-6">
            <div class="animate-spin rounded-full h-4 w-4 border-2 border-zinc-800 border-t-zinc-400"></div>
        </div>
    {:else if showPasswordPrompt}
        <!-- INLINE MASTER PASSWORD FALLBACK -->
        <form onsubmit={handlePasswordVerify} class="grow flex flex-col justify-between p-2 gap-2">
            <div class="space-y-1">
                <div class="flex items-center gap-1.5 text-[10px] text-zinc-400 font-semibold uppercase tracking-wider">
                    <Lock class="h-3.5 w-3.5 text-accent" /> Security Verification
                </div>
                <p class="text-[9px] text-zinc-500 leading-normal">
                    Enter your master password to authorize autofill.
                </p>
                <input
                    type="password"
                    placeholder="Master password..."
                    bind:value={reauthPassword}
                    class="w-full bg-zinc-900 border border-zinc-850 text-white text-xs px-2.5 py-1.5 rounded focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/50 mt-1"
                    required
                    disabled={isVerifying}
                />
                {#if reauthError}
                    <p class="text-[9px] text-red-400 mt-1 font-medium">{reauthError}</p>
                {/if}
            </div>

            <div class="flex gap-2 mt-auto">
                <button
                    type="button"
                    onclick={() => { showPasswordPrompt = false; selectedEntryToAutofill = null; }}
                    class="w-1/2 bg-zinc-900 border border-zinc-800 hover:text-white text-[10px] py-1.5 px-2 rounded cursor-pointer transition-colors focus:outline-none"
                    disabled={isVerifying}
                >
                    Cancel
                </button>
                <button
                    type="submit"
                    class="w-1/2 bg-zinc-50 text-zinc-950 hover:bg-zinc-200 text-[10px] font-semibold py-1.5 px-2 rounded cursor-pointer transition-colors focus:outline-none flex items-center justify-center"
                    disabled={isVerifying}
                >
                    {#if isVerifying}
                        <div class="animate-spin rounded-full h-3 w-3 border-2 border-zinc-950 border-t-transparent"></div>
                    {:else}
                        Verify
                    {/if}
                </button>
            </div>
        </form>
    {:else if !vaultState.isUnlocked}
        <!-- LOCKED STATE -->
        <div class="grow flex flex-col items-center justify-center p-3 text-center gap-2">
            <Lock class="h-5 w-5 text-zinc-500" />
            <p class="text-[11px] text-zinc-400 leading-tight">KeyVault is locked</p>
            <button
                type="button"
                onclick={handleUnlock}
                class="w-full mt-1 bg-zinc-50 hover:bg-zinc-200 text-zinc-950 text-[10px] font-semibold py-1.5 px-3 rounded cursor-pointer transition-colors focus:outline-none"
            >
                Unlock Vault
            </button>
        </div>
    {:else}
        <!-- UNLOCKED SUGGESTIONS STATE -->
        <div class="grow flex flex-col justify-start">
            <header class="flex items-center gap-1.5 px-2 py-1 border-b border-zinc-900 pb-1.5 mb-1 text-[9px] font-semibold text-zinc-500 uppercase tracking-wider">
                <Sparkles class="h-3 w-3 text-accent" /> Suggestions
            </header>

            {#if matchingEntries.length === 0}
                <div class="grow flex flex-col items-center justify-center py-6 px-4 text-center gap-1">
                    <p class="text-[11px] text-zinc-400">No suggestions found</p>
                    <p class="text-[9px] text-zinc-600">Add credentials in popup</p>
                </div>
            {:else}
                <div class="overflow-y-auto max-h-[140px] space-y-0.5 pr-0.5">
                    {#each matchingEntries as match (match.account.id)}
                        <button
                            type="button"
                            onclick={() => handleSelect(match.account)}
                            class="w-full flex items-center justify-between text-left p-2 rounded hover:bg-zinc-900/60 border border-transparent hover:border-zinc-800/50 cursor-pointer transition-all focus:outline-none focus-visible:bg-zinc-900"
                        >
                            <div class="truncate pr-2">
                                <p class="text-xs font-medium text-zinc-200 truncate">{match.account.username || 'No Username'}</p>
                                <p class="text-[9px] text-zinc-500 truncate mt-0.5">{match.group.title}</p>
                            </div>
                            <Key class="h-3.5 w-3.5 text-zinc-600 shrink-0" />
                        </button>
                    {/each}
                </div>
            {/if}
        </div>
    {/if}
</div>
