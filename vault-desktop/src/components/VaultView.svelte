<script lang="ts">
import {
  AlertTriangle,
  Check,
  Copy,
  ExternalLink,
  Eye,
  EyeOff,
  FileText,
  Globe,
  Info,
  Lock,
  Plus,
  Search,
  ShieldAlert,
  Sparkles,
  X,
} from 'lucide-svelte';
import type { Account, VaultItem } from '../lib/types.js';
import { getVaultContext } from '../lib/vault-state.svelte.js';

interface Props {
  requestReauth: () => Promise<boolean>;
  showConfirm: (
    title: string,
    message: string,
    onConfirm: () => void | Promise<void>,
  ) => void;
}

let { requestReauth, showConfirm }: Props = $props();

const vaultState = getVaultContext();

// Local UI & Form States
let isEditing = $state(false);
let isCreating = $state(false);
let createType = $state<'DomainGroup' | 'SecureNote'>('DomainGroup');
let viewAccountActiveTab = $state(0);

// Form Field States
let formTitle = $state('');
let formTags = $state('');
let formNoteNotes = $state('');
let formUsername = $state('');
let formPassword = $state('');
let formUrl = $state('');
let formTotpSecret = $state('');
let formNotes = $state('');
let selectedAccount = $state<Account | null>(null);

let isSaving = $state(false);
let formErrors = $state<Record<string, string>>({});
let touchedFields = $state<Record<string, boolean>>({});

// Visibility maps
let passwordVisible = $state<Record<string, boolean>>({});
let totpVisible = $state<Record<string, boolean>>({});
let showHistory = $state<Record<string, boolean>>({});
let historyPasswordVisible = $state<Record<string, boolean>>({});

// Clipboard & Copy feedback states
let copiedStates = $state<Record<string, boolean>>({});
let clipboardTimeRemaining = $state(0);
let clipboardTimerInterval: any = null;

// TOTP Reactive codes
let currentTimeSeconds = $state(Math.floor(Date.now() / 1000));
let remainingSeconds = $derived(30 - (currentTimeSeconds % 30));
let totpCodes = $state<Record<string, string>>({});
let totpErrors = $state<Record<string, string>>({});

const { invoke } = (window as any).__TAURI__?.core || {};

$effect(() => {
  const interval = setInterval(() => {
    currentTimeSeconds = Math.floor(Date.now() / 1000);
  }, 1000);
  return () => clearInterval(interval);
});

// Keep TOTPs updated reactively
$effect(() => {
  const item = vaultState.selectedItem;
  const ts = currentTimeSeconds;

  if (item && item.type === 'DomainGroup' && item.accounts && invoke) {
    for (const acc of item.accounts) {
      if (acc.totp_secret) {
        const secret = acc.totp_secret;
        invoke('generate_totp', { secret, timestamp: ts })
          .then((code: string) => {
            totpCodes[acc.id] = code;
            totpCodes = { ...totpCodes };
            totpErrors[acc.id] = '';
            totpErrors = { ...totpErrors };
          })
          .catch((err: any) => {
            console.error('Failed to generate TOTP for', acc.id, err);
            totpCodes[acc.id] = '';
            totpCodes = { ...totpCodes };
            totpErrors[acc.id] = String(err);
            totpErrors = { ...totpErrors };
          });
      }
    }
  }
});

// Debounced validation helper
let debounceTimer: any = null;
function triggerValidation() {
  clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    validateForm(true);
  }, 300);
}

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

const isValidBase32 = (str: string): boolean => {
  const clean = str.replace(/[\s-]/g, '');
  if (!clean) return true;
  return /^[A-Z2-7]+=*$/i.test(clean);
};

function validateForm(onlyTouched = false): boolean {
  const errors: Record<string, string> = {};

  if (!onlyTouched || touchedFields.title) {
    if (!formTitle.trim()) {
      errors.title = 'Title is required.';
    }
  }

  const isLogin =
    createType === 'DomainGroup' ||
    (vaultState.selectedItem &&
      vaultState.selectedItem.type === 'DomainGroup' &&
      selectedAccount !== null);

  if (isLogin) {
    if (!onlyTouched || touchedFields.url) {
      if (formUrl.trim()) {
        const res = processUrl(formUrl);
        if (!res.isValid) {
          errors.url = 'Please enter a valid website URL.';
        }
      }
    }

    if (!onlyTouched || touchedFields.totp) {
      if (formTotpSecret.trim()) {
        if (!isValidBase32(formTotpSecret)) {
          errors.totp =
            'Invalid TOTP Secret. Base32 keys only use A-Z and 2-7.';
        }
      }
    }
  }

  formErrors = errors;
  return Object.keys(errors).length === 0;
}

function initCreateForm(
  type: 'DomainGroup' | 'SecureNote',
  prefill?: { title: string; url: string; tags: string },
) {
  createType = type;
  formTitle = prefill?.title || '';
  formTags = prefill?.tags || '';
  formNoteNotes = '';
  formUsername = '';
  formPassword = '';
  formUrl = prefill?.url || '';
  formTotpSecret = '';
  formNotes = '';
  selectedAccount = null;
  formErrors = {};
  touchedFields = {};
  isCreating = true;
  isEditing = false;
}

function initEditForm(acc?: Account) {
  if (!vaultState.selectedItem) return;
  const item = vaultState.selectedItem;
  formTitle = item.title;
  formTags = item.tags.join(', ');

  if (item.type === 'DomainGroup') {
    if (acc) {
      selectedAccount = acc;
      formUsername = acc.username || '';
      formPassword = acc.password || '';
      formUrl = item.urls[0] || '';
      formTotpSecret = acc.totp_secret || '';
      formNotes = acc.notes || '';
    } else {
      const firstAcc = item.accounts[0];
      if (firstAcc) {
        selectedAccount = firstAcc;
        formUsername = firstAcc.username || '';
        formPassword = firstAcc.password || '';
        formUrl = item.urls[0] || '';
        formTotpSecret = firstAcc.totp_secret || '';
        formNotes = firstAcc.notes || '';
      } else {
        selectedAccount = null;
        formUsername = '';
        formPassword = '';
        formUrl = item.urls[0] || '';
        formTotpSecret = '';
        formNotes = '';
      }
    }
  } else {
    selectedAccount = null;
    formNoteNotes = item.notes;
  }
  formErrors = {};
  touchedFields = {};
  isEditing = true;
  isCreating = false;
}

async function handleDeleteAccount() {
  if (!vaultState.selectedItem || !selectedAccount) return;
  const groupId = vaultState.selectedItem.id;
  const accId = selectedAccount.id;

  showConfirm(
    'Delete Account?',
    'Are you sure you want to delete this account? This cannot be undone.',
    async () => {
      const success = await vaultState.deleteAccount(groupId, accId);
      if (success) {
        isEditing = false;
        isCreating = false;
        selectedAccount = null;
      }
    },
  );
}

async function handleSaveItem() {
  touchedFields = {
    title: true,
    url: true,
    totp: true,
  };

  const isValid = validateForm(false);
  if (!isValid) return;

  try {
    isSaving = true;
    await new Promise((resolve) => setTimeout(resolve, 50));

    const tagsArray = formTags
      .split(',')
      .map((t) => t.trim())
      .filter((t) => t.length > 0);

    let success = false;

    if (isCreating) {
      success = await vaultState.addEntry(
        formTitle,
        formUsername,
        formPassword,
        formUrl,
        formNotes,
        createType === 'DomainGroup' ? 'Login' : 'SecureNote',
        formTotpSecret,
        tagsArray,
      );
    } else {
      if (vaultState.selectedItem?.type === 'SecureNote') {
        success = await vaultState.updateEntry(
          vaultState.selectedItem.id,
          formTitle,
          undefined,
          undefined,
          undefined,
          formNoteNotes,
          'SecureNote',
          undefined,
          tagsArray,
        );
      } else if (selectedAccount) {
        success = await vaultState.updateEntry(
          selectedAccount.id,
          formTitle,
          formUsername,
          formPassword,
          formUrl,
          formNotes,
          'Login',
          formTotpSecret,
          tagsArray,
        );
      }
    }

    if (success) {
      isEditing = false;
      isCreating = false;
      selectedAccount = null;
    }
  } catch (err) {
    console.error('Failed to save item:', err);
    formErrors.general = String(err);
  } finally {
    isSaving = false;
  }
}

async function handleQuickGeneratePassword() {
  const config = {
    type: 'Character',
    length: 16,
    min_uppercase: 1,
    min_lowercase: 1,
    min_numbers: 1,
    min_symbols: 1,
    exclude_ambiguous: false,
  };
  const result = await vaultState.generateCredential(config);
  if (result) {
    formPassword = result.credential;
    touchedFields.password = true;
    triggerValidation();
  }
}

async function copyText(text: string, key: string) {
  if (!text) return;
  if (
    key.startsWith('acc_p_') ||
    key.startsWith('acc_t_') ||
    key.startsWith('cf_') ||
    key === 'note_n'
  ) {
    const authorized = await requestReauth();
    if (!authorized) return;
  }

  navigator.clipboard.writeText(text);
  copiedStates[key] = true;
  setTimeout(() => {
    copiedStates[key] = false;
  }, 2000);

  // Auto-clear clipboard in 30 seconds
  if (clipboardTimerInterval) {
    clearInterval(clipboardTimerInterval);
  }
  clipboardTimeRemaining = 30;
  clipboardTimerInterval = setInterval(() => {
    clipboardTimeRemaining--;
    if (clipboardTimeRemaining <= 0) {
      navigator.clipboard.writeText('');
      clearInterval(clipboardTimerInterval);
      clipboardTimerInterval = null;
    }
  }, 1000);
}

async function togglePassword(key: string) {
  if (!passwordVisible[key]) {
    if (key === 'form_p' && isCreating) {
      // Bypass verification during new entry creation
    } else {
      const authorized = await requestReauth();
      if (!authorized) return;
    }
  }
  passwordVisible[key] = !passwordVisible[key];
}

async function toggleTotp(key: string) {
  if (!totpVisible[key]) {
    const authorized = await requestReauth();
    if (!authorized) return;
  }
  totpVisible[key] = !totpVisible[key];
}
</script>

<div class="grow flex overflow-hidden">
  <!-- Items List Sidebar (Split Left) -->
  <div class="w-80 border-r border-[#27272a] flex flex-col bg-[#09090b] shrink-0">
    <!-- Search Bar -->
    <div class="px-3 py-3 border-b border-[#27272a]/70">
      <div class="relative">
        <input
          type="text"
          placeholder="Search vault..."
          bind:value={vaultState.searchQuery}
          class="w-full bg-transparent border-none pl-7 pr-3 py-1 text-xs text-[#fafafa] placeholder-[#52525b] outline-none focus:ring-0"
        />
        <Search class="w-3.5 h-3.5 text-[#52525b] absolute left-1 top-1/2 -translate-y-1/2" />
      </div>
    </div>

    <!-- Categories -->
    <div class="px-3 py-2">
      <div class="flex bg-[#18181b] border border-[#27272a] rounded-lg p-0.5 text-xs">
        <button
          class="flex-1 py-1 rounded text-center transition-colors cursor-pointer border-0 {vaultState.activeTab === 'all' ? 'bg-[#27272a] text-white font-medium' : 'text-[#a1a1aa] hover:text-white'}"
          onclick={() => { vaultState.activeTab = 'all'; vaultState.selectedTag = null; }}
        >
          All
        </button>
        <button
          class="flex-1 py-1 rounded text-center transition-colors cursor-pointer border-0 {vaultState.activeTab === 'logins' ? 'bg-[#27272a] text-white font-medium' : 'text-[#a1a1aa] hover:text-white'}"
          onclick={() => { vaultState.activeTab = 'logins'; vaultState.selectedTag = null; }}
        >
          Logins
        </button>
        <button
          class="flex-1 py-1 rounded text-center transition-colors cursor-pointer border-0 {vaultState.activeTab === 'notes' ? 'bg-[#27272a] text-white font-medium' : 'text-[#a1a1aa] hover:text-white'}"
          onclick={() => { vaultState.activeTab = 'notes'; vaultState.selectedTag = null; }}
        >
          Notes
        </button>
      </div>
    </div>

    <!-- Tags Filter list -->
    {#if vaultState.allTags.length > 0}
      <div class="px-3 pb-2 border-b border-[#27272a] flex gap-1 overflow-x-auto scrollbar-none py-1">
        <button
          class="shrink-0 text-[10px] px-2 py-0.5 rounded-full border transition-colors cursor-pointer {vaultState.selectedTag === null ? 'bg-[#06b6d4]/10 border-[#06b6d4]/30 text-[#06b6d4]' : 'bg-transparent border-[#27272a] text-[#a1a1aa] hover:text-white'}"
          onclick={() => vaultState.selectedTag = null}
        >
          All Tags
        </button>
        {#each vaultState.allTags as tag}
          <button
            class="shrink-0 text-[10px] px-2 py-0.5 rounded-full border transition-colors cursor-pointer {vaultState.selectedTag === tag ? 'bg-[#06b6d4]/10 border-[#06b6d4]/30 text-[#06b6d4]' : 'bg-transparent border-[#27272a] text-[#a1a1aa] hover:text-white'}"
            onclick={() => vaultState.selectedTag = tag}
          >
            #{tag}
          </button>
        {/each}
      </div>
    {/if}

    <!-- Clipboard Cleared Indicator -->
    {#if clipboardTimeRemaining > 0}
      <div class="bg-[#06b6d4]/10 border-b border-[#06b6d4]/20 text-[#06b6d4] text-[10px] px-3 py-1.5 flex items-center justify-between">
        <span class="flex items-center gap-1.5">
          <ShieldAlert class="w-3.5 h-3.5 animate-pulse" />
          Clipboard clearing active
        </span>
        <span class="font-mono bg-[#09090b] border border-[#06b6d4]/30 px-1 rounded">{clipboardTimeRemaining}s</span>
      </div>
    {/if}

    <!-- Add Entry Row -->
    <div class="px-4 py-2.5 border-b border-[#27272a]/45 flex justify-between items-center text-xs shrink-0 bg-[#09090b]">
      <span class="text-[#a1a1aa] font-semibold uppercase tracking-wider text-[10px]">
        {vaultState.filteredItems.length} {vaultState.filteredItems.length === 1 ? 'Item' : 'Items'}
      </span>
      <button
        onclick={() => initCreateForm('DomainGroup')}
        class="bg-[#18181b] border border-[#27272a] hover:border-[#3f3f46] text-[#fafafa] text-[10px] font-semibold px-2.5 py-1 rounded-md transition-colors flex items-center gap-1 cursor-pointer"
      >
        <Plus class="w-3.5 h-3.5 text-[#06b6d4]" /> Add Entry
      </button>
    </div>

    <!-- Vault Scrollable List -->
    <div class="grow overflow-y-auto p-2 space-y-1">
      {#if vaultState.filteredItems.length === 0}
        <div class="text-center py-8 text-[#a1a1aa] text-xs">
          <Info class="w-4 h-4 mx-auto mb-2 opacity-40" />
          No items match this filter.
        </div>
      {:else}
        {#each vaultState.filteredItems as item}
          <div
            role="button"
            tabindex="0"
            class="w-full text-left p-2.5 rounded-lg border transition-all duration-150 flex items-start gap-3 cursor-pointer min-w-0 overflow-hidden no-scale-windows {vaultState.selectedItem?.id === item.id ? 'bg-[#18181b] border-[#06b6d4]/40 shadow-sm' : 'bg-transparent border-transparent hover:bg-[#18181b]/50'}"
            onclick={() => {
              vaultState.selectedItem = item;
              viewAccountActiveTab = 0;
              isEditing = false;
              isCreating = false;
            }}
            onkeydown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                vaultState.selectedItem = item;
                viewAccountActiveTab = 0;
                isEditing = false;
                isCreating = false;
              }
            }}
          >
            <div class="p-2 rounded bg-[#18181b] border border-[#27272a] text-[#a1a1aa] shrink-0">
              {#if item.type === 'DomainGroup'}
                <Globe class="w-3.5 h-3.5" />
              {:else}
                <FileText class="w-3.5 h-3.5" />
              {/if}
            </div>
            <div class="grow min-w-0">
              <div class="flex items-center justify-between gap-1.5 min-w-0">
                <span class="text-xs font-semibold truncate text-white">{item.title}</span>
                {#if item.type === 'DomainGroup'}
                  <span class="bg-cyan-500/10 text-cyan-400 border border-cyan-500/20 text-[8px] font-bold uppercase tracking-wider px-1.5 py-0.5 rounded select-none shrink-0">
                    {item.accounts?.length || 0} {item.accounts?.length === 1 ? 'acc' : 'accs'}
                  </span>
                {:else}
                  <span class="bg-amber-500/10 text-amber-400 border border-amber-500/20 text-[8px] font-bold uppercase tracking-wider px-1.5 py-0.5 rounded select-none shrink-0">
                    Note
                  </span>
                {/if}
              </div>

              {#if item.type === 'DomainGroup'}
                {#if item.tags && item.tags.length > 0}
                  <div class="text-[10px] text-[#a1a1aa] truncate mt-0.5">
                    #{item.tags.join(', #')}
                  </div>
                {:else if item.urls && item.urls[0]}
                  <div class="text-[10px] text-[#a1a1aa] truncate mt-0.5">
                    {item.urls[0].replace(/^https?:\/\/(www\.)?/, "")}
                  </div>
                {:else}
                  <div class="text-[10px] text-zinc-500 truncate mt-0.5">
                    No domain URL
                  </div>
                {/if}
              {:else}
                {#if item.tags && item.tags.length > 0}
                  <div class="text-[10px] text-[#a1a1aa] truncate mt-0.5">
                    #{item.tags.join(', #')}
                  </div>
                {:else}
                  <div class="text-[10px] text-[#a1a1aa] truncate mt-0.5">
                    {item.notes || 'Empty note'}
                  </div>
                {/if}
              {/if}
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </div>

  <!-- Details/Editor Pane (Split Right) -->
  <div class="grow overflow-y-auto p-6 flex flex-col">
    {#if vaultState.error}
      <div class="max-w-2xl w-full mx-auto mb-4 bg-red-500/10 border border-red-500/20 text-red-400 p-3 rounded-lg text-xs flex items-center justify-between gap-2">
        <div class="flex items-center gap-2">
          <AlertTriangle class="h-4 w-4 shrink-0 text-red-400" />
          <span>{vaultState.error}</span>
        </div>
        <button
          onclick={() => { vaultState.error = ''; }}
          class="text-red-400 hover:text-white p-0.5 rounded cursor-pointer border-0 bg-transparent transition-colors"
          title="Dismiss Error"
        >
          <X class="h-3.5 w-3.5" />
        </button>
      </div>
    {/if}

    {#if isEditing || isCreating}
      <!-- Editor Form (DomainGroup & SecureNote) -->
      <div class="max-w-2xl w-full mx-auto space-y-6">
        <div class="flex justify-between items-center pb-4 border-b border-[#27272a]">
          <div>
            <h1 class="text-base font-bold text-white">
              {isCreating ? `Create New ${createType === 'DomainGroup' ? 'Login' : 'Secure Note'}` : `Edit ${vaultState.selectedItem?.title}`}
            </h1>
            <p class="text-xs text-[#a1a1aa]">{isCreating ? 'Add a new secure credential or note.' : 'Modify secure credential or note details.'}</p>
          </div>
          <button
            onclick={() => { isEditing = false; isCreating = false; selectedAccount = null; }}
            class="p-1.5 hover:bg-[#18181b] border border-transparent hover:border-[#27272a] rounded transition-colors cursor-pointer bg-transparent text-white"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        {#if isCreating}
          <!-- Entry Type Selector -->
          <div class="grid grid-cols-2 gap-1 bg-[#18181b] border border-[#27272a] p-1 rounded-lg">
            <button
              type="button"
              class="text-xs py-1.5 font-semibold rounded-md transition-all outline-none cursor-pointer border-0 {createType === 'DomainGroup' ? 'bg-[#27272a] text-white font-bold border border-[#06b6d4]/20 shadow-sm' : 'text-[#a1a1aa] hover:text-white border border-transparent'}"
              onclick={() => {
                createType = 'DomainGroup';
                touchedFields = {};
                formErrors = {};
                formTitle = '';
                formUsername = '';
                formPassword = '';
                formUrl = '';
                formTotpSecret = '';
                formNotes = '';
                formTags = '';
              }}
            >
              Login
            </button>
            <button
              type="button"
              class="text-xs py-1.5 font-semibold rounded-md transition-all outline-none cursor-pointer border-0 {createType === 'SecureNote' ? 'bg-[#27272a] text-white font-bold border border-[#06b6d4]/20 shadow-sm' : 'text-[#a1a1aa] hover:text-white border border-transparent'}"
              onclick={() => {
                createType = 'SecureNote';
                touchedFields = {};
                formErrors = {};
                formTitle = '';
                formTags = '';
                formNoteNotes = '';
              }}
            >
              Secure Note
            </button>
          </div>
        {/if}

        <div class="space-y-5">
          <!-- Common Title Field -->
          <div class="space-y-1">
            <span class="text-[9px] font-bold text-[#a1a1aa] uppercase tracking-wider block">Title *</span>
            <input
              type="text"
              placeholder="e.g. Google Account or Note Title"
              bind:value={formTitle}
              oninput={() => { touchedFields.title = true; triggerValidation(); }}
              class="w-full bg-[#18181b] border rounded-lg px-3 py-2 text-xs text-[#fafafa] placeholder-[#52525b] outline-none focus:ring-1 transition-all {formErrors.title ? 'border-[#ef4444]/60 focus:border-[#ef4444] focus:ring-[#ef4444]/30' : 'border-[#27272a] focus:border-[#d4d4d8] focus:ring-[#06b6d4]/40'}"
              autocapitalize="none"
              autocorrect="off"
              spellcheck="false"
            />
            {#if formErrors.title}
              <span class="text-[10px] text-[#ef4444] mt-1 flex items-center gap-1">
                <ShieldAlert class="w-3.5 h-3.5" />
                {formErrors.title}
              </span>
            {/if}
          </div>

          {#if createType === 'DomainGroup' || (vaultState.selectedItem && vaultState.selectedItem.type === 'DomainGroup' && selectedAccount !== null)}
            <!-- Username / Email -->
            <div class="space-y-1">
              <span class="text-[9px] font-bold text-[#a1a1aa] uppercase block">Username / Email</span>
              <input
                type="text"
                placeholder="e.g. user@gmail.com"
                bind:value={formUsername}
                class="w-full bg-[#18181b] border border-[#27272a] rounded-lg px-3 py-2 text-xs text-[#fafafa] placeholder-[#52525b] outline-none focus:border-[#d4d4d8] focus:ring-1 focus:ring-[#06b6d4]/40 transition-all"
                autocapitalize="none"
                autocorrect="off"
                spellcheck="false"
              />
            </div>

            <!-- Password -->
            <div class="space-y-1">
              <span class="text-[9px] font-bold text-[#a1a1aa] uppercase block">Password</span>
              <div class="flex gap-2">
                <div class="relative grow">
                  <input
                    type={passwordVisible['form_p'] ? 'text' : 'password'}
                    placeholder="Enter password"
                    bind:value={formPassword}
                    class="w-full bg-[#18181b] border border-[#27272a] rounded-lg px-3 pr-10 py-2 text-xs text-[#fafafa] placeholder-[#52525b] outline-none focus:border-[#d4d4d8] focus:ring-1 focus:ring-[#06b6d4]/40 transition-all"
                    autocapitalize="none"
                    autocorrect="off"
                    spellcheck="false"
                  />
                  <button
                    type="button"
                    onclick={() => togglePassword('form_p')}
                    class="absolute right-2 top-1/2 -translate-y-1/2 text-[#a1a1aa] hover:text-white p-1 cursor-pointer border-0 bg-transparent"
                    title={passwordVisible['form_p'] ? 'Hide password' : 'Show password'}
                  >
                    {#if passwordVisible['form_p']}
                      <EyeOff class="w-3.5 h-3.5" />
                    {:else}
                      <Eye class="w-3.5 h-3.5" />
                    {/if}
                  </button>
                </div>
                <button
                  type="button"
                  onclick={handleQuickGeneratePassword}
                  class="bg-[#18181b] border border-[#27272a] hover:border-zinc-700 text-[#fafafa] hover:text-white text-xs font-semibold px-3 py-2 rounded-lg transition-colors cursor-pointer shrink-0 flex items-center justify-center gap-1.5"
                >
                  <Sparkles class="w-3.5 h-3.5 text-[#06b6d4]" />
                </button>
              </div>
            </div>

            <!-- Website URL -->
            <div class="space-y-1">
              <span class="text-[9px] font-bold text-[#a1a1aa] uppercase block">Website URL</span>
              <input
                type="text"
                placeholder="e.g. google.com"
                bind:value={formUrl}
                oninput={() => { touchedFields.url = true; triggerValidation(); }}
                class="w-full bg-[#18181b] border rounded-lg px-3 py-2 text-xs text-[#fafafa] placeholder-[#52525b] outline-none focus:ring-1 transition-all {formErrors.url ? 'border-[#ef4444]/60 focus:border-[#ef4444] focus:ring-[#ef4444]/30' : 'border-[#27272a] focus:border-[#d4d4d8] focus:ring-[#06b6d4]/40'}"
                autocapitalize="none"
                autocorrect="off"
                spellcheck="false"
              />
              {#if formErrors.url}
                <span class="text-[10px] text-[#ef4444] mt-1 flex items-center gap-1">
                  <ShieldAlert class="w-3.5 h-3.5" />
                  {formErrors.url}
                </span>
              {/if}
            </div>

            <!-- TOTP Secret -->
            <div class="space-y-1">
              <span class="text-[9px] font-bold text-[#a1a1aa] uppercase block">TOTP Secret (2FA)</span>
              <input
                type="text"
                placeholder="e.g. JBSWY3DPEHPK3PXP"
                bind:value={formTotpSecret}
                oninput={() => { touchedFields.totp = true; triggerValidation(); }}
                class="w-full bg-[#18181b] border rounded-lg px-3 py-2 text-xs text-[#fafafa] placeholder-[#52525b] outline-none focus:ring-1 transition-all {formErrors.totp ? 'border-[#ef4444]/60 focus:border-[#ef4444] focus:ring-[#ef4444]/30' : 'border-[#27272a] focus:border-[#d4d4d8] focus:ring-[#06b6d4]/40'}"
                autocapitalize="none"
                autocorrect="off"
                spellcheck="false"
              />
              {#if formErrors.totp}
                <span class="text-[10px] text-[#ef4444] mt-1 flex items-center gap-1">
                  <ShieldAlert class="w-3.5 h-3.5" />
                  {formErrors.totp}
                </span>
              {/if}
            </div>

            <!-- Notes -->
            <div class="space-y-1">
              <span class="text-[9px] font-bold text-[#a1a1aa] uppercase block">Notes</span>
              <textarea
                placeholder="Add custom notes..."
                bind:value={formNotes}
                rows="4"
                class="w-full bg-[#18181b] border border-[#27272a] rounded-lg px-3 py-2 text-xs text-[#fafafa] placeholder-[#52525b] outline-none focus:border-[#d4d4d8] resize-none"
              ></textarea>
            </div>
          {:else}
            <!-- Secure Note Body -->
            <div class="space-y-1">
              <span class="text-[9px] font-bold text-[#a1a1aa] uppercase tracking-wider block">Secure Note Body</span>
              <textarea
                placeholder="Type your secure note here..."
                bind:value={formNoteNotes}
                rows="12"
                class="w-full bg-[#18181b] border border-[#27272a] rounded-lg p-4 text-xs text-[#fafafa] placeholder-[#52525b] outline-none focus:border-[#d4d4d8] resize-none font-mono"
              ></textarea>
            </div>
          {/if}

          <!-- Tags -->
          <div class="space-y-1">
            <span class="text-[9px] font-bold text-[#a1a1aa] uppercase tracking-wider block">Tags (comma-separated)</span>
            <input
              type="text"
              placeholder="e.g. work, personal, financial"
              bind:value={formTags}
              class="w-full bg-[#18181b] border border-[#27272a] rounded-lg px-3 py-2 text-xs text-[#fafafa] placeholder-[#52525b] outline-none focus:border-[#d4d4d8] focus:ring-1 focus:ring-[#06b6d4]/40 transition-all"
              autocapitalize="none"
              autocorrect="off"
              spellcheck="false"
            />
          </div>
        </div>

        <!-- Save / Cancel button footer -->
        <div class="flex justify-between items-center pt-4 border-t border-[#27272a]">
          <div>
            {#if isEditing && selectedAccount}
              <button
                type="button"
                onclick={handleDeleteAccount}
                class="px-4 py-2 border border-red-500/20 text-[#ef4444] text-xs font-semibold rounded-lg hover:bg-red-500/10 transition-colors cursor-pointer bg-transparent"
              >
                Delete Account
              </button>
            {/if}
          </div>
          <div class="flex gap-2">
            <button
              type="button"
              onclick={() => { isEditing = false; isCreating = false; selectedAccount = null; }}
              class="px-4 py-2 bg-transparent hover:bg-zinc-800 text-[#fafafa] border border-[#27272a] text-xs font-semibold rounded-lg transition-colors cursor-pointer"
            >
              Cancel
            </button>
            <button
              type="button"
              onclick={handleSaveItem}
              disabled={isSaving}
              class="px-4 py-2 bg-white hover:bg-zinc-200 text-zinc-900 text-xs font-bold rounded-lg transition-colors flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
            >
              Save Entry
            </button>
          </div>
        </div>
      </div>
    {:else}
      <!-- Detail Display View -->
      {#if vaultState.selectedItem}
        <div class="max-w-2xl w-full mx-auto space-y-6">
          <!-- Header -->
          <div class="flex justify-between items-center pb-4 border-b border-[#27272a]">
            <div class="flex items-center gap-3">
              <div class="bg-[#18181b] border border-[#27272a] p-2.5 rounded-lg text-[#06b6d4]">
                {#if vaultState.selectedItem.type === 'DomainGroup'}
                  <Globe class="w-5 h-5" />
                {:else}
                  <FileText class="w-5 h-5" />
                {/if}
              </div>
              <div class="min-w-0 flex flex-col justify-center">
                <div class="flex items-center gap-2">
                  <h1 class="text-base font-bold text-white leading-tight truncate">{vaultState.selectedItem.title}</h1>
                  {#if vaultState.selectedItem.type === 'DomainGroup'}
                    <span class="bg-cyan-500/10 text-cyan-400 border border-cyan-500/20 text-[8px] font-bold uppercase tracking-wider px-1.5 py-0.5 rounded select-none shrink-0">
                      {vaultState.selectedItem.accounts?.length || 0} {vaultState.selectedItem.accounts?.length === 1 ? 'account' : 'accounts'}
                    </span>
                  {/if}
                </div>
                {#if vaultState.selectedItem && vaultState.selectedItem.type === 'DomainGroup'}
                  {@const group = vaultState.selectedItem}
                  {#if group.urls && group.urls[0]}
                    <a
                      href={group.urls[0]}
                      target="_blank"
                      rel="noreferrer"
                      class="text-xs text-[#06b6d4] hover:underline flex items-center gap-1 mt-0.5 min-w-0 max-w-full"
                    >
                      <span class="truncate">{group.urls[0].replace(/^https?:\/\/(www\.)?/, '')}</span>
                    </a>
                  {/if}
                {/if}
              </div>
            </div>

            {#if vaultState.selectedItem.type === 'DomainGroup'}
              <button
                onclick={() => {
                  const item = vaultState.selectedItem;
                  if (item && item.type === 'DomainGroup') {
                    initCreateForm('DomainGroup', { title: item.title, url: item.urls[0] || '', tags: item.tags.join(', ') });
                  }
                }}
                class="bg-[#27272a] hover:bg-[#3f3f46] text-[#fafafa] border border-[#27272a] text-xs font-semibold px-4 py-2 rounded-lg transition-all flex items-center gap-1.5 cursor-pointer"
              >
                <Plus class="w-3.5 h-3.5" /> Add Account
              </button>
            {:else}
              <button
                onclick={() => initEditForm()}
                class="bg-[#27272a] hover:bg-[#3f3f46] text-[#fafafa] border border-[#27272a] text-xs font-semibold px-4 py-2 rounded-lg transition-all flex items-center gap-1.5 cursor-pointer"
              >
                Edit Entry
              </button>
            {/if}
          </div>

          {#if vaultState.selectedItem.type === 'DomainGroup'}
            <!-- Tags list -->
            {#if vaultState.selectedItem.tags && vaultState.selectedItem.tags.length > 0}
              <div class="flex flex-wrap gap-1.5">
                {#each vaultState.selectedItem.tags as tag}
                  <span class="bg-zinc-900 text-zinc-300 border border-zinc-800 px-2.5 py-0.5 rounded-full text-[9px] font-semibold">
                    #{tag}
                  </span>
                {/each}
              </div>
            {/if}

            <!-- Accounts Card List -->
            {#if vaultState.selectedItem.accounts && vaultState.selectedItem.accounts.length > 0}
              <div class="space-y-4 pt-2">
                {#each vaultState.selectedItem.accounts as acc (acc.id)}
                  <div class="p-4 bg-[#18181b] border border-[#27272a] rounded-lg space-y-4 shadow-sm">
                    <!-- Username -->
                    <div class="space-y-1.5">
                      <span class="text-[9px] font-bold text-[#a1a1aa] uppercase tracking-wider block">Username</span>
                      <div class="flex items-center gap-2">
                        <input
                          type="text"
                          readonly
                          value={acc.username || ""}
                          placeholder="No Username"
                          class="w-full bg-[#09090b] border border-[#27272a] rounded-lg px-3 py-1.5 text-xs text-[#fafafa] outline-none cursor-default select-all"
                        />
                        {#if acc.username}
                          <button
                            onclick={() => copyText(acc.username || '', 'acc_u_' + acc.id)}
                            class="h-8 w-8 flex items-center justify-center bg-[#27272a] hover:bg-[#3f3f46] text-[#fafafa] rounded-lg transition-colors border border-[#27272a] shrink-0 cursor-pointer"
                            title="Copy Username"
                          >
                            {#if copiedStates['acc_u_' + acc.id]}
                              <Check class="w-3.5 h-3.5 text-green-400" />
                            {:else}
                              <Copy class="w-3.5 h-3.5" />
                            {/if}
                          </button>
                        {/if}
                      </div>
                    </div>

                    <!-- Password -->
                    <div class="space-y-1.5">
                      <span class="text-[9px] font-bold text-[#a1a1aa] uppercase tracking-wider block">Password</span>
                      <div class="flex items-center gap-2">
                        <input
                          type={passwordVisible['acc_p_' + acc.id] ? "text" : "password"}
                          readonly
                          value={acc.password || ""}
                          placeholder="No Password"
                          class="w-full bg-[#09090b] border border-[#27272a] rounded-lg px-3 py-1.5 text-xs text-[#fafafa] outline-none cursor-default"
                        />
                        <button
                          onclick={() => togglePassword('acc_p_' + acc.id)}
                          class="h-8 w-8 flex items-center justify-center bg-[#27272a] hover:bg-[#3f3f46] text-[#fafafa] rounded-lg transition-colors border border-[#27272a] shrink-0 cursor-pointer"
                          title={passwordVisible['acc_p_' + acc.id] ? "Hide password" : "Show password"}
                        >
                          {#if passwordVisible['acc_p_' + acc.id]}
                            <EyeOff class="w-3.5 h-3.5" />
                          {:else}
                            <Eye class="w-3.5 h-3.5" />
                          {/if}
                        </button>
                        {#if acc.password}
                          <button
                            onclick={() => copyText(acc.password || '', 'acc_p_' + acc.id)}
                            class="h-8 w-8 flex items-center justify-center bg-[#27272a] hover:bg-[#3f3f46] text-[#fafafa] rounded-lg transition-colors border border-[#27272a] shrink-0 cursor-pointer"
                            title="Copy Password"
                          >
                            {#if copiedStates['acc_p_' + acc.id]}
                              <Check class="w-3.5 h-3.5 text-green-400" />
                            {:else}
                              <Copy class="w-3.5 h-3.5" />
                            {/if}
                          </button>
                        {/if}
                      </div>
                    </div>

                    <!-- Sites / URL -->
                    {#if vaultState.selectedItem.urls && vaultState.selectedItem.urls.length > 0}
                      <div class="space-y-1.5">
                        <span class="text-[9px] font-bold text-[#a1a1aa] uppercase tracking-wider block">Sites</span>
                        <div class="flex items-center gap-2">
                          <a
                            href={vaultState.selectedItem.urls[0]}
                            target="_blank"
                            rel="noreferrer"
                            class="text-xs text-[#06b6d4] hover:underline flex items-center gap-1 mt-0.5 min-w-0 max-w-full"
                          >
                            <span class="truncate">{vaultState.selectedItem.urls[0]}</span>
                            <ExternalLink class="w-3 h-3 shrink-0" />
                          </a>
                        </div>
                      </div>
                    {/if}

                    <!-- TOTP -->
                    {#if acc.totp_secret}
                      <div class="space-y-1.5">
                        <span class="text-[9px] font-bold text-[#a1a1aa] uppercase tracking-wider block">One-Time Password (TOTP)</span>
                        <div class="flex items-center gap-2">
                          <div class="flex-1 bg-[#09090b] border border-[#27272a] rounded-lg px-3 py-1.5 flex items-center justify-between min-w-0 h-8">
                            {#if totpCodes[acc.id]}
                              <span class="font-mono text-base font-bold tracking-[0.2em] text-[#06b6d4] select-all leading-none">
                                {totpCodes[acc.id].slice(0, 3)} {totpCodes[acc.id].slice(3)}
                              </span>
                            {:else}
                              {#if totpErrors[acc.id]}
                                <span class="text-xs text-[#ef4444] italic truncate max-w-50" title={totpErrors[acc.id]}>
                                  Error: {totpErrors[acc.id]}
                                </span>
                              {:else}
                                <span class="text-xs text-[#71717a] italic animate-pulse">Generating...</span>
                              {/if}
                            {/if}

                            <div class="flex items-center gap-2 shrink-0">
                              <div class="w-12 h-1.5 bg-[#27272a] rounded-full overflow-hidden">
                                <div
                                  class="h-full bg-[#06b6d4] transition-all duration-1000 ease-linear"
                                  style="width: {(remainingSeconds / 30) * 100}%"
                                ></div>
                              </div>
                              <span class="text-[10px] font-mono text-[#a1a1aa] w-4 text-right leading-none">{remainingSeconds}s</span>
                            </div>
                          </div>

                          <button
                            onclick={() => copyText(totpCodes[acc.id] || '', 'acc_totp_code_' + acc.id)}
                            class="h-8 w-8 flex items-center justify-center bg-[#27272a] hover:bg-[#3f3f46] text-[#fafafa] rounded-lg transition-colors border border-[#27272a] shrink-0 cursor-pointer"
                            title="Copy TOTP Code"
                            disabled={!totpCodes[acc.id]}
                          >
                            {#if copiedStates['acc_totp_code_' + acc.id]}
                              <Check class="w-3.5 h-3.5 text-green-400" />
                            {:else}
                              <Copy class="w-3.5 h-3.5" />
                            {/if}
                          </button>
                        </div>
                      </div>

                      <div class="space-y-1.5">
                        <span class="text-[9px] font-bold text-[#a1a1aa] uppercase tracking-wider block">TOTP Secret</span>
                        <div class="flex items-center gap-2">
                          <input
                            type={totpVisible['acc_t_' + acc.id] ? "text" : "password"}
                            readonly
                            value={acc.totp_secret}
                            class="w-full bg-[#09090b] border border-[#27272a] rounded-lg px-3 py-1.5 text-xs text-[#fafafa] outline-none cursor-default"
                          />
                          <button
                            onclick={() => toggleTotp('acc_t_' + acc.id)}
                            class="h-8 w-8 flex items-center justify-center bg-[#27272a] hover:bg-[#3f3f46] text-[#fafafa] rounded-lg transition-colors border border-[#27272a] shrink-0 cursor-pointer"
                            title={totpVisible['acc_t_' + acc.id] ? "Hide TOTP Secret" : "Show TOTP Secret"}
                          >
                            {#if totpVisible['acc_t_' + acc.id]}
                              <EyeOff class="w-3.5 h-3.5" />
                            {:else}
                              <Eye class="w-3.5 h-3.5" />
                            {/if}
                          </button>
                          <button
                            onclick={() => copyText(acc.totp_secret || '', 'acc_t_' + acc.id)}
                            class="h-8 w-8 flex items-center justify-center bg-[#27272a] hover:bg-[#3f3f46] text-[#fafafa] rounded-lg transition-colors border border-[#27272a] shrink-0 cursor-pointer"
                            title="Copy TOTP Secret"
                          >
                            {#if copiedStates['acc_t_' + acc.id]}
                              <Check class="w-3.5 h-3.5 text-green-400" />
                            {:else}
                              <Copy class="w-3.5 h-3.5" />
                            {/if}
                          </button>
                        </div>
                      </div>
                    {/if}

                    <!-- Note -->
                    <div class="space-y-1.5">
                      <span class="text-[9px] font-bold text-[#a1a1aa] uppercase tracking-wider block">Note</span>
                      <div class="bg-[#09090b] border border-[#27272a] rounded-lg p-3 text-xs text-[#fafafa] min-h-12 whitespace-pre-wrap leading-relaxed select-text font-sans">
                        {acc.notes || "No note added"}
                      </div>
                    </div>

                    <!-- Actions -->
                    <div class="flex justify-between items-center pt-2 border-t border-[#27272a]/60">
                      <div class="flex gap-2">
                        <button
                          onclick={() => initEditForm(acc)}
                          class="px-3.5 py-1.5 bg-[#27272a] hover:bg-[#3f3f46] text-[#fafafa] text-xs font-semibold rounded-lg transition-colors border border-[#27272a] cursor-pointer"
                        >
                          Edit
                        </button>
                        <button
                          onclick={() => {
                            selectedAccount = acc;
                            handleDeleteAccount();
                          }}
                          class="px-3.5 py-1.5 border border-red-500/20 text-[#ef4444] hover:bg-red-500/10 text-xs font-semibold rounded-lg transition-colors cursor-pointer"
                        >
                          Delete
                        </button>
                      </div>

                      {#if acc.password_history && acc.password_history.length > 0}
                        <button
                          onclick={() => showHistory[acc.id] = !showHistory[acc.id]}
                          class="text-xs text-[#06b6d4] hover:underline cursor-pointer font-semibold border-0 bg-transparent"
                        >
                          {showHistory[acc.id] ? "Hide History" : `History (${acc.password_history.length})`}
                        </button>
                      {/if}
                    </div>

                    <!-- History drawer -->
                    {#if showHistory[acc.id] && acc.password_history && acc.password_history.length > 0}
                      <div class="mt-3 space-y-2 bg-[#09090b] p-3 border border-[#27272a] rounded-lg">
                        <span class="text-[9px] font-bold text-[#a1a1aa] uppercase tracking-wider block">Password History</span>
                        <div class="space-y-2.5 max-h-32 overflow-y-auto pr-1">
                          {#each acc.password_history.slice().reverse() as hist, index}
                            {@const histKey = `${acc.id}_h_${index}`}
                            <div class="flex items-center justify-between gap-2 border-b border-[#27272a]/40 pb-2 last:border-0 last:pb-0">
                              <div class="flex flex-col min-w-0">
                                <span class="text-xs font-mono text-zinc-300 truncate select-all">
                                  {historyPasswordVisible[histKey]
                                    ? hist.password
                                    : "•".repeat(Math.max(1, hist.password.length))}
                                </span>
                                <span class="text-[9px] text-[#a1a1aa]">
                                  {new Date(hist.changed_at).toLocaleString()}
                                </span>
                              </div>
                              <div class="flex items-center gap-1 shrink-0">
                                <button
                                  onclick={() => historyPasswordVisible[histKey] = !historyPasswordVisible[histKey]}
                                  class="h-7 w-7 flex items-center justify-center bg-[#27272a] hover:bg-[#3f3f46] text-[#fafafa] rounded-lg transition-colors border border-[#27272a] cursor-pointer"
                                  title={historyPasswordVisible[histKey] ? "Hide password" : "Show password"}
                                >
                                  {#if historyPasswordVisible[histKey]}
                                    <EyeOff class="w-3.5 h-3.5" />
                                  {:else}
                                    <Eye class="w-3.5 h-3.5" />
                                  {/if}
                                </button>
                                <button
                                  onclick={() => copyText(hist.password, histKey)}
                                  class="h-7 w-7 flex items-center justify-center bg-[#27272a] hover:bg-[#3f3f46] text-[#fafafa] rounded-lg transition-colors border border-[#27272a] cursor-pointer"
                                  title="Copy password"
                                >
                                  {#if copiedStates[histKey]}
                                    <Check class="w-3 h-3 text-green-400" />
                                  {:else}
                                    <Copy class="w-3 h-3" />
                                  {/if}
                                </button>
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
              <div class="flex flex-col items-center justify-center py-12 text-zinc-400 gap-2 border border-[#27272a] rounded-lg bg-[#18181b]/50">
                <span class="text-2xl">👤</span>
                <p class="text-xs font-semibold text-zinc-400">No accounts in this group</p>
                <button
                  onclick={() => {
                    const item = vaultState.selectedItem;
                    if (item && item.type === 'DomainGroup') {
                      initCreateForm('DomainGroup', { title: item.title, url: item.urls[0] || '', tags: item.tags.join(', ') });
                    }
                  }}
                  class="mt-2 bg-[#fafafa] hover:bg-[#fafafa]/90 text-[#18181b] text-xs font-bold px-4 py-2 rounded-lg transition-colors cursor-pointer border-0"
                >
                  Add Account
                </button>
              </div>
            {/if}
          {:else}
            <!-- Secure note display -->
            <div class="space-y-2 border-t border-[#27272a] pt-4">
              <div class="flex justify-between items-center">
                <span class="text-[9px] font-bold text-[#a1a1aa] uppercase tracking-wider block">Secure Note Content</span>
                <button
                  onclick={() => {
                    if (vaultState.selectedItem?.type === 'SecureNote') {
                      copyText(vaultState.selectedItem.notes, 'note_n');
                    }
                  }}
                  class="text-[10px] text-[#06b6d4] hover:underline flex items-center gap-1 cursor-pointer border-0 bg-transparent"
                >
                  {#if copiedStates['note_n']}
                    <Check class="w-3 h-3 text-green-400" /> Copied!
                  {:else}
                    <Copy class="w-3 h-3" /> Copy Full Note
                  {/if}
                </button>
              </div>
              <div class="w-full bg-[#09090b] border border-[#27272a] rounded-lg p-4 text-sm text-zinc-100 whitespace-pre-wrap select-text font-mono leading-relaxed min-h-40">
                {vaultState.selectedItem.type === 'SecureNote' ? vaultState.selectedItem.notes : 'Empty note'}
              </div>
            </div>
          {/if}

          <!-- Metadata timestamps -->
          <div class="flex justify-between items-center pt-4 border-t border-[#27272a]">
            <div class="text-[10px] text-[#a1a1aa] flex flex-col gap-0.5">
              <span>Created: {new Date(vaultState.selectedItem.created_at || '').toLocaleString()}</span>
              <span>Updated: {new Date(vaultState.selectedItem.updated_at).toLocaleString()}</span>
            </div>

            <button
              onclick={() => {
                showConfirm(
                  "Move to Trash?",
                  "Are you sure you want to move this item to the trash? You can restore it later.",
                  async () => {
                    if (vaultState.selectedItem) {
                      await vaultState.deleteItem(vaultState.selectedItem.id);
                    }
                  }
                );
              }}
              class="text-xs border border-destructive/20 hover:border-destructive text-[#ef4444] px-4 py-2 rounded-lg transition-colors flex items-center gap-1.5 cursor-pointer bg-transparent"
            >
              Move to Trash
            </button>
          </div>
        </div>
      {:else}
        <!-- No Item Selected State -->
        <div class="grow flex flex-col items-center justify-center text-[#a1a1aa]">
          <div class="bg-[#18181b] border border-[#27272a] p-4 rounded-full mb-4">
            <Lock class="w-8 h-8 opacity-40" />
          </div>
          <p class="text-sm font-medium">Select an item to view details</p>
          <p class="text-xs text-zinc-500 mt-1">Or click "Add Entry" to create a new one.</p>
        </div>
      {/if}
    {/if}
  </div>
</div>
