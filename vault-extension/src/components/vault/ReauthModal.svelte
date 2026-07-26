<script lang="ts">
import { Lock } from 'lucide-svelte';
import { Button } from '~/components/ui/button/index.js';
import { Input } from '~/components/ui/input/index.js';
import { Label } from '~/components/ui/label/index.js';
import { reauthController } from '~/lib/reauth-state.svelte.js';

let password = $state('');
let isVerifying = $state(false);

const handleVerify = async (e: Event) => {
  e.preventDefault();
  if (!password.trim()) return;

  isVerifying = true;
  const success = await reauthController.verifyPassword(password);
  isVerifying = false;

  if (success) {
    password = '';
  }
};

const handleCancel = () => {
  password = '';
  reauthController.cancel();
};
</script>

{#if reauthController.showModal}
    <div
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4"
    >
        <div
            class="w-full max-w-sm bg-zinc-950 border border-zinc-800 rounded-xl p-5 shadow-2xl space-y-4 text-zinc-200"
        >
            <div class="flex items-center gap-2 border-b border-zinc-900 pb-3">
                <Lock class="h-4.5 w-4.5 text-accent" />
                <h3 class="text-sm font-semibold tracking-tight">
                    Security Authentication
                </h3>
            </div>

            <p class="text-xs text-zinc-400 leading-normal">
                Please enter your Master Password to complete this action.
            </p>

            <form onsubmit={handleVerify} class="space-y-4">
                <div class="space-y-1.5">
                    <Label
                        for="reauth-password"
                        class="text-[10px] font-semibold text-zinc-500 uppercase tracking-wider"
                        >Master Password</Label
                    >
                    <Input
                        id="reauth-password"
                        type="password"
                        placeholder="Enter master password..."
                        bind:value={password}
                        class="bg-zinc-900 border-zinc-800 text-white text-xs h-9 focus-visible:ring-1 focus-visible:ring-accent"
                        required
                        disabled={isVerifying}
                    />
                </div>

                {#if reauthController.errorMsg}
                    <div
                        class="p-2.5 bg-red-950/20 border border-red-900/50 text-red-400 rounded-lg text-[11px] leading-tight"
                    >
                        {reauthController.errorMsg}
                    </div>
                {/if}

                <div class="flex gap-2 pt-2">
                    <Button
                        type="button"
                        variant="outline"
                        class="w-1/2 bg-zinc-900 border-zinc-800 text-zinc-300 hover:text-white text-xs h-9"
                        onclick={handleCancel}
                        disabled={isVerifying}
                    >
                        Cancel
                    </Button>
                    <Button
                        type="submit"
                        class="w-1/2 bg-zinc-50 text-zinc-950 hover:bg-zinc-200 text-xs h-9 font-semibold flex items-center justify-center gap-1.5"
                        disabled={isVerifying}
                    >
                        {#if isVerifying}
                            <div
                                class="animate-spin rounded-full h-3.5 w-3.5 border-2 border-zinc-950 border-t-transparent"
                            ></div>
                        {:else}
                            Verify
                        {/if}
                    </Button>
                </div>
            </form>
        </div>
    </div>
{/if}
