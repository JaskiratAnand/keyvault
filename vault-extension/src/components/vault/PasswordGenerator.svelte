<script lang="ts">
import { Check, Copy, RefreshCw } from 'lucide-svelte';
import { onMount } from 'svelte';
import { Button } from '~/components/ui/button/index.js';
import { Label } from '~/components/ui/label/index.js';
import { vaultState } from '~/lib/vault-state.svelte.js';

let mode = $state<'Password' | 'Passphrase' | 'Pin'>('Password');
let length = $state(16);
let wordCount = $state(4);
let separator = $state('-');
let pinLength = $state(6);

// Min requirement counts state
let minUppercase = $state(1);
let minLowercase = $state(1);
let minNumbers = $state(1);
let minSymbols = $state(1);

// Dynamically derived max values based on other sliders' current values
let maxUppercase = $derived(
  Math.max(0, length - (minLowercase + minNumbers + minSymbols)),
);
let maxLowercase = $derived(
  Math.max(0, length - (minUppercase + minNumbers + minSymbols)),
);
let maxNumbers = $derived(
  Math.max(0, length - (minUppercase + minLowercase + minSymbols)),
);
let maxSymbols = $derived(
  Math.max(0, length - (minUppercase + minLowercase + minNumbers)),
);

let generatedText = $state('');
let entropy = $state(0);
let copied = $state(false);
let isRegenerating = $state(false);

const generate = () => {
  isRegenerating = true;
  setTimeout(() => {
    isRegenerating = false;
  }, 150);

  let config: Record<string, unknown>;

  if (mode === 'Password') {
    config = {
      type: 'Character',
      length,
      min_uppercase: minUppercase,
      min_lowercase: minLowercase,
      min_numbers: minNumbers,
      min_symbols: minSymbols,
      exclude_ambiguous: false,
    };
  } else if (mode === 'Passphrase') {
    let mappedSeparator = 'Hyphen';
    if (separator === ' ') mappedSeparator = 'Space';
    else if (separator === '_') mappedSeparator = 'Underscore';
    else if (separator === '.') mappedSeparator = 'Period';
    else if (separator === '') mappedSeparator = 'None';

    config = {
      type: 'Passphrase',
      words: wordCount,
      separator: mappedSeparator,
      capitalization: 'Lowercase',
    };
  } else {
    config = {
      type: 'Pin',
      length: pinLength,
    };
  }

  const result = vaultState.generateCredential(config);
  if (result) {
    generatedText = result.credential;
    entropy = Number(result.entropy.toFixed(1));
  }
};

const handleCopy = async () => {
  if (!generatedText) return;
  try {
    await navigator.clipboard.writeText(generatedText);
    copied = true;
    setTimeout(() => {
      copied = false;
    }, 2000);
  } catch (err) {
    console.error('Failed to copy generated password:', err);
  }
};

onMount(() => {
  generate();
});

// Clamp sliders to their derived maximum bounds if length changes
$effect(() => {
  if (minUppercase > maxUppercase) minUppercase = maxUppercase;
  if (minLowercase > maxLowercase) minLowercase = maxLowercase;
  if (minNumbers > maxNumbers) minNumbers = maxNumbers;
  if (minSymbols > maxSymbols) minSymbols = maxSymbols;
});

// Subscribe to dependencies and generate
$effect(() => {
  const _m = mode;
  const _l = length;
  const _wc = wordCount;
  const _sep = separator;
  const _u = minUppercase;
  const _low = minLowercase;
  const _d = minNumbers;
  const _s = minSymbols;
  const _pl = pinLength;
  generate();
});
</script>

<div class="flex flex-col h-full justify-between text-zinc-100 overflow-hidden">
    <div class="space-y-4 overflow-y-auto pr-1 grow">
        <!-- Generated Output Display -->
        <div
            class="relative bg-zinc-950 border border-zinc-800 rounded-lg p-3.5 flex items-center justify-between min-h-12 transition-all duration-200 hover:border-zinc-700 gap-3"
        >
            <span
                class="text-xs font-mono break-all text-white select-all tracking-wider leading-relaxed grow transition-opacity duration-150"
                class:opacity-50={isRegenerating}
                >{generatedText || "Generating..."}</span
            >
            <div class="flex items-center gap-0.5 shrink-0">
                <Button
                    variant="ghost"
                    size="icon"
                    class="h-8 w-8 text-zinc-400 hover:text-white"
                    onclick={generate}
                    title="Regenerate"
                >
                    <RefreshCw
                        class="h-4 w-4 {isRegenerating ? 'animate-spin' : ''}"
                    />
                </Button>
                <Button
                    variant="ghost"
                    size="icon"
                    class="h-8 w-8 text-zinc-400 hover:text-white"
                    onclick={handleCopy}
                    title="Copy Password"
                >
                    {#if copied}
                        <Check class="h-4 w-4 text-green-400" />
                    {:else}
                        <Copy class="h-4 w-4" />
                    {/if}
                </Button>
            </div>
        </div>

        <!-- Strength Meter -->
        <div
            class="space-y-2 bg-zinc-900/20 border border-zinc-800/40 p-3 rounded-lg"
        >
            <div
                class="flex justify-between items-center text-[11px] font-medium text-zinc-400"
            >
                <span>Entropy Strength</span>
                <span
                    class="font-bold uppercase tracking-wider text-[10px]"
                    class:text-red-400={entropy < 40}
                    class:text-amber-400={entropy >= 40 && entropy < 60}
                    class:text-emerald-400={entropy >= 60}
                >
                    {entropy} bits ({entropy < 40
                        ? "Weak"
                        : entropy < 60
                          ? "Medium"
                          : "Strong"})
                </span>
            </div>
            <!-- Segmented strength bar -->
            <div class="grid grid-cols-3 gap-1.5">
                <div
                    class="h-1.5 rounded-full transition-all duration-300 {entropy >
                    0
                        ? entropy < 40
                            ? 'bg-red-500'
                            : entropy < 60
                              ? 'bg-amber-500'
                              : 'bg-emerald-500'
                        : 'bg-zinc-900'}"
                ></div>
                <div
                    class="h-1.5 rounded-full transition-all duration-300 {entropy >=
                    40
                        ? entropy < 60
                            ? 'bg-amber-500'
                            : 'bg-emerald-500'
                        : 'bg-zinc-900'}"
                ></div>
                <div
                    class="h-1.5 rounded-full transition-all duration-300 {entropy >=
                    60
                        ? 'bg-emerald-500'
                        : 'bg-zinc-900'}"
                ></div>
            </div>
        </div>

        <!-- Mode Toggle -->
        <div
            class="grid grid-cols-3 gap-1 bg-zinc-950 border border-zinc-800 p-1 rounded-lg"
        >
            <button
                class="text-xs h-7.5 rounded-md font-medium transition-all duration-200 flex items-center justify-center {mode ===
                'Password'
                    ? 'bg-zinc-800 text-white shadow-sm font-semibold'
                    : 'text-zinc-400 hover:text-zinc-200'}"
                onclick={() => (mode = "Password")}
            >
                Password
            </button>
            <button
                class="text-xs h-7.5 rounded-md font-medium transition-all duration-200 flex items-center justify-center {mode ===
                'Passphrase'
                    ? 'bg-zinc-800 text-white shadow-sm font-semibold'
                    : 'text-zinc-400 hover:text-zinc-200'}"
                onclick={() => (mode = "Passphrase")}
            >
                Passphrase
            </button>
            <button
                class="text-xs h-7.5 rounded-md font-medium transition-all duration-200 flex items-center justify-center {mode ===
                'Pin'
                    ? 'bg-zinc-800 text-white shadow-sm font-semibold'
                    : 'text-zinc-400 hover:text-zinc-200'}"
                onclick={() => (mode = "Pin")}
            >
                PIN
            </button>
        </div>

        <!-- Configuration Options -->
        {#if mode === "Password"}
            <!-- Password Config -->
            <div class="space-y-4">
                <!-- Length Card -->
                <div
                    class="bg-zinc-900/30 border border-zinc-800/50 p-3 rounded-lg space-y-2.5"
                >
                    <div class="flex justify-between items-baseline">
                        <Label
                            for="len-slider"
                            class="text-[11px] font-semibold text-zinc-400 uppercase tracking-wider"
                            >Password Length</Label
                        >
                        <span class="text-xs text-white font-mono font-bold"
                            >{length} characters</span
                        >
                    </div>
                    <input
                        id="len-slider"
                        type="range"
                        min="8"
                        max="64"
                        bind:value={length}
                        class="w-full"
                    />
                </div>

                <!-- Character Requirements Card -->
                <div
                    class="bg-zinc-900/30 border border-zinc-800/50 p-3 rounded-lg space-y-3.5"
                >
                    <div
                        class="text-[11px] font-semibold text-zinc-400 uppercase tracking-wider pb-1 border-b border-zinc-800/30"
                    >
                        Character Requirements
                    </div>
                    <div
                        class="grid grid-cols-1 sm:grid-cols-2 gap-y-3.5 gap-x-6"
                    >
                        <div class="space-y-1.5">
                            <div
                                class="flex justify-between items-baseline text-[10px] font-medium text-zinc-400"
                            >
                                <span>Min Uppercase</span>
                                <span class="text-white font-mono font-bold"
                                    >{minUppercase}</span
                                >
                            </div>
                            <input
                                type="range"
                                min="0"
                                max={maxUppercase}
                                bind:value={minUppercase}
                                class="w-full"
                            />
                        </div>

                        <div class="space-y-1.5">
                            <div
                                class="flex justify-between items-baseline text-[10px] font-medium text-zinc-400"
                            >
                                <span>Min Lowercase</span>
                                <span class="text-white font-mono font-bold"
                                    >{minLowercase}</span
                                >
                            </div>
                            <input
                                type="range"
                                min="0"
                                max={maxLowercase}
                                bind:value={minLowercase}
                                class="w-full"
                            />
                        </div>

                        <div class="space-y-1.5">
                            <div
                                class="flex justify-between items-baseline text-[10px] font-medium text-zinc-400"
                            >
                                <span>Min Numbers</span>
                                <span class="text-white font-mono font-bold"
                                    >{minNumbers}</span
                                >
                            </div>
                            <input
                                type="range"
                                min="0"
                                max={maxNumbers}
                                bind:value={minNumbers}
                                class="w-full"
                            />
                        </div>

                        <div class="space-y-1.5">
                            <div
                                class="flex justify-between items-baseline text-[10px] font-medium text-zinc-400"
                            >
                                <span>Min Symbols</span>
                                <span class="text-white font-mono font-bold"
                                    >{minSymbols}</span
                                >
                            </div>
                            <input
                                type="range"
                                min="0"
                                max={maxSymbols}
                                bind:value={minSymbols}
                                class="w-full"
                            />
                        </div>
                    </div>
                </div>
            </div>
        {:else if mode === "Passphrase"}
            <!-- Passphrase Config -->
            <div class="space-y-4">
                <div
                    class="bg-zinc-900/30 border border-zinc-800/50 p-3 rounded-lg space-y-2.5"
                >
                    <div class="flex justify-between items-baseline">
                        <Label
                            for="words-slider"
                            class="text-[11px] font-semibold text-zinc-400 uppercase tracking-wider"
                            >Number of Words</Label
                        >
                        <span class="text-xs text-white font-mono font-bold"
                            >{wordCount} words</span
                        >
                    </div>
                    <input
                        id="words-slider"
                        type="range"
                        min="3"
                        max="10"
                        bind:value={wordCount}
                        class="w-full"
                    />
                </div>

                <div
                    class="bg-zinc-900/30 border border-zinc-800/50 p-3 rounded-lg space-y-1.5"
                >
                    <Label
                        for="separator-input"
                        class="text-[11px] font-semibold text-zinc-400 uppercase tracking-wider"
                        >Word Separator</Label
                    >
                    <select
                        id="separator-input"
                        bind:value={separator}
                        class="flex w-full rounded-lg border border-zinc-800 bg-zinc-900 px-2.5 py-1.5 text-xs text-white h-9 outline-none focus:border-zinc-700 cursor-pointer"
                    >
                        <option value="-">Hyphen (-)</option>
                        <option value="_">Underscore (_)</option>
                        <option value=".">Period (.)</option>
                        <option value=" ">Space ( )</option>
                        <option value="">None</option>
                    </select>
                </div>
            </div>
        {:else}
            <!-- PIN Config -->
            <div class="space-y-4">
                <div
                    class="bg-zinc-900/30 border border-zinc-800/50 p-3 rounded-lg space-y-2.5"
                >
                    <div class="flex justify-between items-baseline">
                        <Label
                            for="pin-slider"
                            class="text-[11px] font-semibold text-zinc-400 uppercase tracking-wider"
                            >PIN Length</Label
                        >
                        <span class="text-xs text-white font-mono font-bold"
                            >{pinLength} digits</span
                        >
                    </div>
                    <input
                        id="pin-slider"
                        type="range"
                        min="4"
                        max="32"
                        bind:value={pinLength}
                        class="w-full"
                    />
                </div>
            </div>
        {/if}
    </div>

    <div class="p-0 pt-2 shrink-0">
        <Button
            class="w-full bg-zinc-50 text-zinc-950 hover:bg-zinc-200 text-xs font-bold uppercase tracking-wider h-9 flex items-center justify-center gap-1.5"
            onclick={generate}
        >
            <RefreshCw
                class="h-3.5 w-3.5 {isRegenerating ? 'animate-spin' : ''}"
            /> Generate Password
        </Button>
    </div>
</div>

<style>
    /* Custom styled range inputs scoped locally for KeyVault design system */
    input[type="range"] {
        -webkit-appearance: none;
        appearance: none;
        background: transparent;
        cursor: pointer;
        height: 20px;
    }
    input[type="range"]:focus {
        outline: none;
    }
    /* Track styling */
    input[type="range"]::-webkit-slider-runnable-track {
        background: #27272a; /* bg-zinc-800 */
        height: 4px;
        border-radius: 9999px;
        transition: background 150ms ease;
    }
    input[type="range"]::-moz-range-track {
        background: #27272a;
        height: 4px;
        border-radius: 9999px;
    }
    /* Thumb styling */
    input[type="range"]::-webkit-slider-thumb {
        -webkit-appearance: none;
        appearance: none;
        margin-top: -6px; /* center thumb vertically */
        background-color: #fafafa;
        height: 16px;
        width: 16px;
        border-radius: 9999px;
        border: 2px solid #09090b;
        box-shadow: 0 0 0 1px #27272a;
        transition:
            background-color 150ms ease,
            transform 150ms ease;
    }
    input[type="range"]::-moz-range-thumb {
        background-color: #fafafa;
        height: 12px;
        width: 12px;
        border-radius: 9999px;
        border: 2px solid #09090b;
        box-shadow: 0 0 0 1px #27272a;
        transition:
            background-color 150ms ease,
            transform 150ms ease;
    }
    /* Hover thumb state */
    input[type="range"]:hover::-webkit-slider-thumb {
        background-color: #06b6d4; /* electric steel blue accent */
        transform: scale(1.15);
    }
    input[type="range"]:hover::-moz-range-thumb {
        background-color: #06b6d4;
        transform: scale(1.15);
    }
</style>
