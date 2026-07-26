<script lang="ts">
import {
  ArrowLeft,
  Check,
  Copy,
  Globe,
  Key,
  LogOut,
  Plus,
  Search,
  Tag,
} from 'lucide-svelte';
import { Button } from '~/components/ui/button/index.js';
import { Input } from '~/components/ui/input/index.js';
import { reauthController } from '~/lib/reauth-state.svelte.js';
import {
  type Account,
  type DomainGroup,
  type SecureNote,
  type VaultItem,
  vaultState,
} from '~/lib/vault-state.svelte.js';

interface Props {
  activePanel: 'list' | 'add' | 'detail';
  // biome-ignore lint/suspicious/noExplicitAny: template compatibility
  selectedEntry: any | null;
  selectedGroup: DomainGroup | null;
}

let {
  activePanel = $bindable(),
  selectedEntry = $bindable(),
  selectedGroup = $bindable(),
}: Props = $props();

let searchQuery = $state('');
let copiedStates = $state<Record<string, boolean>>({});

// Filtered items derived rune (searches titles, URLs, and account usernames)
let filteredItems = $derived.by(() => {
  const q = searchQuery.toLowerCase().trim();
  const items = vaultState.vault.items || [];
  if (!q) return items;

  return items.filter((item) => {
    if (item.type === 'SecureNote') {
      return (
        item.title.toLowerCase().includes(q) ||
        item.notes.toLowerCase().includes(q)
      );
    } else {
      return (
        item.title.toLowerCase().includes(q) ||
        (item.urls || []).some((u) => u.toLowerCase().includes(q)) ||
        (item.accounts || []).some((a) => a.username?.toLowerCase().includes(q))
      );
    }
  });
});

const triggerCopy = async (text: string | undefined, key: string) => {
  if (!text) return;
  try {
    await navigator.clipboard.writeText(text);
    copiedStates[key] = true;

    // Auto-clear clipboard after 30 seconds
    setTimeout(() => {
      navigator.clipboard.writeText('');
    }, 30000);

    // Reset copied indicator after 2 seconds
    setTimeout(() => {
      copiedStates[key] = false;
    }, 2000);
  } catch (e) {
    console.error('Failed to copy to clipboard:', e);
  }
};
</script>

<div class="flex flex-col h-full justify-between overflow-hidden">
    <!-- Header/Search Area -->
    <div class="space-y-3 p-1">
        {#if selectedGroup}
            <!-- Back navigation for Group View -->
            <div class="flex items-center gap-2 border-b border-zinc-900 pb-2.5">
                <Button
                    variant="ghost"
                    size="icon"
                    class="h-7 w-7 text-zinc-400 hover:text-white"
                    onclick={() => (selectedGroup = null)}
                >
                    <ArrowLeft class="h-4 w-4" />
                </Button>
                <div class="flex flex-col min-w-0">
                    <span class="text-xs font-semibold text-white truncate">{selectedGroup.title}</span>
                    {#if selectedGroup.urls[0]}
                        <span class="text-[9px] text-zinc-500 truncate">{selectedGroup.urls[0]}</span>
                    {/if}
                </div>
            </div>
        {:else}
            <!-- Main Search Input -->
            <div class="relative flex items-center">
                <Search class="absolute left-3 h-4 w-4 text-zinc-500" />
                <Input
                    type="text"
                    placeholder="Search vault..."
                    bind:value={searchQuery}
                    class="bg-zinc-900 border-zinc-800 text-white pl-9 text-xs h-9"
                />
            </div>

            <!-- Stats/Add Row -->
            <div class="flex justify-between items-center text-xs">
                <span class="text-zinc-400 font-semibold uppercase tracking-wider"
                    >{filteredItems.length} groups / notes</span
                >
                <Button
                    variant="outline"
                    size="sm"
                    class="bg-zinc-900 border-zinc-800 text-zinc-300 hover:text-white h-7 text-xs flex items-center gap-1"
                    onclick={() => (activePanel = "add")}
                >
                    <Plus class="h-3 w-3" /> Add
                </Button>
            </div>
        {/if}
    </div>

    <!-- Scrollable Items List -->
    <div class="grow min-h-0 overflow-y-auto my-3 pr-1 space-y-2 pb-2">
        {#if selectedGroup}
            <!-- Accounts List inside the Selected Domain Group -->
            {#each selectedGroup.accounts as account (account.id)}
                <div
                    class="flex items-center justify-between p-2 bg-zinc-900/40 border border-zinc-900 hover:border-zinc-800 rounded-lg transition-all duration-200"
                >
                    <button
                        class="flex items-center gap-3 grow text-left outline-none select-none min-w-0"
                        onclick={() => {
                            selectedEntry = account;
                            activePanel = "detail";
                        }}
                    >
                        <div class="h-8 w-8 rounded-full bg-zinc-800/80 border border-zinc-700/60 text-zinc-300 flex items-center justify-center font-semibold uppercase text-xs shrink-0">
                            👤
                        </div>
                        <div class="flex flex-col min-w-0">
                            <span class="text-xs font-semibold text-white truncate">
                                {account.username || "No Username"}
                            </span>
                            <span class="text-[9px] text-zinc-500 truncate">
                                Click to view account details
                            </span>
                        </div>
                    </button>

                    <div class="flex items-center gap-0.5 ml-2 shrink-0">
                        <Button
                            variant="ghost"
                            size="icon"
                            class="h-7 w-7 text-zinc-400 hover:text-white"
                            onclick={() =>
                                triggerCopy(account.username, account.id + "_u")}
                            title="Copy Username"
                        >
                            {#if copiedStates[account.id + "_u"]}
                                <Check class="h-3.5 w-3.5 text-green-400" />
                            {:else}
                                <Copy class="h-3.5 w-3.5" />
                            {/if}
                        </Button>
                        <Button
                            variant="ghost"
                            size="icon"
                            class="h-7 w-7 text-zinc-400 hover:text-white"
                            onclick={async () => {
                                const authorized =
                                    await reauthController.requestReauth();
                                if (authorized) {
                                    triggerCopy(
                                        account.password,
                                        account.id + "_p",
                                    );
                                }
                            }}
                            title="Copy Password"
                        >
                            {#if copiedStates[account.id + "_p"]}
                                <Check class="h-3.5 w-3.5 text-green-400" />
                            {:else}
                                <Copy class="h-3.5 w-3.5 text-accent" />
                            {/if}
                        </Button>
                    </div>
                </div>
            {/each}
        {:else}
            <!-- Top-Level Domain Groups & Secure Notes -->
            {#if filteredItems.length === 0}
                <div
                    class="flex flex-col items-center justify-center py-12 text-zinc-400 gap-1.5"
                >
                    <span class="text-3xl">🔍</span>
                    <p class="text-xs font-medium">No items found</p>
                </div>
            {:else}
                {#each filteredItems as item (item.id)}
                    {#if item.type === "SecureNote"}
                        <!-- SECURE NOTE CARD -->
                        <div
                            class="flex items-center justify-between p-2 bg-zinc-900/40 border border-zinc-900 hover:border-zinc-800 rounded-lg transition-all duration-200"
                        >
                            <button
                                class="flex items-center gap-3 grow text-left outline-none select-none min-w-0"
                                onclick={() => {
                                    selectedGroup = null;
                                    selectedEntry = item;
                                    activePanel = "detail";
                                }}
                            >
                                <div class="h-8 w-8 rounded-full bg-zinc-800 border border-zinc-700 text-zinc-300 flex items-center justify-center font-semibold uppercase text-xs shrink-0">
                                    📝
                                </div>
                                <div class="flex flex-col min-w-0">
                                    <div class="flex items-center gap-1.5 min-w-0">
                                        <span class="text-xs font-semibold text-white truncate">{item.title}</span>
                                        <span class="bg-amber-500/10 text-amber-400 border border-amber-500/20 text-[8px] font-bold uppercase tracking-wider px-1 py-0.2 rounded select-none shrink-0">
                                            Note
                                        </span>
                                    </div>
                                    {#if item.tags && item.tags.length > 0}
                                        <span class="text-[9px] text-zinc-500 truncate flex items-center gap-1 mt-0.5">
                                            <Tag class="h-2.5 w-2.5 text-accent" /> {item.tags.join(', ')}
                                        </span>
                                    {/if}
                                </div>
                            </button>
                        </div>
                    {:else}
                        <!-- DOMAIN GROUP CARD -->
                        <div
                            class="flex items-center justify-between p-2 bg-zinc-900/40 border border-zinc-900 hover:border-zinc-800 rounded-lg transition-all duration-200"
                        >
                            <button
                                class="flex items-center gap-3 grow text-left outline-none select-none min-w-0"
                                onclick={() => {
                                    selectedGroup = item;
                                    selectedEntry = null;
                                    activePanel = "detail";
                                }}
                            >
                                <div class="h-8 w-8 rounded-full bg-zinc-800 border border-zinc-700 text-zinc-300 flex items-center justify-center font-semibold uppercase text-xs shrink-0">
                                    🌐
                                </div>
                                <div class="flex flex-col min-w-0">
                                    <div class="flex items-center gap-1.5 min-w-0">
                                        <span class="text-xs font-semibold text-white truncate">{item.title}</span>
                                        <span class="bg-cyan-500/10 text-cyan-400 border border-cyan-500/20 text-[8px] font-bold uppercase tracking-wider px-1 py-0.2 rounded select-none shrink-0">
                                            {item.accounts.length} {item.accounts.length === 1 ? 'account' : 'accounts'}
                                        </span>
                                    </div>
                                    {#if item.tags && item.tags.length > 0}
                                        <span class="text-[9px] text-zinc-500 truncate flex items-center gap-1 mt-0.5">
                                            <Tag class="h-2.5 w-2.5 text-accent" /> {item.tags.join(', ')}
                                        </span>
                                    {:else if item.urls[0]}
                                        <span class="text-[9px] text-zinc-500 truncate mt-0.5">
                                            {item.urls[0].replace(/^https?:\/\/(www\.)?/, '')}
                                        </span>
                                    {/if}
                                </div>
                            </button>
                        </div>
                    {/if}
                {/each}
            {/if}
        {/if}
    </div>

    <!-- Footer Actions -->
    <div class="grid grid-cols-2 gap-2 border-t border-zinc-900 pt-3">
        <Button
            variant="outline"
            class="w-full bg-zinc-900 border-zinc-800 text-zinc-300 hover:text-white text-xs h-9"
            onclick={() => browser.runtime.openOptionsPage()}
        >
            Dashboard
        </Button>
        <Button
            variant="outline"
            class="w-full bg-zinc-900 border-zinc-800 text-zinc-300 text-xs h-9 flex items-center justify-center gap-1.5 hover:border-red-950 hover:text-red-400"
            onclick={() => vaultState.lock()}
        >
            <LogOut class="h-3.5 w-3.5" /> Lock
        </Button>
    </div>
</div>
