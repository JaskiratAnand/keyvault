<script lang="ts">
import {
  ChevronLeft,
  ChevronRight,
  CircleHelp,
  FolderLock,
  Globe,
  Lock,
  Settings,
  Sparkles,
  Trash2,
} from 'lucide-svelte';
import { getVaultContext } from '../lib/vault-state.svelte.js';

interface Props {
  currentTab: 'vault' | 'generator' | 'trash' | 'sync' | 'settings' | 'help';
  onLock: () => void;
  onTabChange?: (tab: string) => void;
}

let { currentTab = $bindable(), onLock, onTabChange }: Props = $props();

const vaultState = getVaultContext();

let sidebarCollapsed = $state(
  localStorage.getItem('sidebar_collapsed') === 'true',
);

$effect(() => {
  localStorage.setItem('sidebar_collapsed', String(sidebarCollapsed));
});

function selectTab(
  tab: 'vault' | 'generator' | 'trash' | 'sync' | 'settings' | 'help',
) {
  currentTab = tab;
  if (onTabChange) {
    onTabChange(tab);
  }
}
</script>

<aside class="w-16 {sidebarCollapsed ? 'p-3' : 'lg:w-60 p-3 lg:p-4'} bg-zinc-950/50 border-r border-[#27272a] flex flex-col justify-between shrink-0 transition-all duration-200 ease-in-out">
  <div class="space-y-6">
    <!-- Branding -->
    <div class="flex items-center px-2 py-1 select-none justify-start">
      <Lock class="h-5 w-5 shrink-0 text-[#06b6d4]" />
      <span class="font-bold text-base tracking-tight text-white ml-2.5 transition-all duration-200 ease-in-out overflow-hidden whitespace-nowrap {sidebarCollapsed ? 'max-w-0 opacity-0 ml-0 pointer-events-none' : 'max-w-40 opacity-100'}"
          >Key<span class="text-[#06b6d4]">Vault</span></span
      >
    </div>

    <!-- Navigation Links -->
    <nav class="space-y-1">
      <button
          class="w-full text-left rounded-lg text-xs font-semibold flex items-center transition-colors cursor-pointer px-3 py-2 justify-start border-0 bg-transparent {currentTab === 'vault' ? 'bg-zinc-800 text-white font-bold' : 'text-zinc-400 hover:text-zinc-200'}"
          onclick={() => selectTab('vault')}
          title="My Vault"
      >
          <FolderLock class="h-4 w-4 shrink-0" />
          <span class="ml-2.5 transition-all duration-200 ease-in-out overflow-hidden whitespace-nowrap {sidebarCollapsed ? 'max-w-0 opacity-0 ml-0 pointer-events-none' : 'max-w-40 opacity-100'}">
            My Vault
          </span>
      </button>
      <button
          class="w-full text-left rounded-lg text-xs font-semibold flex items-center transition-colors cursor-pointer px-3 py-2 justify-start border-0 bg-transparent {currentTab === 'generator' ? 'bg-zinc-800 text-white font-bold' : 'text-zinc-400 hover:text-zinc-200'}"
          onclick={() => selectTab('generator')}
          title="Password Generator"
      >
          <Sparkles class="h-4 w-4 shrink-0 text-[#06b6d4]" />
          <span class="ml-2.5 transition-all duration-200 ease-in-out overflow-hidden whitespace-nowrap {sidebarCollapsed ? 'max-w-0 opacity-0 ml-0 pointer-events-none' : 'max-w-40 opacity-100'} text-[#06b6d4]">
            Generator
          </span>
      </button>
      <button
          class="w-full text-left rounded-lg text-xs font-semibold flex items-center transition-colors cursor-pointer px-3 py-2 justify-start border-0 bg-transparent {currentTab === 'trash' ? 'bg-zinc-800 text-white font-bold' : 'text-zinc-400 hover:text-zinc-200'}"
          onclick={() => selectTab('trash')}
          title="Trash Bin"
      >
          <Trash2 class="h-4 w-4 shrink-0 text-red-400" />
          <span class="ml-2.5 transition-all duration-200 ease-in-out overflow-hidden whitespace-nowrap {sidebarCollapsed ? 'max-w-0 opacity-0 ml-0 pointer-events-none' : 'max-w-40 opacity-100'} text-red-400">
            Trash
          </span>
      </button>
      <button
          class="w-full text-left rounded-lg text-xs font-semibold flex items-center transition-colors cursor-pointer px-3 py-2 justify-start border-0 bg-transparent {currentTab === 'sync' ? 'bg-zinc-800 text-white font-bold' : 'text-zinc-400 hover:text-zinc-200'}"
          onclick={() => selectTab('sync')}
          title="Google Drive Sync"
      >
          <Globe class="h-4 w-4 shrink-0" />
          <span class="ml-2.5 transition-all duration-200 ease-in-out overflow-hidden whitespace-nowrap {sidebarCollapsed ? 'max-w-0 opacity-0 ml-0 pointer-events-none' : 'max-w-40 opacity-100'}">
            Sync Config
          </span>
      </button>
      <button
          class="w-full text-left rounded-lg text-xs font-semibold flex items-center transition-colors cursor-pointer px-3 py-2 justify-start border-0 bg-transparent {currentTab === 'settings' ? 'bg-zinc-800 text-white font-bold' : 'text-zinc-400 hover:text-zinc-200'}"
          onclick={() => selectTab('settings')}
          title="Application Settings"
      >
          <Settings class="h-4 w-4 shrink-0" />
          <span class="ml-2.5 transition-all duration-200 ease-in-out overflow-hidden whitespace-nowrap {sidebarCollapsed ? 'max-w-0 opacity-0 ml-0 pointer-events-none' : 'max-w-40 opacity-100'}">
            Settings
          </span>
      </button>
      <button
          class="w-full text-left rounded-lg text-xs font-semibold flex items-center transition-colors cursor-pointer px-3 py-2 justify-start border-0 bg-transparent {currentTab === 'help' ? 'bg-zinc-800 text-white font-bold' : 'text-zinc-400 hover:text-zinc-200'}"
          onclick={() => selectTab('help')}
          title="Help"
      >
          <CircleHelp class="h-4 w-4 shrink-0" />
          <span class="ml-2.5 transition-all duration-200 ease-in-out overflow-hidden whitespace-nowrap {sidebarCollapsed ? 'max-w-0 opacity-0 ml-0 pointer-events-none' : 'max-w-40 opacity-100'}">
            Help
          </span>
      </button>
    </nav>
  </div>

  <!-- Sidebar Bottom Controls -->
  <div class="space-y-4">
    {#if vaultState.gdriveAuthenticated}
      <div class="px-2 transition-all duration-200 ease-in-out overflow-hidden whitespace-nowrap {sidebarCollapsed ? 'max-h-0 opacity-0 pointer-events-none' : 'max-h-12 opacity-100'}">
        <p class="text-[9px] text-[#a1a1aa] uppercase tracking-wider font-semibold">Cloud Connected</p>
        <p class="text-[11px] text-zinc-300 truncate" title={vaultState.gdriveEmail}>{vaultState.gdriveEmail}</p>
      </div>
    {/if}

    <div class="border-t border-[#27272a] pt-4 flex {sidebarCollapsed ? 'flex-col items-center gap-2' : 'lg:flex-row lg:items-center gap-2 flex-col'}">
      {#if sidebarCollapsed}
        <!-- Collapse / Expand Sidebar Toggle (Above Lock Vault when collapsed) -->
        <button
          onclick={() => sidebarCollapsed = !sidebarCollapsed}
          class="w-full h-9.5 rounded-lg text-zinc-400 hover:text-white border border-[#27272a] hover:border-white/10 bg-[#09090b] transition-all cursor-pointer items-center justify-center shrink-0 hidden lg:flex"
          title="Expand Sidebar"
        >
          <ChevronRight class="h-4 w-4 shrink-0 text-zinc-400" />
        </button>

        <!-- Lock Vault (Below Collapse Toggle when collapsed) -->
        <button
          onclick={onLock}
          class="w-full h-9.5 rounded-lg border border-[#27272a] bg-[#09090b] text-[#a1a1aa] hover:text-white hover:border-[#fafafa]/20 transition-all flex items-center justify-center cursor-pointer shrink-0"
          title="Lock Vault"
        >
          <Lock class="w-3.5 h-3.5 shrink-0" />
        </button>
      {:else}
        <!-- Lock Vault (Left side when expanded) -->
        <button
          onclick={onLock}
          class="grow w-full h-9.5 rounded-lg border border-[#27272a] bg-[#09090b] text-[#a1a1aa] hover:text-white hover:border-[#fafafa]/20 transition-all flex items-center cursor-pointer justify-start px-3 gap-2.5"
          title="Lock Vault"
        >
          <Lock class="w-3.5 h-3.5 shrink-0" />
          <span class="text-xs font-semibold whitespace-nowrap">Lock Vault</span>
        </button>

        <!-- Collapse / Expand Sidebar Toggle (Right of Lock Vault when expanded) -->
        <button
          onclick={() => sidebarCollapsed = !sidebarCollapsed}
          class="h-9.5 w-9.5 rounded-lg border border-[#27272a] hover:border-white/10 hover:bg-[#18181b] bg-[#09090b] text-zinc-400 hover:text-white transition-all cursor-pointer items-center justify-center shrink-0 hidden lg:flex"
          title="Collapse Sidebar"
        >
          <ChevronLeft class="h-4 w-4 shrink-0 text-zinc-400" />
        </button>
      {/if}
    </div>
  </div>
</aside>
