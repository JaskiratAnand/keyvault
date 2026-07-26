<script lang="ts">
import { Trash2 } from 'lucide-svelte';
import type { VaultItem } from '../lib/types.js';
import { getVaultContext } from '../lib/vault-state.svelte.js';

interface Props {
  showConfirm: (
    title: string,
    message: string,
    onConfirm: () => void | Promise<void>,
  ) => void;
  selectedTrashEntry: VaultItem | null;
}

let { showConfirm, selectedTrashEntry = $bindable() }: Props = $props();

const vaultState = getVaultContext();

function handleEmptyTrash() {
  showConfirm(
    'Empty Trash?',
    'Are you sure you want to permanently empty the trash? All deleted credentials will be destroyed forever.',
    async () => {
      if (vaultState.vault?.trash) {
        for (const entry of [...vaultState.vault.trash]) {
          await vaultState.purgeItem(entry.id);
        }
        selectedTrashEntry = null;
      }
    },
  );
}
</script>

<div class="grow flex overflow-hidden">
  <!-- Trash Sidebar List -->
  <div class="w-80 border-r border-[#27272a] flex flex-col justify-between shrink-0 bg-[#09090b]">
    <div class="p-4 border-b border-[#27272a] flex justify-between items-center shrink-0 animate-fade-in">
      <span class="text-zinc-400 font-semibold uppercase tracking-wider text-xs">
        {vaultState.vault?.trash?.length || 0} DELETED ITEMS
      </span>
      {#if vaultState.vault?.trash && vaultState.vault.trash.length > 0}
        <button
          onclick={handleEmptyTrash}
          class="bg-red-950/30 border border-red-900/50 text-red-200 hover:bg-red-900/50 hover:text-white h-7 px-2.5 text-[10px] uppercase font-bold flex items-center gap-1 transition-colors cursor-pointer rounded-md"
        >
          Empty Trash
        </button>
      {/if}
    </div>

    <div class="grow overflow-y-auto p-3 space-y-2">
      {#if !vaultState.vault?.trash || vaultState.vault.trash.length === 0}
        <div class="flex flex-col items-center justify-center py-20 text-zinc-500 gap-2 select-none">
          <span class="text-3xl">🗑️</span>
          <p class="text-xs font-semibold">Trash is empty</p>
        </div>
      {:else}
        {#each vaultState.vault.trash as entry (entry.id)}
          <button
            class="w-full flex items-center gap-3 p-3 rounded-lg border border-transparent text-left transition-all select-none min-w-0 cursor-pointer bg-transparent {selectedTrashEntry?.id === entry.id ? 'bg-[#18181b] border-[#06b6d4]/40' : 'hover:bg-[#18181b]/50'}"
            onclick={() => selectedTrashEntry = entry}
          >
            <div class="h-8 w-8 rounded-full bg-zinc-900 border border-zinc-800 text-zinc-500 flex items-center justify-center font-bold uppercase text-xs shrink-0">
              {entry.type === 'SecureNote' ? '📝' : (entry.title.trim()[0] || '🔑')}
            </div>
            <div class="flex flex-col min-w-0">
              <div class="flex items-center gap-1.5 min-w-0">
                <span class="text-xs font-semibold text-zinc-400 truncate">{entry.title}</span>
                {#if entry.type === 'SecureNote'}
                  <span class="bg-amber-500/10 text-amber-500 border border-amber-500/10 text-[8px] font-bold uppercase tracking-wider px-1 py-0.2 rounded shrink-0 opacity-70">Note</span>
                {:else}
                  <span class="bg-cyan-500/10 text-cyan-500 border border-cyan-500/10 text-[8px] font-bold uppercase tracking-wider px-1 py-0.2 rounded shrink-0 opacity-70">Login</span>
                {/if}
              </div>
              <span class="text-[10px] text-zinc-500 truncate">
                {entry.type === 'SecureNote' ? 'Secure Note' : `${entry.accounts?.length || 0} accounts`}
              </span>
            </div>
          </button>
        {/each}
      {/if}
    </div>
  </div>

  <!-- Trash details / Restore panel -->
  <div class="grow overflow-y-auto p-8 flex flex-col justify-between">
    {#if !selectedTrashEntry}
      <div class="m-auto flex flex-col items-center text-center max-w-sm space-y-4 py-20 select-none animate-fade-in">
        <div class="h-12 w-12 rounded-full bg-[#18181b] border border-[#27272a] flex items-center justify-center text-zinc-500">
          <Trash2 class="h-6 w-6" />
        </div>
        <div class="space-y-1">
          <h3 class="text-sm font-semibold text-white">No Deleted Item Selected</h3>
          <p class="text-xs text-zinc-500 leading-relaxed">
            Select a soft-deleted item from the sidebar list to view details, restore it, or permanently destroy it.
          </p>
        </div>
      </div>
    {:else}
      <div class="max-w-xl mx-auto w-full space-y-6 animate-fade-in">
        <!-- Warning banner -->
        <div class="p-3 bg-red-950/20 border border-red-800/40 text-red-300 rounded-lg text-xs flex items-center gap-2">
          <Trash2 class="h-4 w-4 text-red-400" />
          <span>This item is in the Trash and will not auto-fill.</span>
        </div>

        <!-- Header Title -->
        <div>
          <h2 class="text-lg font-bold text-white tracking-tight">{selectedTrashEntry.title}</h2>
          <p class="text-xs text-zinc-500 mt-1 font-mono">ID: {selectedTrashEntry.id}</p>
        </div>

        <!-- Static details -->
        <div class="space-y-4">
          {#if selectedTrashEntry.type === 'DomainGroup'}
            {#each selectedTrashEntry.accounts || [] as account}
              <div class="p-3 bg-[#18181b]/40 border border-[#27272a] rounded-lg space-y-2">
                <div class="flex justify-between items-center text-xs">
                  <span class="text-zinc-400 font-medium">Username: {account.username || '(No Username)'}</span>
                </div>
                <div class="flex justify-between items-center text-xs">
                  <span class="text-zinc-400">Password: ••••••••</span>
                </div>
                {#if account.notes}
                  <p class="text-[10px] text-zinc-500 font-sans italic">{account.notes}</p>
                {/if}
              </div>
            {/each}
            {#if selectedTrashEntry.urls?.[0]}
              <div class="space-y-1">
                <span class="text-[9px] font-bold text-zinc-500 uppercase tracking-wider block">Website URL</span>
                <input readonly value={selectedTrashEntry.urls[0]} class="w-full bg-[#18181b]/10 border border-[#27272a] rounded-lg px-3 py-2 text-xs text-zinc-400 outline-none" />
              </div>
            {/if}
          {:else}
            <div class="space-y-1">
              <span class="text-[9px] font-bold text-zinc-500 uppercase tracking-wider block">Secure Note Body</span>
              <textarea readonly class="w-full bg-[#18181b]/10 border border-[#27272a] rounded-lg text-zinc-400 p-3 text-xs h-64 resize-none focus:outline-none">{selectedTrashEntry.notes}</textarea>
            </div>
          {/if}
        </div>

        <!-- Actions -->
        <div class="border-t border-[#27272a] pt-6 flex gap-4">
          <button
            onclick={async () => {
              if (selectedTrashEntry) {
                await vaultState.restoreItem(selectedTrashEntry.id);
                selectedTrashEntry = null;
              }
            }}
            class="w-1/2 bg-[#fafafa] hover:bg-white text-[#18181b] text-xs font-bold py-2.5 rounded-lg transition-colors cursor-pointer border-0"
          >
            Restore Entry
          </button>
          <button
            onclick={() => {
              if (selectedTrashEntry) {
                showConfirm(
                  "Delete Permanently?",
                  "Are you sure you want to permanently delete this entry from the vault? This cannot be undone.",
                  async () => {
                    if (selectedTrashEntry) {
                      await vaultState.purgeItem(selectedTrashEntry.id);
                      selectedTrashEntry = null;
                    }
                  }
                );
              }
            }}
            class="w-1/2 bg-red-950/20 border border-red-800/40 text-red-300 hover:bg-red-900/40 hover:text-white text-xs font-bold py-2.5 rounded-lg transition-colors cursor-pointer border-0"
          >
            Delete Permanently
          </button>
        </div>
      </div>
    {/if}
  </div>
</div>
