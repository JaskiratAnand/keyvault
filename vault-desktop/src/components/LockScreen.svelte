<script lang="ts">
import { KeyRound, Lock, RefreshCw, ShieldAlert, Unlock } from 'lucide-svelte';
import { getVaultContext } from '../lib/vault-state.svelte.js';
import SetupWizard from './SetupWizard.svelte';

interface Props {
  onHelpTab: () => void;
}

let { onHelpTab }: Props = $props();

const vaultState = getVaultContext();

let password = $state('');
let confirmPassword = $state('');
let showResetConfirm = $state(false);

async function handleUnlock() {
  if (!vaultState.vaultExists) {
    if (!password) {
      vaultState.error = 'Password cannot be empty.';
      return;
    }
    if (password !== confirmPassword) {
      vaultState.error = 'Passwords do not match.';
      return;
    }
  }

  vaultState.error = '';
  vaultState.loading = true;

  // Brief timeout to let the UI update its loading state
  await new Promise((resolve) => setTimeout(resolve, 50));

  const success = await vaultState.unlock(password);
  if (success) {
    password = '';
    confirmPassword = '';
  }
}

function handleResetVault() {
  showResetConfirm = true;
}
</script>

<div class="grow flex items-center justify-center p-4 overflow-y-auto">
  {#if !vaultState.vaultExists}
    <div class="w-full max-w-xl border border-[#27272a] bg-[#18181b] p-6 md:p-8 rounded-lg shadow-xl">
      <SetupWizard onComplete={onHelpTab} />
    </div>
  {:else}
    <div class="w-full max-w-sm border border-[#27272a] bg-[#18181b] p-8 rounded-lg shadow-xl text-center">
      <!-- Logo -->
      <div class="flex justify-center items-center gap-2 mb-6">
        <div class="bg-[#06b6d4]/10 p-2.5 rounded-lg border border-[#06b6d4]/30">
          <Lock class="w-6 h-6 text-[#06b6d4]" />
        </div>
        <span class="text-xl font-bold tracking-tight">Key<span class="text-[#06b6d4]">Vault</span></span>
        <span class="text-[9px] uppercase px-1.5 py-0.5 rounded border border-[#27272a] text-[#a1a1aa] bg-[#09090b]">Desktop</span>
      </div>

      {#if showResetConfirm}
        <h2 class="text-base font-semibold mb-1 text-[#ef4444] flex items-center justify-center gap-1.5">
          <ShieldAlert class="w-5 h-5 shrink-0" />
          Wipe & Reset Vault?
        </h2>
        <p class="text-xs text-[#a1a1aa] mb-6 leading-relaxed">
          This will permanently delete the local vault file and wipe all stored credentials from this device. This cannot be undone.
        </p>

        <div class="space-y-3">
          <button
            onclick={async () => {
              await vaultState.resetVault();
              showResetConfirm = false;
              password = '';
              confirmPassword = '';
            }}
            disabled={vaultState.loading}
            class="w-full bg-[#ef4444] hover:bg-[#ef4444]/90 text-white text-sm font-semibold py-2 rounded-lg transition-colors flex items-center justify-center gap-2 disabled:opacity-50 font-sans cursor-pointer"
          >
            {#if vaultState.loading}
              <RefreshCw class="w-4 h-4 animate-spin" />
              Resetting...
            {:else}
              Yes, Reset Vault
            {/if}
          </button>

          <button
            onclick={() => showResetConfirm = false}
            disabled={vaultState.loading}
            class="w-full bg-transparent border border-[#27272a] hover:border-white/10 text-white text-xs font-semibold py-2 rounded-lg transition-colors cursor-pointer"
          >
            Cancel
          </button>
        </div>
      {:else}
        {#if vaultState.vaultExists}
          <h2 class="text-base font-semibold mb-1 text-white">Unlock Vault</h2>
          <p class="text-xs text-[#a1a1aa] mb-6">Enter your Master Password to decrypt your credentials.</p>
        {:else}
          <h2 class="text-base font-semibold mb-1 text-white">Setup Master Password</h2>
          <p class="text-xs text-[#a1a1aa] mb-6">Create a secure password to initialize your encrypted vault.</p>
        {/if}

        <form onsubmit={(e) => { e.preventDefault(); handleUnlock(); }} class="space-y-4">
          <div class="relative">
            <input
              type="password"
              placeholder="Master Password"
              bind:value={password}
              disabled={vaultState.loading}
              class="w-full bg-[#09090b] border border-[#27272a] rounded-lg px-3 py-2 text-sm text-[#fafafa] placeholder-[#52525b] outline-none transition-colors focus:border-[#d4d4d8] disabled:opacity-50"
              autocapitalize="none"
              autocorrect="off"
              spellcheck="false"
            />
            <div class="absolute right-3 top-1/2 -translate-y-1/2 text-[#a1a1aa]">
              <KeyRound class="w-4 h-4 opacity-50" />
            </div>
          </div>

          {#if !vaultState.vaultExists}
            <div class="relative">
              <input
                type="password"
                placeholder="Confirm Master Password"
                bind:value={confirmPassword}
                disabled={vaultState.loading}
                class="w-full bg-[#09090b] border border-[#27272a] rounded-lg px-3 py-2 text-sm text-[#fafafa] placeholder-[#52525b] outline-none transition-colors focus:border-[#d4d4d8] disabled:opacity-50"
                autocapitalize="none"
                autocorrect="off"
                spellcheck="false"
              />
              <div class="absolute right-3 top-1/2 -translate-y-1/2 text-[#a1a1aa]">
                <KeyRound class="w-4 h-4 opacity-50" />
              </div>
            </div>
          {/if}

          {#if vaultState.error}
            <div class="bg-destructive/10 border border-destructive/20 text-[#ef4444] text-xs p-2 rounded-lg text-left flex items-start gap-2">
              <ShieldAlert class="w-4 h-4 shrink-0 mt-0.5" />
              <span>{vaultState.error}</span>
            </div>
          {/if}

          <button
            type="submit"
            disabled={vaultState.loading}
            class="w-full bg-[#fafafa] hover:bg-[#fafafa]/90 text-[#18181b] text-sm font-semibold py-2 rounded-lg transition-colors flex items-center justify-center gap-2 disabled:opacity-50 cursor-pointer"
          >
            {#if vaultState.loading}
              <RefreshCw class="w-4 h-4 animate-spin" />
              {vaultState.vaultExists ? 'Unlocking...' : 'Initializing...'}
            {:else}
              <Unlock class="w-4 h-4" />
              {vaultState.vaultExists ? 'Unlock' : 'Create Vault'}
            {/if}
          </button>
        </form>

        <div class="mt-6 border-t border-[#27272a] pt-4 flex flex-col gap-2">
          {#if vaultState.vaultExists}
            <button
              onclick={handleResetVault}
              class="text-[10px] text-[#ef4444] hover:underline font-medium bg-transparent border-0 cursor-pointer self-center"
            >
              Forgot Master Password? Reset Vault
            </button>
          {:else}
            <p class="text-[11px] text-[#a1a1aa]">
              Your password is used locally to encrypt all data before saving. It cannot be recovered.
            </p>
          {/if}
        </div>
      {/if}
    </div>
  {/if}
</div>
