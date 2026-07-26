<script lang="ts">
import { AlertTriangle, Globe, Lock, LogOut, RefreshCw } from 'lucide-svelte';
import { getVaultContext } from '../lib/vault-state.svelte.js';

interface Props {
  showConfirm: (
    title: string,
    message: string,
    onConfirm: () => void | Promise<void>,
  ) => void;
}

let { showConfirm }: Props = $props();

const vaultState = getVaultContext();
</script>

<div class="grow overflow-y-auto p-8 space-y-6">
  <div class="max-w-xl mx-auto space-y-4 animate-fade-in">
    <div>
      <h2 class="text-lg font-bold text-white mb-1">Google Drive Sync</h2>
      <p class="text-xs text-zinc-400">Link your Google Account to back up and sync your credentials across all devices.</p>
    </div>

    <div class="space-y-4 pt-2">
      <p class="text-sm text-zinc-300 leading-relaxed">
        Synchronization will copy your encrypted vault database securely to your own private Google Drive account inside the isolated appDataFolder space.
      </p>

      {#if !vaultState.gdriveAuthenticated}
        <div class="p-6 bg-zinc-950/40 border border-[#27272a] rounded-lg flex flex-col items-center gap-4 py-8 text-center">
          <div class="p-3 rounded-full bg-zinc-900/50 border border-zinc-800 text-zinc-500">
            <Lock class="h-6 w-6" />
          </div>
          <p class="text-xs text-zinc-400">Google Drive is not connected.</p>
          <button
            onclick={() => vaultState.signInGDrive()}
            disabled={vaultState.loading}
            class="px-4 py-2 bg-zinc-50 text-zinc-950 hover:bg-zinc-200 disabled:opacity-50 rounded-lg text-xs font-semibold flex items-center gap-1.5 transition-colors cursor-pointer border-0"
          >
            {#if vaultState.loading}
              <RefreshCw class="h-3.5 w-3.5 animate-spin" /> Connecting...
            {:else}
              Connect Google Drive Account
            {/if}
          </button>
        </div>
      {:else}
        <div class="p-5 bg-zinc-950 border border-[#27272a] rounded-lg space-y-4">
          <div class="flex justify-between items-start">
            <div class="flex items-center gap-3">
              <div class="p-2.5 rounded-lg bg-zinc-900 border border-zinc-800 text-[#06b6d4]">
                <Globe class="h-5 w-5" />
              </div>
              <div>
                <p class="text-[9px] text-zinc-400 uppercase tracking-wider font-semibold">Connected Profile</p>
                <p class="text-sm font-semibold text-white mt-0.5">{vaultState.gdriveEmail}</p>
              </div>
            </div>
            <span class="text-[10px] px-2.5 py-0.5 bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 rounded-full font-medium">Active Sync</span>
          </div>

          <!-- Comparison Grid -->
          <div class="grid grid-cols-2 gap-3 pt-2">
            <div class="p-3 bg-zinc-900/30 border border-[#27272a] rounded-lg space-y-2">
              <h4 class="text-[9px] font-bold text-zinc-400 uppercase tracking-wider">Local Database</h4>
              <div class="space-y-1.5 text-xs">
                <div class="flex justify-between text-zinc-400">
                  <span>Active:</span>
                  <span class="text-white font-medium">{vaultState.vault?.items?.length || 0} entries</span>
                </div>
                <div class="flex justify-between text-zinc-400">
                  <span>Trash:</span>
                  <span class="text-white font-medium">{vaultState.vault?.trash?.length || 0} entries</span>
                </div>
              </div>
            </div>

            <div class="p-3 bg-zinc-900/30 border border-[#27272a] rounded-lg space-y-2">
              <h4 class="text-[9px] font-bold text-zinc-400 uppercase tracking-wider">Google Drive Backup</h4>
              <div class="space-y-1.5 text-xs">
                <div class="text-[10px] text-zinc-500 leading-normal">
                  Encryption payload stored securely in your appDataFolder.
                </div>
              </div>
            </div>
          </div>

          <!-- Sync Action Buttons -->
          <div class="space-y-2 pt-2 border-t border-[#27272a]/60">
            <div class="flex gap-2">
              <button
                onclick={() => vaultState.syncVault()}
                disabled={vaultState.syncing}
                class="flex-1 px-4 py-2 bg-zinc-50 hover:bg-zinc-200 text-zinc-950 disabled:opacity-50 rounded-lg text-xs font-semibold flex items-center justify-center gap-1.5 transition-colors cursor-pointer border-0"
              >
                <RefreshCw class="h-3.5 w-3.5 {vaultState.syncing ? 'animate-spin' : ''}" />
                {vaultState.syncing ? 'Syncing...' : 'Sync & Merge'}
              </button>
            </div>
            <div class="flex gap-2">
              <button
                onclick={() => vaultState.restoreRemote()}
                disabled={vaultState.syncing}
                class="w-1/2 px-4 py-2 bg-zinc-900 hover:bg-zinc-800 border border-[#27272a] disabled:opacity-50 rounded-lg text-xs font-semibold text-zinc-200 transition-all flex items-center justify-center gap-1.5 cursor-pointer"
              >
                Restore from Cloud
              </button>
              <button
                onclick={() => vaultState.backupLocal()}
                disabled={vaultState.syncing}
                class="w-1/2 px-4 py-2 bg-zinc-900 hover:bg-zinc-800 border border-[#27272a] disabled:opacity-50 rounded-lg text-xs font-semibold text-zinc-200 transition-all flex items-center justify-center gap-1.5 cursor-pointer"
              >
                Backup to Cloud
              </button>
            </div>
          </div>

          <!-- Disconnect Buttons -->
          <div class="flex gap-2 pt-4 border-t border-[#27272a]/60">
            <button
              onclick={() => vaultState.signOutGDrive()}
              class="w-1/2 px-4 py-2 bg-zinc-900 hover:bg-red-950/20 border border-[#27272a] hover:border-red-900/30 text-zinc-300 hover:text-red-400 rounded-lg text-xs font-semibold transition-all flex items-center justify-center gap-1.5 cursor-pointer"
            >
              <LogOut class="h-3.5 w-3.5" /> Disconnect Sync
            </button>
            <button
              onclick={() => {
                showConfirm(
                  "Wipe Cloud Data?",
                  "This will permanently delete the backup file 'vault.db' from your Google Drive. This cannot be undone.",
                  async () => {
                    await vaultState.wipeCloud();
                  }
                );
              }}
              class="w-1/2 px-4 py-2 bg-zinc-900 hover:bg-red-950/20 border border-[#27272a] hover:border-red-900/30 text-zinc-300 hover:text-red-400 rounded-lg text-xs font-semibold transition-all flex items-center justify-center gap-1.5 cursor-pointer"
            >
              Wipe Cloud Data
            </button>
          </div>
        </div>
      {/if}

      {#if vaultState.syncError}
        <div class="p-3 bg-red-500/10 border border-red-500/20 text-red-400 rounded-lg text-xs flex items-center gap-2">
          <AlertTriangle class="h-4 w-4 shrink-0 text-red-400" />
          <span>{vaultState.syncError}</span>
        </div>
      {/if}
    </div>
  </div>
</div>
