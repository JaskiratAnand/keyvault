<script lang="ts">
import {
  ArrowLeft,
  Check,
  Copy,
  ExternalLink,
  Eye,
  EyeOff,
  Sparkles,
  Trash2,
} from 'lucide-svelte';
import { untrack } from 'svelte';
import { Button } from '~/components/ui/button/index.js';
import { Input } from '~/components/ui/input/index.js';
import { Label } from '~/components/ui/label/index.js';
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

let showPassword = $state(false);
let showHistory = $state(false);
let copiedStates = $state<Record<string, boolean>>({});
let visibleHistoryKeys = $state<Record<string, boolean>>({});

let cardShowHistory = $state<Record<string, boolean>>({});
let cardHistoryPasswordVisible = $state<Record<string, boolean>>({});
let cardPasswordVisible = $state<Record<string, boolean>>({});
let cardTotpVisible = $state<Record<string, boolean>>({});

// TOTP Reactive Timer & State
let currentTimeSeconds = $state(Math.floor(Date.now() / 1000));
let remainingSeconds = $derived(30 - (currentTimeSeconds % 30));
let totpCodes = $state<Record<string, string>>({});

$effect(() => {
  const interval = setInterval(() => {
    currentTimeSeconds = Math.floor(Date.now() / 1000);
  }, 1000);
  return () => clearInterval(interval);
});

$effect(() => {
  const item = selectedGroup;
  const ts = currentTimeSeconds;

  const newCodes: Record<string, string> = {};
  if (item?.accounts) {
    for (const acc of item.accounts) {
      if (acc.totp_secret) {
        newCodes[acc.id] = vaultState.generateTotp(acc.totp_secret, ts);
      }
    }
  }

  untrack(() => {
    totpCodes = newCodes;
  });
});

const toggleCardPassword = async (key: string) => {
  if (!cardPasswordVisible[key]) {
    const authorized = await reauthController.requestReauth();
    if (!authorized) return;
  }
  cardPasswordVisible[key] = !cardPasswordVisible[key];
};

const toggleCardTotp = async (key: string) => {
  if (!cardTotpVisible[key]) {
    const authorized = await reauthController.requestReauth();
    if (!authorized) return;
  }
  cardTotpVisible[key] = !cardTotpVisible[key];
};

const toggleCardHistoryPassword = async (key: string) => {
  if (!cardHistoryPasswordVisible[key]) {
    const authorized = await reauthController.requestReauth();
    if (!authorized) return;
  }
  cardHistoryPasswordVisible[key] = !cardHistoryPasswordVisible[key];
};

const triggerCopySecure = async (
  text: string | null | undefined,
  key: string,
) => {
  if (!text) return;
  const authorized = await reauthController.requestReauth();
  if (authorized) {
    await triggerCopy(text, key);
  }
};

const handleDeleteAccountDirect = async (acc: Account) => {
  const confirmed = confirm(
    'Are you sure you want to delete this account? This cannot be undone.',
  );
  if (!confirmed) return;
  if (!selectedGroup) return;

  const success = await vaultState.deleteAccount(selectedGroup.id, acc.id);
  if (success) {
    const updated = vaultState.vault.items.find(
      (e: VaultItem) => e.type === 'DomainGroup' && e.id === selectedGroup?.id,
    ) as DomainGroup | undefined;
    selectedGroup = updated || null;
    selectedEntry = null;
  }
};

// Edit State variables
let isEditing = $state(false);
let editEntryType = $state<'Login' | 'SecureNote'>('Login');
let editTitle = $state('');
let editUsername = $state('');
let editPassword = $state('');
let editUrl = $state('');
let editTotpSecret = $state('');
let editNotes = $state('');
let editTagsString = $state('');

let editShowPassword = $state(false);
let editGlobalError = $state('');

// Edit Field-specific errors
let editTitleError = $state('');
let editUrlError = $state('');
let editTotpError = $state('');

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

const handleAutofill = async () => {
  if (!selectedEntry) return;
  const authorized = await reauthController.requestReauth();
  if (!authorized) return;
  try {
    const [tab] = await browser.tabs.query({
      active: true,
      currentWindow: true,
    });
    if (!tab?.id) return;

    const username = selectedEntry.username || '';
    const password = selectedEntry.password || '';

    await browser.scripting.executeScript({
      target: { tabId: tab.id },
      func: (user: string, pass: string) => {
        const passFields = Array.from(
          document.querySelectorAll('input[type="password"]'),
        ) as HTMLInputElement[];
        if (passFields.length === 0) return;

        for (const passInput of passFields) {
          passInput.value = pass;
          passInput.dispatchEvent(new Event('input', { bubbles: true }));

          const form = passInput.form;
          let userInput = form?.querySelector(
            'input[type="email"], input[type="text"], input:not([type])',
          ) as HTMLInputElement | null;

          if (!userInput) {
            let sibling = passInput.previousElementSibling;
            while (sibling) {
              if (sibling.tagName === 'INPUT') {
                userInput = sibling as HTMLInputElement;
                break;
              }
              const nest = sibling.querySelector('input');
              if (nest) {
                userInput = nest;
                break;
              }
              sibling = sibling.previousElementSibling;
            }
          }

          if (userInput && user) {
            userInput.value = user;
            userInput.dispatchEvent(new Event('input', { bubbles: true }));
          }
        }
      },
      args: [username, password],
    });
  } catch (e) {
    console.error('Autofill injection failed:', e);
  }
};

const handleDelete = async () => {
  if (!selectedEntry) return;
  let success = false;
  if (selectedGroup) {
    success = await vaultState.deleteAccount(
      selectedGroup.id,
      selectedEntry.id,
    );
  } else {
    success = await vaultState.deleteEntry(selectedEntry.id);
  }
  if (success) {
    selectedEntry = null;
    selectedGroup = null;
    activePanel = 'list';
  }
};

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
let editUrlTimeout: ReturnType<typeof setTimeout>;
let editTotpTimeout: ReturnType<typeof setTimeout>;

// Svelte 5 Reactive validation effects for editing
$effect(() => {
  const titleVal = editTitle;
  if (titleVal.trim()) {
    editTitleError = '';
  }
});

$effect(() => {
  const urlVal = editUrl;
  if (!urlVal.trim()) {
    editUrlError = '';
    return;
  }
  clearTimeout(editUrlTimeout);
  editUrlTimeout = setTimeout(() => {
    const res = processUrl(urlVal);
    if (!res.isValid) {
      editUrlError = 'Please enter a valid website URL.';
    } else {
      editUrlError = '';
    }
  }, 400);
});

$effect(() => {
  const totpVal = editTotpSecret;
  if (!totpVal.trim()) {
    editTotpError = '';
    return;
  }
  clearTimeout(editTotpTimeout);
  editTotpTimeout = setTimeout(() => {
    if (!isValidBase32(totpVal)) {
      editTotpError = 'Invalid TOTP Secret. Base32 keys only use A-Z and 2-7.';
    } else {
      editTotpError = '';
    }
  }, 400);
});

// Trigger editing mode and initialize values
const startEditing = () => {
  if (!selectedEntry) return;
  if (selectedGroup) {
    editEntryType = 'Login';
    editTitle = selectedGroup.title;
    editUsername = selectedEntry.username || '';
    editPassword = selectedEntry.password || '';
    editUrl = selectedGroup.urls?.[0] || '';
    editTotpSecret = selectedEntry.totp_secret || '';
    editNotes = selectedEntry.notes || '';
    editTagsString = selectedGroup.tags ? selectedGroup.tags.join(', ') : '';
  } else {
    editEntryType = 'SecureNote';
    editTitle = selectedEntry.title;
    editUsername = '';
    editPassword = '';
    editUrl = '';
    editTotpSecret = '';
    editNotes = selectedEntry.notes || '';
    editTagsString = selectedEntry.tags ? selectedEntry.tags.join(', ') : '';
  }
  editShowPassword = false;
  editGlobalError = '';
  editTitleError = '';
  editUrlError = '';
  editTotpError = '';
  skipResetIsEditing = true;
  isEditing = true;
};

const handleSaveEdit = async () => {
  if (!selectedEntry) return;

  editGlobalError = '';
  editTitleError = '';
  editUrlError = '';
  editTotpError = '';

  // Synchronous immediate validation checks
  if (!editTitle.trim()) {
    editTitleError = 'Title is required.';
    return;
  }

  let formattedUrl = '';
  if (editEntryType === 'Login' && editUrl.trim()) {
    const urlResult = processUrl(editUrl);
    if (!urlResult.isValid) {
      editUrlError = 'Please enter a valid website URL.';
      return;
    }
    formattedUrl = urlResult.formatted;
  }

  if (editEntryType === 'Login' && editTotpSecret.trim()) {
    if (!isValidBase32(editTotpSecret)) {
      editTotpError = 'Invalid TOTP Secret. Base32 keys only use A-Z and 2-7.';
      return;
    }
  }

  // Parse comma-separated tags
  const tags = editTagsString
    .split(',')
    .map((t) => t.trim())
    .filter((t) => t.length > 0);

  const success = await vaultState.updateEntry(
    selectedEntry.id,
    editTitle,
    editUsername,
    editPassword,
    formattedUrl,
    editNotes,
    editEntryType,
    editTotpSecret,
    tags,
  );

  if (success) {
    if (editEntryType === 'SecureNote') {
      const updated = vaultState.vault.items.find(
        (e: VaultItem) => e.type === 'SecureNote' && e.id === selectedEntry?.id,
      );
      if (updated) {
        selectedEntry = updated;
      }
    } else {
      const updatedGroup = vaultState.vault.items.find(
        (e: VaultItem) =>
          e.type === 'DomainGroup' && e.id === selectedGroup?.id,
      ) as DomainGroup | undefined;
      if (updatedGroup) {
        selectedGroup = updatedGroup;
        const updatedAcc = updatedGroup.accounts.find(
          (a: Account) => a.id === selectedEntry?.id,
        );
        if (updatedAcc) {
          selectedEntry = updatedAcc;
        }
      }
    }
    isEditing = false;
  } else {
    editGlobalError = vaultState.error;
  }
};

const handleQuickGenerateEdit = () => {
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
    editPassword = result.credential;
    editShowPassword = true;
  }
};

let displayTitle = $derived(
  selectedGroup ? selectedGroup.title : selectedEntry?.title || '',
);
let displayUrl = $derived(selectedGroup ? selectedGroup.urls[0] || '' : '');
let displayTags = $derived(
  selectedGroup ? selectedGroup.tags : selectedEntry?.tags || [],
);
let isSecureNote = $derived(!selectedGroup);

let skipResetIsEditing = false;

$effect(() => {
  const _ = selectedEntry;
  if (skipResetIsEditing) {
    skipResetIsEditing = false;
    return;
  }
  isEditing = false;
  showHistory = false;
  visibleHistoryKeys = {};
});
</script>

{#if selectedEntry || selectedGroup}
    <div class="flex flex-col h-full justify-between overflow-hidden">
        <!-- Header -->
        <div
            class="flex items-center justify-between border-b border-zinc-800 pb-2.5"
        >
            <div class="flex items-center gap-3">
                <Button
                    variant="outline"
                    size="sm"
                    class="bg-zinc-900 border-zinc-800 text-zinc-300 hover:text-white h-7 text-xs flex items-center gap-1"
                    onclick={() => (activePanel = "list")}
                >
                    <ArrowLeft class="h-3.5 w-3.5" /> Back
                </Button>
                <h2 class="text-sm font-semibold text-white">
                    {isEditing ? "Edit Entry" : "Credential Details"}
                </h2>
            </div>

            {#if !isEditing}
                <div class="flex items-center gap-1.5">
                    {#if isSecureNote}
                        {#if selectedEntry && selectedEntry.password_history && selectedEntry.password_history.length > 0}
                            <Button
                                variant="outline"
                                size="sm"
                                class="h-7 text-xs border-zinc-800 bg-zinc-900 text-zinc-300 hover:text-white"
                                onclick={() => (showHistory = !showHistory)}
                            >
                                {showHistory ? "Hide History" : "History"}
                            </Button>
                        {/if}
                        <Button
                            variant="outline"
                            size="sm"
                            class="h-7 text-xs border-zinc-800 bg-zinc-900 text-zinc-300 hover:text-white"
                            onclick={startEditing}
                        >
                            Edit
                        </Button>
                    {:else}
                        <Button
                            variant="outline"
                            size="sm"
                            class="h-7 text-xs border-zinc-800 bg-zinc-900 text-zinc-300 hover:text-white"
                            onclick={() => {
                                selectedEntry = null;
                                editEntryType = 'Login';
                                editTitle = selectedGroup ? selectedGroup.title : '';
                                editUsername = '';
                                editPassword = '';
                                editUrl = selectedGroup && selectedGroup.urls ? selectedGroup.urls[0] || '' : '';
                                editTotpSecret = '';
                                editNotes = '';
                                editTagsString = selectedGroup && selectedGroup.tags ? selectedGroup.tags.join(', ') : '';
                                editShowPassword = false;
                                editGlobalError = '';
                                editTitleError = '';
                                editUrlError = '';
                                editTotpError = '';
                                skipResetIsEditing = true;
                                isEditing = true;
                            }}
                        >
                            Add
                        </Button>
                    {/if}
                </div>
            {/if}
        </div>

        <!-- Scrollable content -->
        <div class="grow overflow-y-auto my-3 pr-1 space-y-4 max-h-75">
            {#if !isEditing}
                <!-- VIEW MODE -->
                {#if isSecureNote}
                    <!-- SECURE NOTE VIEW MODE -->
                    <!-- Header Tile Card -->
                    <div class="p-3 bg-zinc-900/50 border border-zinc-800 rounded-lg flex items-center gap-3 min-w-0">
                        <div class="h-10 w-10 rounded-full bg-zinc-800 border border-zinc-700 text-zinc-300 flex items-center justify-center font-bold uppercase text-sm shrink-0">
                            📝
                        </div>
                        <div class="flex flex-col min-w-0">
                            <div class="flex items-center gap-1.5">
                                <span class="text-sm font-semibold text-white truncate">{displayTitle}</span>
                                <span class="bg-zinc-800/80 border border-zinc-700 text-zinc-400 text-[8px] font-bold uppercase tracking-wider px-1.5 py-0.5 rounded select-none">
                                    Note
                                </span>
                            </div>
                        </div>
                    </div>

                    <!-- Secure Note Specific Display -->
                    <div class="space-y-1">
                        <Label class="text-xs font-semibold text-zinc-400 uppercase tracking-wider">Secure Note Body</Label>
                        <div class="w-full bg-zinc-900 border border-zinc-800 rounded-lg text-zinc-200 p-2.5 text-xs h-40 overflow-y-auto whitespace-pre-wrap select-all">
                            {selectedEntry.notes || "(Empty Note)"}
                        </div>
                    </div>

                    <!-- Tags display -->
                    {#if displayTags && displayTags.length > 0}
                        <div class="space-y-1 pt-1">
                            <Label class="text-xs font-semibold text-zinc-400 uppercase tracking-wider">Tags</Label>
                            <div class="flex flex-wrap gap-1.5 pt-0.5">
                                {#each displayTags as tag}
                                    <span class="bg-zinc-900 text-zinc-300 border border-zinc-800 px-2.5 py-0.5 rounded-full text-[9px] font-semibold">
                                        {tag}
                                    </span>
                                {/each}
                            </div>
                        </div>
                    {/if}
                {:else}
                    <!-- DOMAIN GROUP VIEW MODE (Google Password Manager style) -->
                    <!-- Header Tile Card -->
                    <div class="p-3 bg-zinc-900/50 border border-zinc-800 rounded-lg flex items-center gap-3 min-w-0">
                        <div class="h-10 w-10 rounded-full bg-zinc-800 border border-zinc-700 text-zinc-300 flex items-center justify-center font-bold uppercase text-sm shrink-0">
                            🌐
                        </div>
                        <div class="flex flex-col min-w-0">
                            <div class="flex items-center gap-1.5">
                                <span class="text-sm font-semibold text-white truncate">{displayTitle}</span>
                            </div>
                            {#if displayUrl}
                                <a
                                    class="text-xs text-accent hover:underline flex items-center gap-1 mt-0.5 min-w-0 max-w-full"
                                    href={displayUrl}
                                    target="_blank"
                                    rel="noreferrer"
                                >
                                    <span class="truncate">{displayUrl.replace(/^https?:\/\/(www\.)?/, "")}</span>
                                    <ExternalLink class="h-3 w-3 inline shrink-0" />
                                </a>
                            {/if}
                        </div>
                    </div>

                    <!-- Group Tags -->
                    {#if displayTags && displayTags.length > 0}
                        <div class="flex flex-wrap gap-1.5 pt-0.5">
                            {#each displayTags as tag}
                                <span class="bg-zinc-900 text-zinc-300 border border-zinc-800 px-2 py-0.5 rounded-full text-[8px] font-semibold">
                                    #{tag}
                                </span>
                            {/each}
                        </div>
                    {/if}

                    <!-- Account Cards -->
                    {#if selectedGroup && selectedGroup.accounts && selectedGroup.accounts.length > 0}
                        <div class="space-y-3.5">
                            {#each selectedGroup.accounts as acc (acc.id)}
                                <div class="p-3 bg-zinc-900 border border-zinc-800 rounded-lg space-y-3 shadow-sm">
                                    <!-- Username -->
                                    <div class="space-y-1">
                                        <Label class="text-[9px] font-bold text-zinc-500 uppercase tracking-wider block">Username</Label>
                                        <div class="relative flex items-center">
                                            <Input
                                                type="text"
                                                readonly
                                                value={acc.username || ""}
                                                placeholder="No Username"
                                                class="bg-zinc-950 border-zinc-900 text-white pr-9 text-xs h-8"
                                            />
                                            {#if acc.username}
                                                <div class="absolute right-1 top-1/2 -translate-y-1/2 flex items-center">
                                                    <Button
                                                        variant="ghost"
                                                        size="icon"
                                                        class="h-6 w-6 text-zinc-400 hover:text-white"
                                                        onclick={() => triggerCopy(acc.username, 'acc_u_' + acc.id)}
                                                        title="Copy Username"
                                                    >
                                                        {#if copiedStates['acc_u_' + acc.id]}
                                                            <Check class="h-3.5 w-3.5 text-green-400" />
                                                        {:else}
                                                            <Copy class="h-3.5 w-3.5" />
                                                        {/if}
                                                    </Button>
                                                </div>
                                            {/if}
                                        </div>
                                    </div>

                                    <!-- Password -->
                                    <div class="space-y-1">
                                        <Label class="text-[9px] font-bold text-zinc-500 uppercase tracking-wider block">Password</Label>
                                        <div class="relative flex items-center">
                                            <Input
                                                type={cardPasswordVisible['acc_p_' + acc.id] ? "text" : "password"}
                                                readonly
                                                value={acc.password || ""}
                                                placeholder="No Password"
                                                class="bg-zinc-950 border-zinc-900 text-white pr-16 text-xs h-8"
                                            />
                                            <div class="absolute right-1 top-1/2 -translate-y-1/2 flex items-center gap-0.5">
                                                <Button
                                                    variant="ghost"
                                                    size="icon"
                                                    class="h-6 w-6 text-zinc-400 hover:text-white"
                                                    onclick={() => toggleCardPassword('acc_p_' + acc.id)}
                                                    title={cardPasswordVisible['acc_p_' + acc.id] ? "Hide password" : "Show password"}
                                                >
                                                    {#if cardPasswordVisible['acc_p_' + acc.id]}
                                                        <EyeOff class="h-3.5 w-3.5" />
                                                    {:else}
                                                        <Eye class="h-3.5 w-3.5" />
                                                    {/if}
                                                </Button>
                                                {#if acc.password}
                                                    <Button
                                                        variant="ghost"
                                                        size="icon"
                                                        class="h-6 w-6 text-zinc-400 hover:text-white"
                                                        onclick={() => triggerCopySecure(acc.password, 'acc_p_' + acc.id)}
                                                        title="Copy Password"
                                                    >
                                                        {#if copiedStates['acc_p_' + acc.id]}
                                                            <Check class="h-3.5 w-3.5 text-green-400" />
                                                        {:else}
                                                            <Copy class="h-3.5 w-3.5" />
                                                        {/if}
                                                    </Button>
                                                {/if}
                                            </div>
                                        </div>
                                    </div>

                                    <!-- Sites link -->
                                    {#if selectedGroup.urls && selectedGroup.urls.length > 0}
                                        <div class="space-y-1">
                                            <Label class="text-[9px] font-bold text-zinc-500 uppercase tracking-wider block">Sites</Label>
                                            <a
                                                class="text-xs text-accent hover:underline flex items-center gap-1 mt-0.5 min-w-0 max-w-full"
                                                href={selectedGroup.urls[0]}
                                                target="_blank"
                                                rel="noreferrer"
                                            >
                                                <span class="truncate">{selectedGroup.urls[0]}</span>
                                                <ExternalLink class="h-3 w-3 inline shrink-0" />
                                            </a>
                                        </div>
                                    {/if}

                                    <!-- TOTP Secret -->
                                    {#if acc.totp_secret}
                                        <!-- 6-digit TOTP Code Display -->
                                        <div class="space-y-1">
                                            <Label class="text-[9px] font-bold text-zinc-500 uppercase tracking-wider block">One-Time Password (TOTP)</Label>
                                            <div class="relative flex items-center">
                                                <div class="flex-1 bg-zinc-950 border border-zinc-900 rounded-lg px-2.5 py-1 flex items-center justify-between min-w-0 h-8 pr-8">
                                                    {#if totpCodes[acc.id]}
                                                        <span class="font-mono text-sm font-bold tracking-[0.2em] text-[#06b6d4] select-all leading-none">
                                                            {totpCodes[acc.id].slice(0, 3)} {totpCodes[acc.id].slice(3)}
                                                        </span>
                                                    {:else}
                                                        <span class="text-[11px] text-zinc-500 italic animate-pulse">Generating...</span>
                                                    {/if}

                                                    <!-- Visual countdown bar/indicator -->
                                                    <div class="flex items-center gap-1.5 shrink-0">
                                                        <div class="w-10 h-1 bg-zinc-800 rounded-full overflow-hidden">
                                                            <div
                                                                class="h-full bg-[#06b6d4] transition-all duration-1000 ease-linear"
                                                                style="width: {(remainingSeconds / 30) * 100}%"
                                                            ></div>
                                                        </div>
                                                        <span class="text-[9px] font-mono text-zinc-400 w-4 text-right leading-none">{remainingSeconds}s</span>
                                                    </div>
                                                </div>
                                                <Button
                                                    variant="ghost"
                                                    size="icon"
                                                    class="h-6 w-6 text-zinc-400 hover:text-white shrink-0 absolute right-1"
                                                    onclick={() => triggerCopy(totpCodes[acc.id], 'acc_totp_code_' + acc.id)}
                                                    title="Copy TOTP Code"
                                                    disabled={!totpCodes[acc.id]}
                                                >
                                                    {#if copiedStates['acc_totp_code_' + acc.id]}
                                                        <Check class="h-3.5 w-3.5 text-green-400" />
                                                    {:else}
                                                        <Copy class="h-3.5 w-3.5" />
                                                    {/if}
                                                </Button>
                                            </div>
                                        </div>

                                        <div class="space-y-1">
                                            <Label class="text-[9px] font-bold text-zinc-500 uppercase tracking-wider block">TOTP Secret</Label>
                                            <div class="relative flex items-center">
                                                <Input
                                                    type={cardTotpVisible['acc_t_' + acc.id] ? "text" : "password"}
                                                    readonly
                                                    value={acc.totp_secret}
                                                    class="bg-zinc-950 border-zinc-900 text-white pr-16 text-xs h-8"
                                                />
                                                <div class="absolute right-1 top-1/2 -translate-y-1/2 flex items-center gap-0.5">
                                                    <Button
                                                        variant="ghost"
                                                        size="icon"
                                                        class="h-6 w-6 text-zinc-400 hover:text-white"
                                                        onclick={() => toggleCardTotp('acc_t_' + acc.id)}
                                                        title={cardTotpVisible['acc_t_' + acc.id] ? "Hide TOTP Secret" : "Show TOTP Secret"}
                                                    >
                                                        {#if cardTotpVisible['acc_t_' + acc.id]}
                                                            <EyeOff class="h-3.5 w-3.5" />
                                                        {:else}
                                                            <Eye class="h-3.5 w-3.5" />
                                                        {/if}
                                                    </Button>
                                                    <Button
                                                        variant="ghost"
                                                        size="icon"
                                                        class="h-6 w-6 text-zinc-400 hover:text-white"
                                                        onclick={() => triggerCopySecure(acc.totp_secret, 'acc_t_' + acc.id)}
                                                        title="Copy TOTP Secret"
                                                    >
                                                        {#if copiedStates['acc_t_' + acc.id]}
                                                            <Check class="h-3.5 w-3.5 text-green-400" />
                                                        {:else}
                                                            <Copy class="h-3.5 w-3.5" />
                                                        {/if}
                                                    </Button>
                                                </div>
                                            </div>
                                        </div>
                                    {/if}

                                    <!-- Note -->
                                    <div class="space-y-1">
                                        <Label class="text-[9px] font-bold text-zinc-500 uppercase tracking-wider block">Note</Label>
                                        <div class="w-full bg-zinc-950 border border-zinc-900 rounded-lg text-zinc-300 p-2.5 text-xs min-h-9 overflow-y-auto whitespace-pre-wrap select-all font-sans leading-normal">
                                            {acc.notes || "No note added"}
                                        </div>
                                    </div>

                                    <!-- Actions Row -->
                                    <div class="flex justify-between items-center pt-1.5 border-t border-zinc-800/80">
                                        <div class="flex items-center gap-1.5">
                                            <Button
                                                variant="outline"
                                                size="sm"
                                                class="h-6 text-[10px] border-zinc-800 bg-zinc-900 text-zinc-300 hover:text-white px-2 py-0"
                                                onclick={() => {
                                                    selectedEntry = acc;
                                                    startEditing();
                                                }}
                                            >
                                                Edit
                                            </Button>
                                            <Button
                                                variant="outline"
                                                size="sm"
                                                class="h-6 text-[10px] border-red-950 text-red-400 hover:bg-red-950/20 px-2 py-0"
                                                onclick={() => handleDeleteAccountDirect(acc)}
                                            >
                                                Delete
                                            </Button>
                                        </div>

                                        {#if acc.password_history && acc.password_history.length > 0}
                                            <Button
                                                variant="ghost"
                                                size="sm"
                                                class="h-6 text-[10px] text-accent hover:text-white p-0 hover:bg-transparent"
                                                onclick={() => cardShowHistory[acc.id] = !cardShowHistory[acc.id]}
                                            >
                                                {cardShowHistory[acc.id] ? "Hide History" : `History (${acc.password_history.length})`}
                                            </Button>
                                        {/if}
                                    </div>

                                    <!-- Password History List -->
                                    {#if cardShowHistory[acc.id] && acc.password_history && acc.password_history.length > 0}
                                        <div class="mt-2 space-y-1.5 bg-zinc-950/40 p-2 border border-zinc-900 rounded-lg">
                                            <Label class="text-[9px] font-bold text-zinc-500 uppercase tracking-wider block">Password History</Label>
                                            <div class="space-y-2 max-h-24 overflow-y-auto pr-1">
                                                {#each acc.password_history.slice().reverse() as hist, index}
                                                    {@const histKey = `${acc.id}_h_${index}`}
                                                    <div class="flex items-center justify-between gap-2 border-b border-zinc-900/50 pb-1.5 last:border-0 last:pb-0">
                                                        <div class="flex flex-col min-w-0">
                                                            <span class="text-[11px] font-mono text-zinc-300 truncate select-all">
                                                                {cardHistoryPasswordVisible[histKey]
                                                                    ? hist.password
                                                                    : "•".repeat(Math.max(1, hist.password.length))}
                                                            </span>
                                                            <span class="text-[8px] text-zinc-500">
                                                                {new Date(hist.changed_at).toLocaleString()}
                                                            </span>
                                                        </div>
                                                        <div class="flex items-center gap-0.5 shrink-0">
                                                            <Button
                                                                variant="ghost"
                                                                size="icon"
                                                                class="h-6 w-6 text-zinc-400 hover:text-white"
                                                                onclick={() => toggleCardHistoryPassword(histKey)}
                                                                title={cardHistoryPasswordVisible[histKey] ? "Hide password" : "Show password"}
                                                            >
                                                                {#if cardHistoryPasswordVisible[histKey]}
                                                                    <EyeOff class="h-3 w-3" />
                                                                {:else}
                                                                    <Eye class="h-3 w-3" />
                                                                {/if}
                                                            </Button>
                                                            <Button
                                                                variant="ghost"
                                                                size="icon"
                                                                class="h-6 w-6 text-zinc-400 hover:text-white"
                                                                onclick={() => triggerCopySecure(hist.password, histKey)}
                                                                title="Copy password"
                                                            >
                                                                    {#if copiedStates[histKey]}
                                                                        <Check class="h-3 w-3 text-green-400" />
                                                                    {:else}
                                                                        <Copy class="h-3 w-3" />
                                                                    {/if}
                                                            </Button>
                                                        </div>
                                                    </div>
                                                {/each}
                                            </div>
                                        </div>
                                    {/if}
                                </div>
                            {/each}
                        </div>
                    {:else}
                        <div class="flex flex-col items-center justify-center py-6 text-zinc-400 gap-2 border border-zinc-800 rounded-lg bg-zinc-900/30">
                            <span class="text-xl">👤</span>
                            <p class="text-xs font-semibold text-zinc-500">No accounts in this group</p>
                            <Button
                                variant="outline"
                                size="sm"
                                class="bg-zinc-900 border-zinc-800 text-zinc-300 hover:text-white"
                                onclick={() => {
                                    selectedEntry = null;
                                    editEntryType = 'Login';
                                    editTitle = selectedGroup ? selectedGroup.title : '';
                                    editUsername = '';
                                    editPassword = '';
                                    editUrl = selectedGroup && selectedGroup.urls ? selectedGroup.urls[0] || '' : '';
                                    editTotpSecret = '';
                                    editNotes = '';
                                    editTagsString = selectedGroup && selectedGroup.tags ? selectedGroup.tags.join(', ') : '';
                                    editShowPassword = false;
                                    editGlobalError = '';
                                    editTitleError = '';
                                    editUrlError = '';
                                    editTotpError = '';
                                    skipResetIsEditing = true;
                                    isEditing = true;
                                }}
                            >
                                Add Account
                            </Button>
                        </div>
                    {/if}
                {/if}
            {:else}
                <!-- Common Title Field -->
                <div class="space-y-1">
                    <Label
                        for="edit-title"
                        class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                        >Title *</Label
                    >
                    <Input
                        id="edit-title"
                        type="text"
                        placeholder="Title"
                        bind:value={editTitle}
                        class="bg-zinc-900 border-zinc-800 text-white text-xs h-9 focus-visible:ring-1 focus-visible:ring-accent {editTitleError
                            ? 'border-red-500/80 focus-visible:ring-red-500'
                            : ''}"
                    />
                    {#if editTitleError}
                        <p class="text-[10px] text-red-400 font-semibold">
                            {editTitleError}
                        </p>
                    {/if}
                </div>

                {#if editEntryType === "Login"}
                    <!-- Login Fields -->
                    <div class="space-y-1">
                        <Label
                            for="edit-username"
                            class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                            >Username</Label
                        >
                        <Input
                            id="edit-username"
                            type="text"
                            placeholder="Username"
                            bind:value={editUsername}
                            class="bg-zinc-900 border-zinc-800 text-white text-xs h-9 focus-visible:ring-1 focus-visible:ring-accent"
                        />
                    </div>

                    <div class="space-y-1">
                        <Label
                            for="edit-password"
                            class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                            >Password</Label
                        >
                        <div class="flex gap-2">
                            <div class="relative grow">
                                <Input
                                    id="edit-password"
                                    type={editShowPassword
                                        ? "text"
                                        : "password"}
                                    placeholder="Password"
                                    bind:value={editPassword}
                                    class="bg-zinc-900 border-zinc-800 text-white pr-9 text-xs h-9 focus-visible:ring-1 focus-visible:ring-accent"
                                />
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    class="h-7 w-7 absolute right-1 top-1/2 -translate-y-1/2 text-zinc-400 hover:text-white"
                                    onclick={() =>
                                        (editShowPassword = !editShowPassword)}
                                >
                                    {#if editShowPassword}
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
                                onclick={handleQuickGenerateEdit}
                                title="Regenerate password"
                            >
                                <Sparkles class="h-4 w-4 text-accent" />
                            </Button>
                        </div>
                    </div>

                    <div class="space-y-1">
                        <Label
                            for="edit-url"
                            class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                            >Website URL</Label
                        >
                        <Input
                            id="edit-url"
                            type="text"
                            placeholder="Website URL"
                            bind:value={editUrl}
                            class="bg-zinc-900 border-zinc-800 text-white text-xs h-9 focus-visible:ring-1 focus-visible:ring-accent {editUrlError
                                ? 'border-red-500/80 focus-visible:ring-red-500'
                                : ''}"
                        />
                        {#if editUrlError}
                            <p class="text-[10px] text-red-400 font-semibold">
                                {editUrlError}
                            </p>
                        {/if}
                    </div>

                    <div class="space-y-1">
                        <Label
                            for="edit-totp"
                            class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                            >TOTP Secret</Label
                        >
                        <Input
                            id="edit-totp"
                            type="text"
                            placeholder="Base32 Key"
                            bind:value={editTotpSecret}
                            class="bg-zinc-900 border-zinc-800 text-white text-xs h-9 focus-visible:ring-1 focus-visible:ring-accent {editTotpError
                                ? 'border-red-500/80 focus-visible:ring-red-500'
                                : ''}"
                        />
                        {#if editTotpError}
                            <p class="text-[10px] text-red-400 font-semibold">
                                {editTotpError}
                            </p>
                        {/if}
                    </div>

                    <div class="space-y-1">
                        <Label
                            for="edit-notes"
                            class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                            >Notes</Label
                        >
                        <textarea
                            id="edit-notes"
                            placeholder="Notes..."
                            bind:value={editNotes}
                            class="w-full bg-zinc-900 border border-zinc-800 rounded-lg text-white p-2.5 text-xs outline-none focus:border-zinc-700 h-16 resize-none box-sizing:border-box"
                        ></textarea>
                    </div>
                {:else}
                    <!-- Secure Note body input -->
                    <div class="space-y-1">
                        <Label
                            for="edit-note-body"
                            class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                            >Secure Note Body</Label
                        >
                        <textarea
                            id="edit-note-body"
                            placeholder="Type note content here..."
                            bind:value={editNotes}
                            class="w-full bg-zinc-900 border border-zinc-800 rounded-lg text-white p-3 text-xs outline-none focus:border-zinc-700 h-40 resize-none box-sizing:border-box"
                        ></textarea>
                    </div>
                {/if}

                <!-- Tags Input -->
                <div class="space-y-1">
                    <Label
                        for="edit-tags"
                        class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                        >Tags (comma-separated)</Label
                    >
                    <Input
                        id="edit-tags"
                        type="text"
                        placeholder="Tags"
                        bind:value={editTagsString}
                        class="bg-zinc-900 border-zinc-800 text-white text-xs h-9 focus-visible:ring-1 focus-visible:ring-accent"
                    />
                </div>

                {#if editGlobalError}
                    <div
                        class="p-3 bg-red-950/30 border border-red-800/50 text-red-400 rounded-md text-xs"
                    >
                        {editGlobalError}
                    </div>
                {/if}
            {/if}
        </div>

        <!-- Actions -->
        <div class="border-t border-zinc-900 pt-3">
            {#if !isEditing}
                <div class="space-y-2">
                    <Button
                        class="w-full bg-zinc-50 text-zinc-950 hover:bg-zinc-200 text-xs h-9 font-semibold flex items-center justify-center gap-1.5 transition-colors"
                        onclick={handleAutofill}
                    >
                        <Sparkles class="h-4 w-4" /> Autofill on Page
                    </Button>
                    <Button
                        class="w-full text-xs h-9 font-semibold bg-red-950/40 border border-red-800/60 text-red-200 hover:bg-red-900/50 hover:text-white flex items-center justify-center gap-1.5 transition-colors"
                        onclick={handleDelete}
                    >
                        <Trash2 class="h-4 w-4" /> Delete Credential
                    </Button>
                </div>
            {:else}
                <div class="grid grid-cols-2 gap-2">
                    <Button
                        variant="outline"
                        class="w-full bg-zinc-900 border-zinc-800 text-zinc-300 hover:text-white text-xs h-9"
                        onclick={() => (isEditing = false)}
                    >
                        Cancel
                    </Button>
                    <Button
                        class="w-full bg-zinc-50 text-zinc-950 hover:bg-zinc-200 text-xs h-9 font-semibold"
                        onclick={handleSaveEdit}
                    >
                        Save Changes
                    </Button>
                </div>
            {/if}
        </div>
    </div>
{/if}
