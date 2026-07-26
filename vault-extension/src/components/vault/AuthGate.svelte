<script lang="ts">
import { Eye, EyeOff, Globe } from 'lucide-svelte';
import { Button } from '~/components/ui/button/index.js';
import { Input } from '~/components/ui/input/index.js';
import { Label } from '~/components/ui/label/index.js';
import { vaultState } from '~/lib/vault-state.svelte.js';
import SetupWizard from './SetupWizard.svelte';

let password = $state('');
let showPassword = $state(false);

const isPopup =
  typeof window !== 'undefined' && window.location.pathname.includes('popup');

const openDashboard = () => {
  if (typeof browser !== 'undefined' && browser.runtime?.openOptionsPage) {
    browser.runtime.openOptionsPage();
  }
};

const handleUnlock = async () => {
  await vaultState.unlock(password);
  if (vaultState.isUnlocked) {
    password = '';
  }
};
</script>

<div class="flex flex-col justify-center w-full py-4 space-y-5 text-zinc-100">
  {#if !vaultState.isRegistered}
    <SetupWizard />
  {:else}
    <!-- UNLOCK SCREEN -->
    <div class="space-y-1.5">
      <h2 class="text-base font-bold text-white tracking-tight">Unlock Vault</h2>
      <p class="text-xs text-zinc-400 leading-relaxed">
        Enter your master password to derive the key and decrypt your stored credentials locally.
      </p>
    </div>

    <div class="space-y-4">
      <div class="space-y-1.5">
        <Label for="unlock-password" class="text-xs font-semibold text-zinc-400 uppercase tracking-wider">Master Password</Label>
        <div class="relative">
          <Input
            id="unlock-password"
            type={showPassword ? "text" : "password"}
            bind:value={password}
            placeholder="Enter master password"
            class="bg-zinc-950 border-zinc-800 text-white pr-10 text-xs h-9 focus-visible:ring-1 focus-visible:ring-zinc-700"
            onkeydown={(e) => e.key === "Enter" && handleUnlock()}
          />
          <Button
            variant="ghost"
            size="icon"
            class="h-7 w-7 absolute right-1.5 top-1/2 -translate-y-1/2 text-zinc-400 hover:text-white"
            onclick={() => (showPassword = !showPassword)}
          >
            {#if showPassword}
              <EyeOff class="h-3.5 w-3.5" />
            {:else}
              <Eye class="h-3.5 w-3.5" />
            {/if}
          </Button>
        </div>
      </div>

      {#if vaultState.error}
        <div class="p-3 bg-red-950/30 border border-red-800/50 text-red-400 rounded-md text-xs">
          {vaultState.error}
        </div>
      {/if}

      <Button class="w-full bg-zinc-50 text-zinc-950 hover:bg-zinc-200 text-xs font-semibold h-9 transition-colors" onclick={handleUnlock}>
        Unlock Vault
      </Button>

      {#if isPopup}
        <Button
          variant="outline"
          class="w-full bg-zinc-900 border-zinc-800 text-zinc-300 hover:text-white text-xs font-semibold h-9 transition-colors mt-2 flex items-center justify-center gap-1.5"
          onclick={openDashboard}
        >
          <Globe class="h-3.5 w-3.5" /> Open Web Dashboard
        </Button>
      {/if}
    </div>
  {/if}
</div>
