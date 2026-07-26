<script lang="ts">
import { ArrowLeft, Eye, EyeOff, Sparkles } from 'lucide-svelte';
import { Button } from '~/components/ui/button/index.js';
import { Input } from '~/components/ui/input/index.js';
import { Label } from '~/components/ui/label/index.js';
import { vaultState } from '~/lib/vault-state.svelte.js';

interface Props {
  activePanel: 'list' | 'add' | 'detail';
}

let { activePanel = $bindable() }: Props = $props();

// Form states
let entryType = $state<'Login' | 'SecureNote'>('Login');
let newTitle = $state('');
let newUsername = $state('');
let newPassword = $state('');
let newUrl = $state('');
let newTotpSecret = $state('');
let newNotes = $state('');
let newTagsString = $state('');

let showNewPassword = $state(false);
let globalError = $state('');

// Field-specific error states
let titleError = $state('');
let urlError = $state('');
let totpError = $state('');

// URL helper validation
const processUrl = (str: string): { isValid: boolean; formatted: string } => {
  const trimmed = str.trim();
  if (!trimmed) return { isValid: true, formatted: '' };

  let target = trimmed;
  if (!/^https?:\/\//i.test(target)) {
    target = `https://${target}`;
  }

  try {
    const parsed = new URL(target);
    const isLocal =
      parsed.hostname === 'localhost' ||
      parsed.hostname === '127.0.0.1' ||
      parsed.hostname.startsWith('192.168.');
    const hasDot = parsed.hostname.includes('.');
    return { isValid: isLocal || hasDot, formatted: target };
  } catch {
    return { isValid: false, formatted: '' };
  }
};

// TOTP Base32 helper validation
const isValidBase32 = (str: string): boolean => {
  const clean = str.replace(/[\s-]/g, '');
  if (!clean) return true;
  return /^[A-Z2-7]+=*$/i.test(clean);
};

// Debounce validation timers
let urlTimeout: ReturnType<typeof setTimeout>;
let totpTimeout: ReturnType<typeof setTimeout>;

// Svelte 5 Reactive validation effects
$effect(() => {
  const titleVal = newTitle;
  if (titleVal.trim()) {
    titleError = '';
  }
});

$effect(() => {
  const urlVal = newUrl;
  if (!urlVal.trim()) {
    urlError = '';
    return;
  }
  clearTimeout(urlTimeout);
  urlTimeout = setTimeout(() => {
    const res = processUrl(urlVal);
    if (!res.isValid) {
      urlError = 'Please enter a valid website URL.';
    } else {
      urlError = '';
    }
  }, 400);
});

$effect(() => {
  const totpVal = newTotpSecret;
  if (!totpVal.trim()) {
    totpError = '';
    return;
  }
  clearTimeout(totpTimeout);
  totpTimeout = setTimeout(() => {
    if (!isValidBase32(totpVal)) {
      totpError = 'Invalid TOTP Secret. Base32 keys only use A-Z and 2-7.';
    } else {
      totpError = '';
    }
  }, 400);
});

const handleAddEntry = async () => {
  globalError = '';
  titleError = '';
  urlError = '';
  totpError = '';

  // Synchronous immediate validation checks
  if (!newTitle.trim()) {
    titleError = 'Title is required.';
    return;
  }

  let formattedUrl = '';
  if (entryType === 'Login' && newUrl.trim()) {
    const urlResult = processUrl(newUrl);
    if (!urlResult.isValid) {
      urlError = 'Please enter a valid website URL.';
      return;
    }
    formattedUrl = urlResult.formatted;
  }

  if (entryType === 'Login' && newTotpSecret.trim()) {
    if (!isValidBase32(newTotpSecret)) {
      totpError = 'Invalid TOTP Secret. Base32 keys only use A-Z and 2-7.';
      return;
    }
  }

  // Parse comma-separated tags
  const tags = newTagsString
    .split(',')
    .map((t) => t.trim())
    .filter((t) => t.length > 0);

  const success = await vaultState.addEntry(
    newTitle,
    newUsername,
    newPassword,
    formattedUrl,
    newNotes,
    entryType,
    newTotpSecret,
    tags,
  );

  if (success) {
    newTitle = '';
    newUsername = '';
    newPassword = '';
    newUrl = '';
    newTotpSecret = '';
    newNotes = '';
    newTagsString = '';
    showNewPassword = false;
    activePanel = 'list';
  } else {
    globalError = vaultState.error;
  }
};

const handleQuickGenerate = () => {
  const config = {
    type: 'Character',
    length: 16,
    min_uppercase: 1,
    min_lowercase: 1,
    min_numbers: 1,
    min_symbols: 1,
    exclude_ambiguous: false,
  };
  const result = vaultState.generateCredential(config);
  if (result) {
    newPassword = result.credential;
    showNewPassword = true;
  }
};
</script>

<div class="flex flex-col h-full justify-between overflow-hidden">
    <!-- Header -->
    <div class="flex items-center gap-3 border-b border-zinc-800 pb-2.5">
        <Button
            variant="outline"
            size="sm"
            class="bg-zinc-900 border-zinc-800 text-zinc-300 hover:text-white h-7 text-xs flex items-center gap-1"
            onclick={() => (activePanel = "list")}
        >
            <ArrowLeft class="h-3.5 w-3.5" /> Back
        </Button>
        <h2 class="text-sm font-semibold text-white">Add Entry</h2>
    </div>

    <!-- Form Fields -->
    <div class="grow overflow-y-auto my-3 pr-1 space-y-3.5 max-h-75">
        <!-- Entry Type Toggle -->
        <div
            class="grid grid-cols-2 gap-1 bg-zinc-900/50 p-1 border border-zinc-800 rounded-lg"
        >
            <button
                type="button"
                class="text-xs py-1 h-7.5 font-semibold rounded-md transition-all outline-none {entryType ===
                'Login'
                    ? 'bg-zinc-800 text-white'
                    : 'text-zinc-400 hover:text-zinc-200'}"
                onclick={() => (entryType = "Login")}
            >
                Login
            </button>
            <button
                type="button"
                class="text-xs py-1 h-7.5 font-semibold rounded-md transition-all outline-none {entryType ===
                'SecureNote'
                    ? 'bg-zinc-800 text-white'
                    : 'text-zinc-400 hover:text-zinc-200'}"
                onclick={() => (entryType = "SecureNote")}
            >
                Secure Note
            </button>
        </div>

        <!-- Common Title Field -->
        <div class="space-y-1">
            <Label
                for="new-title"
                class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                >Title *</Label
            >
            <Input
                id="new-title"
                type="text"
                placeholder="e.g. Google Account or Note Title"
                bind:value={newTitle}
                class="bg-zinc-900 border-zinc-800 text-white text-xs h-9 focus-visible:ring-1 focus-visible:ring-accent {titleError
                    ? 'border-red-500/80 focus-visible:ring-red-500'
                    : ''}"
            />
            {#if titleError}
                <p class="text-[10px] text-red-400 font-semibold">
                    {titleError}
                </p>
            {/if}
        </div>

        {#if entryType === "Login"}
            <!-- Login Specific Fields -->
            <div class="space-y-1">
                <Label
                    for="new-username"
                    class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                    >Username / Email</Label
                >
                <Input
                    id="new-username"
                    type="text"
                    placeholder="e.g. user@gmail.com"
                    bind:value={newUsername}
                    class="bg-zinc-900 border-zinc-800 text-white text-xs h-9 focus-visible:ring-1 focus-visible:ring-accent"
                />
            </div>

            <div class="space-y-1">
                <Label
                    for="new-password"
                    class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                    >Password</Label
                >
                <div class="flex gap-2">
                    <div class="relative grow">
                        <Input
                            id="new-password"
                            type={showNewPassword ? "text" : "password"}
                            placeholder="Enter password"
                            bind:value={newPassword}
                            class="bg-zinc-900 border-zinc-800 text-white pr-9 text-xs h-9 focus-visible:ring-1 focus-visible:ring-accent"
                        />
                        <Button
                            variant="ghost"
                            size="icon"
                            class="h-7 w-7 absolute right-1 top-1/2 -translate-y-1/2 text-zinc-400 hover:text-white"
                            onclick={() => (showNewPassword = !showNewPassword)}
                        >
                            {#if showNewPassword}
                                <EyeOff class="h-3.5 w-3.5" />
                            {:else}
                                <Eye class="h-3.5 w-3.5" />
                            {/if}
                        </Button>
                    </div>
                    <Button
                        variant="outline"
                        size="icon"
                        class="bg-zinc-900 border-zinc-800 text-zinc-400 hover:text-white h-9 w-9 shrink-0"
                        onclick={handleQuickGenerate}
                        title="Quick Generate Strong Password"
                    >
                        <Sparkles class="h-4 w-4 text-accent" />
                    </Button>
                </div>
            </div>

            <div class="space-y-1">
                <Label
                    for="new-url"
                    class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                    >Website URL</Label
                >
                <Input
                    id="new-url"
                    type="text"
                    placeholder="e.g. google.com"
                    bind:value={newUrl}
                    class="bg-zinc-900 border-zinc-800 text-white text-xs h-9 focus-visible:ring-1 focus-visible:ring-accent {urlError
                        ? 'border-red-500/80 focus-visible:ring-red-500'
                        : ''}"
                />
                {#if urlError}
                    <p class="text-[10px] text-red-400 font-semibold">
                        {urlError}
                    </p>
                {/if}
            </div>

            <div class="space-y-1">
                <Label
                    for="new-totp"
                    class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                    >TOTP Secret (2FA)</Label
                >
                <Input
                    id="new-totp"
                    type="text"
                    placeholder="e.g. JBSWY3DPEHPK3PXP"
                    bind:value={newTotpSecret}
                    class="bg-zinc-900 border-zinc-800 text-white text-xs h-9 focus-visible:ring-1 focus-visible:ring-accent {totpError
                        ? 'border-red-500/80 focus-visible:ring-red-500'
                        : ''}"
                />
                {#if totpError}
                    <p class="text-[10px] text-red-400 font-semibold">
                        {totpError}
                    </p>
                {/if}
            </div>

            <div class="space-y-1">
                <Label
                    for="new-notes"
                    class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                    >Notes</Label
                >
                <textarea
                    id="new-notes"
                    placeholder="Add custom notes..."
                    bind:value={newNotes}
                    class="w-full bg-zinc-900 border border-zinc-800 rounded-lg text-white p-2.5 text-xs outline-none focus:border-zinc-700 h-16 resize-none box-sizing:border-box"
                ></textarea>
            </div>
        {:else}
            <!-- Secure Note Specific Fields -->
            <div class="space-y-1">
                <Label
                    for="new-note-body"
                    class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                    >Secure Note Body</Label
                >
                <textarea
                    id="new-note-body"
                    placeholder="Type your secure note here..."
                    bind:value={newNotes}
                    class="w-full bg-zinc-900 border border-zinc-800 rounded-lg text-white p-3 text-xs outline-none focus:border-zinc-700 h-40 resize-none box-sizing:border-box"
                ></textarea>
            </div>
        {/if}

        <!-- Common Tags Field -->
        <div class="space-y-1">
            <Label
                for="new-tags"
                class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                >Tags (comma-separated)</Label
            >
            <Input
                id="new-tags"
                type="text"
                placeholder="e.g. work, personal, critical"
                bind:value={newTagsString}
                class="bg-zinc-900 border-zinc-800 text-white text-xs h-9 focus-visible:ring-1 focus-visible:ring-accent"
            />
        </div>

        {#if globalError}
            <div
                class="p-3 bg-red-950/30 border border-red-800/50 text-red-400 rounded-md text-xs"
            >
                {globalError}
            </div>
        {/if}
    </div>

    <!-- Actions -->
    <div class="grid grid-cols-2 gap-2 border-t border-zinc-900 pt-3">
        <Button
            variant="outline"
            class="w-full bg-zinc-900 border-zinc-800 text-zinc-300 hover:text-white text-xs h-9"
            onclick={() => (activePanel = "list")}
        >
            Cancel
        </Button>
        <Button
            class="w-full bg-zinc-50 text-zinc-950 hover:bg-zinc-200 text-xs h-9 font-semibold"
            onclick={handleAddEntry}
        >
            Save
        </Button>
    </div>
</div>
