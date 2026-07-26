<script lang="ts">
import { Lock, ShieldCheck } from 'lucide-svelte';
import { onMount } from 'svelte';
import * as Tabs from '~/components/ui/tabs/index.js';
import AddCredential from '~/components/vault/AddCredential.svelte';
import AuthGate from '~/components/vault/AuthGate.svelte';
import CredentialDetail from '~/components/vault/CredentialDetail.svelte';
import CredentialsList from '~/components/vault/CredentialsList.svelte';
import PasswordGenerator from '~/components/vault/PasswordGenerator.svelte';
import ReauthModal from '~/components/vault/ReauthModal.svelte';
import {
  type Account,
  type DomainGroup,
  type SecureNote,
  vaultState,
} from '~/lib/vault-state.svelte.js';

let activePanel = $state<'list' | 'add' | 'detail'>('list');
// biome-ignore lint/suspicious/noExplicitAny: template compatibility
let selectedEntry = $state<any | null>(null);
// biome-ignore lint/suspicious/noExplicitAny: template compatibility
let selectedGroup = $state<any | null>(null);

const openSetup = () => {
  if (typeof browser !== 'undefined' && browser.runtime?.openOptionsPage) {
    browser.runtime.openOptionsPage().catch((err) => {
      console.error('Failed to open options page:', err);
    });
  }
};

$effect(() => {
  if (vaultState.wasmReady && !vaultState.isRegistered) {
    openSetup();
  }
});

onMount(() => {
  vaultState.initWasm();
});
</script>

<main class="container">
    <header class="header">
        <div class="logo-area">
            <Lock class="h-4.5 w-4.5 text-zinc-200" />
            <h1 class="title">KeyVault</h1>
        </div>

        <div
            class="flex items-center gap-1.5 px-2.5 py-1 bg-zinc-900 border border-zinc-800 rounded-full select-none"
        >
            <ShieldCheck class="h-3 w-3 text-green-400" />
            <span
                class="text-[9px] font-semibold text-zinc-300 uppercase tracking-wider"
                >Secure WASM</span
            >
        </div>
    </header>

    {#if !vaultState.wasmReady}
        <div class="grow flex flex-col items-center justify-center gap-4">
            <div
                class="animate-spin rounded-full h-8 w-8 border-2 border-zinc-800 border-t-zinc-400"
            ></div>
            <p class="text-xs text-zinc-400">
                Loading secure cryptographic engine...
            </p>
        </div>
    {:else if !vaultState.isRegistered}
        <div class="grow flex flex-col items-center justify-center text-center p-6 gap-6 bg-[#09090b]">
            <div class="flex items-center justify-center w-12 h-12 rounded-full bg-zinc-900 border border-zinc-800">
                <Lock class="h-6 w-6 text-[#06b6d4]" />
            </div>
            <div class="space-y-2">
                <h2 class="text-sm font-bold text-zinc-100 tracking-tight">Setup Required</h2>
                <p class="text-xs text-zinc-400 leading-relaxed max-w-[240px]">
                    Please complete the initial configuration in the dashboard to secure your vault.
                </p>
            </div>
            <button
                onclick={openSetup}
                class="w-full py-2 px-3 text-xs font-semibold bg-[#fafafa] text-[#18181b] rounded-lg hover:bg-zinc-200 active:scale-[0.98] transition-all duration-150 cursor-pointer"
            >
                Open Setup in Web Dashboard
            </button>
        </div>
    {:else if !vaultState.isUnlocked}
        <div class="grow flex items-center">
            <AuthGate />
        </div>
    {:else}
        <Tabs.Root
            value="vault"
            class="w-full grow flex flex-col justify-between min-h-0"
        >
            <Tabs.List
                class="grid w-full grid-cols-2 bg-zinc-900 border border-zinc-800/80 p-0.5 rounded-lg mb-3"
            >
                <Tabs.Trigger
                    value="vault"
                    class="text-xs py-1 h-7 transition-all data-[state=active]:bg-zinc-800 data-[state=active]:text-white data-[state=active]:font-semibold"
                    >Vault</Tabs.Trigger
                >
                <Tabs.Trigger
                    value="generator"
                    class="text-xs py-1 h-7 transition-all data-[state=active]:bg-zinc-800 data-[state=active]:text-white data-[state=active]:font-semibold"
                    >Generator</Tabs.Trigger
                >
            </Tabs.List>

            <Tabs.Content
                value="vault"
                class="grow flex flex-col focus:outline-none overflow-hidden min-h-0"
            >
                {#if activePanel === "list"}
                    <CredentialsList bind:activePanel bind:selectedEntry bind:selectedGroup />
                {:else if activePanel === "add"}
                    <AddCredential bind:activePanel />
                {:else if activePanel === "detail" && (selectedEntry || selectedGroup)}
                    <CredentialDetail bind:activePanel bind:selectedEntry bind:selectedGroup />
                {/if}
            </Tabs.Content>

            <Tabs.Content
                value="generator"
                class="grow flex flex-col focus:outline-none overflow-hidden min-h-0"
            >
                <PasswordGenerator />
            </Tabs.Content>
        </Tabs.Root>
    {/if}
    <ReauthModal />
</main>

<style>
    :global(html), :global(body), :global(#app) {
        margin: 0;
        font-family:
            -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica,
            Arial, sans-serif;
        background-color: #09090b;
        color: #f3f4f6;
        width: 360px;
        height: 500px;
        overflow: hidden;
    }

    .container {
        box-sizing: border-box;
        padding: 16px;
        height: 100%;
        display: flex;
        flex-direction: column;
        gap: 12px;
    }

    .header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        border-bottom: 1px solid #18181b;
        padding-bottom: 10px;
    }

    .logo-area {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .title {
        font-size: 15px;
        font-weight: 700;
        margin: 0;
        color: #ffffff;
        letter-spacing: -0.025em;
    }
</style>
