<script lang="ts">
import {
  ArrowLeft,
  Check,
  CircleHelp,
  Cloud,
  CloudOff,
  Copy,
  Database,
  Download,
  ExternalLink,
  Eye,
  EyeOff,
  FileSpreadsheet,
  Fingerprint,
  FolderLock,
  Globe,
  KeyRound,
  LoaderCircle,
  Lock,
  LogOut,
  Plus,
  RefreshCw,
  Search,
  ShieldCheck,
  Sparkles,
  Trash2,
  Upload,
} from 'lucide-svelte';
import { onMount, untrack } from 'svelte';
import { wasm_decrypt_vault_packaged } from 'vault-core';
import { Button } from '~/components/ui/button/index.js';
import * as Card from '~/components/ui/card/index.js';
import { Input } from '~/components/ui/input/index.js';
import { Label } from '~/components/ui/label/index.js';
import AuthGate from '~/components/vault/AuthGate.svelte';
import PasswordGenerator from '~/components/vault/PasswordGenerator.svelte';
import ReauthModal from '~/components/vault/ReauthModal.svelte';
import SetupWizard from '~/components/vault/SetupWizard.svelte';
import {
  disableBiometrics,
  isBiometricsEnabled,
  isBiometricsSupported,
  registerBiometrics,
} from '~/lib/biometrics.js';
import { reauthController } from '~/lib/reauth-state.svelte.js';
import { getStorageProvider } from '~/lib/sync/factory.js';
import {
  type Account,
  type DomainGroup,
  type SecureNote,
  type VaultItem,
  vaultState,
} from '~/lib/vault-state.svelte.js';

// Widescreen state
let activeTab = $state<
  'vault' | 'generator' | 'sync' | 'settings' | 'trash' | 'help'
>('vault');
let widescreenPanel = $state<'none' | 'add' | 'detail'>('none');
// biome-ignore lint/suspicious/noExplicitAny: template compatibility
let selectedEntry = $state<any | null>(null);
// biome-ignore lint/suspicious/noExplicitAny: template compatibility
let selectedGroup = $state<any | null>(null);
// biome-ignore lint/suspicious/noExplicitAny: template compatibility
let selectedTrashEntry = $state<any | null>(null);
// biome-ignore lint/suspicious/noExplicitAny: template compatibility
let selectedTrashGroup = $state<any | null>(null);

// Forms bindings
let entryType = $state<'Login' | 'SecureNote'>('Login');
let newTitle = $state('');
let newUsername = $state('');
let newPassword = $state('');
let newUrl = $state('');
let newTotpSecret = $state('');
let newNotes = $state('');
let newTagsString = $state('');
let showNewPassword = $state(false);
let addError = $state('');

// Field-specific error states
let titleError = $state('');
let urlError = $state('');
let totpError = $state('');

// Widescreen editing states
let isWidescreenEditing = $state(false);
let showWidescreenHistory = $state(false);
let visibleWidescreenHistoryKeys = $state<Record<string, boolean>>({});
let editWideEntryType = $state<'Login' | 'SecureNote'>('Login');
let editWideTitle = $state('');
let editWideUsername = $state('');
let editWidePassword = $state('');
let editWideUrl = $state('');
let editWideTotpSecret = $state('');
let editWideNotes = $state('');
let editWideTagsString = $state('');
let editWideShowPassword = $state(false);
let editWideTitleError = $state('');
let editWideUrlError = $state('');
let editWideTotpError = $state('');
let editWideGlobalError = $state('');

// Search & Copy states
let searchQuery = $state('');
let showDetailPassword = $state(false);
let cardShowHistory = $state<Record<string, boolean>>({});
let cardHistoryPasswordVisible = $state<Record<string, boolean>>({});
let cardPasswordVisible = $state<Record<string, boolean>>({});
let cardTotpVisible = $state<Record<string, boolean>>({});
let copiedKey = $state<string | null>(null);

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
  const entry = selectedEntry;
  const ts = currentTimeSeconds;

  const newCodes: Record<string, string> = {};
  if (item?.accounts) {
    for (const acc of item.accounts) {
      if (acc.totp_secret) {
        newCodes[acc.id] = vaultState.generateTotp(acc.totp_secret, ts);
      }
    }
  }
  if (entry?.totp_secret) {
    newCodes[entry.id] = vaultState.generateTotp(entry.totp_secret, ts);
  }

  untrack(() => {
    totpCodes = newCodes;
  });
});

// Sync states
let googleUser = $state<{ email: string; name?: string } | null>(null);
let isSyncing = $state(false);
let syncError = $state('');
let syncSuccess = $state(false);

// Remote backup & overwrite states
let isConnecting = $state(false);
let remoteFileInfo = $state<{
  lastModified: string;
  sizeBytes?: number;
  entryCount?: number | string;
  isDifferentKey: boolean;
} | null>(null);

let localFileInfo = $derived.by(() => {
  let entryCount = 0;
  let trashCount = 0;
  try {
    const parsed = JSON.parse(vaultState.vaultJson);
    if (Array.isArray(parsed.items)) {
      entryCount = parsed.items.reduce((acc: number, item: VaultItem) => {
        return (
          acc + (item.type === 'DomainGroup' ? item.accounts?.length || 0 : 1)
        );
      }, 0);
    }
    if (Array.isArray(parsed.trash)) {
      trashCount = parsed.trash.reduce((acc: number, item: VaultItem) => {
        return (
          acc + (item.type === 'DomainGroup' ? item.accounts?.length || 0 : 1)
        );
      }, 0);
    }
  } catch {}
  return {
    entryCount,
    trashCount,
  };
});

let showRestoreConfirm = $state(false);
let showBackupConfirm = $state(false);
let showDeleteCloudConfirm = $state(false);
let isRestoring = $state(false);
let isBackingUp = $state(false);
let isDeletingCloud = $state(false);
let syncPassword = $state('');
let showSyncPassword = $state(false);
let syncMismatchError = $state('');

// Filtered entries derived rune
let filteredEntries = $derived.by(() => {
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

let displayTitleWide = $derived(
  selectedGroup ? selectedGroup.title : selectedEntry?.title || '',
);
let displayUrlWide = $derived(selectedGroup ? selectedGroup.urls[0] || '' : '');
let displayTagsWide = $derived(
  selectedGroup ? selectedGroup.tags : selectedEntry?.tags || [],
);
let isSecureNoteWide = $derived(!selectedGroup);

const fetchSyncStateInfo = async () => {
  remoteFileInfo = null;
  const provider = getStorageProvider('google');
  if (!provider) return;
  try {
    const auth = await provider.isAuthenticated();
    if (!auth) return;

    const result = await provider.downloadVault();
    if (!result) return;

    const sizeBytes = result.payload.length;
    const lastModified = result.metadata.lastModified;

    // Try to decrypt using the new packaged WASM decrypt helper
    let entryCount: number | string = 'Key Mismatch';
    let isDifferentKey = false;

    try {
      if (vaultState.masterKey) {
        const decryptedBytes = wasm_decrypt_vault_packaged(
          vaultState.masterKey,
          result.payload,
        );
        const decoder = new TextDecoder();
        const json = JSON.parse(decoder.decode(decryptedBytes));
        entryCount = json.entries?.length || 0;
      }
    } catch (e) {
      isDifferentKey = true;
    }

    remoteFileInfo = {
      lastModified,
      sizeBytes,
      entryCount,
      isDifferentKey,
    };
  } catch (err) {
    console.error('Error fetching sync details:', err);
  }
};

const checkConnection = async () => {
  const provider = getStorageProvider('google');
  if (provider && (await provider.isAuthenticated())) {
    googleUser = await provider.getUserInfo();
    await fetchSyncStateInfo();
  } else {
    googleUser = null;
    remoteFileInfo = null;
  }
};

const handleSignIn = async () => {
  syncError = '';
  syncSuccess = false;
  isConnecting = true;
  try {
    const provider = getStorageProvider('google');
    if (!provider) return;
    const success = await provider.signIn(true);
    if (success) {
      await checkConnection();
    } else {
      syncError = 'Failed to connect to Google Drive.';
    }
  } catch (e) {
    syncError =
      e instanceof Error ? e.message : 'Failed to connect to Google Drive.';
  } finally {
    isConnecting = false;
  }
};

const handleSignOut = async () => {
  syncError = '';
  syncSuccess = false;
  const provider = getStorageProvider('google');
  if (!provider) return;
  await provider.signOut();
  googleUser = null;
  remoteFileInfo = null;
};

const handleSyncNow = async () => {
  syncError = '';
  syncSuccess = false;
  isSyncing = true;
  try {
    const success = await vaultState.sync();
    if (success) {
      syncSuccess = true;
      await fetchSyncStateInfo();
    } else {
      syncError = vaultState.error || 'Sync failed.';
    }
  } catch (e) {
    syncError = e instanceof Error ? e.message : String(e);
  } finally {
    isSyncing = false;
  }
};

const handleRestoreRemote = async () => {
  syncError = '';
  syncSuccess = false;
  isRestoring = true;
  try {
    const success = await vaultState.restoreRemote();
    if (success) {
      syncSuccess = true;
      showRestoreConfirm = false;
      await fetchSyncStateInfo();
    } else if (vaultState.syncNeedsPassword) {
      showRestoreConfirm = false;
      syncPassword = '';
      syncMismatchError = vaultState.error || 'Master password required.';
    } else {
      syncError = vaultState.error || 'Restore failed.';
    }
  } catch (e) {
    syncError = e instanceof Error ? e.message : String(e);
  } finally {
    isRestoring = false;
  }
};

const handleBackupLocal = async () => {
  syncError = '';
  syncSuccess = false;
  isBackingUp = true;
  try {
    const success = await vaultState.backupLocal();
    if (success) {
      syncSuccess = true;
      showBackupConfirm = false;
      await fetchSyncStateInfo();
    } else {
      syncError = vaultState.error || 'Backup failed.';
    }
  } catch (e) {
    syncError = e instanceof Error ? e.message : String(e);
  } finally {
    isBackingUp = false;
  }
};

const handleSignOutAndDelete = async () => {
  syncError = '';
  syncSuccess = false;
  isDeletingCloud = true;
  try {
    const provider = getStorageProvider('google');
    if (provider?.deleteVault) {
      const deleted = await provider.deleteVault();
      if (!deleted) {
        throw new Error('Failed to delete remote vault backup file.');
      }
    }
    await handleSignOut();
    showDeleteCloudConfirm = false;
  } catch (e) {
    syncError = e instanceof Error ? e.message : String(e);
  } finally {
    isDeletingCloud = false;
  }
};

const handleResolveSyncMismatch = async (mode: 'merge' | 'restore') => {
  if (!syncPassword.trim()) return;

  syncMismatchError = '';
  isSyncing = true;
  try {
    let success = false;
    if (mode === 'merge') {
      success = await vaultState.resolveSyncSaltMismatch(syncPassword);
    } else {
      success = await vaultState.restoreRemote(syncPassword);
    }

    if (success) {
      syncPassword = '';
      syncMismatchError = '';
      syncSuccess = true;
      await fetchSyncStateInfo();
      setTimeout(() => {
        syncSuccess = false;
      }, 3000);
    } else {
      syncMismatchError = vaultState.error || 'Sync validation failed.';
    }
  } catch (e) {
    syncMismatchError = e instanceof Error ? e.message : String(e);
  } finally {
    isSyncing = false;
  }
};

const handleCancelSyncMismatch = () => {
  vaultState.cancelSyncMismatch();
  syncPassword = '';
  syncMismatchError = '';
};

const triggerCopy = async (text: string | undefined, key: string) => {
  if (!text) return;
  try {
    await navigator.clipboard.writeText(text);
    copiedKey = key;
    setTimeout(() => {
      copiedKey = null;
    }, 2000);
  } catch (e) {
    console.error('Failed to copy text:', e);
  }
};

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

const triggerCopySecure = async (text: string | undefined, key: string) => {
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
    widescreenPanel = 'none';
  }
};

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

// Debounce validation timers for widescreen editing
let editWideUrlTimeout: ReturnType<typeof setTimeout>;
let editWideTotpTimeout: ReturnType<typeof setTimeout>;

// Svelte 5 Reactive validation effects for widescreen editing
$effect(() => {
  const titleVal = editWideTitle;
  if (titleVal.trim()) {
    editWideTitleError = '';
  }
});

$effect(() => {
  const urlVal = editWideUrl;
  if (!urlVal.trim()) {
    editWideUrlError = '';
    return;
  }
  clearTimeout(editWideUrlTimeout);
  editWideUrlTimeout = setTimeout(() => {
    const res = processUrl(urlVal);
    if (!res.isValid) {
      editWideUrlError = 'Please enter a valid website URL.';
    } else {
      editWideUrlError = '';
    }
  }, 400);
});

$effect(() => {
  const totpVal = editWideTotpSecret;
  if (!totpVal.trim()) {
    editWideTotpError = '';
    return;
  }
  clearTimeout(editWideTotpTimeout);
  editWideTotpTimeout = setTimeout(() => {
    if (!isValidBase32(totpVal)) {
      editWideTotpError =
        'Invalid TOTP Secret. Base32 keys only use A-Z and 2-7.';
    } else {
      editWideTotpError = '';
    }
  }, 400);
});

// Trigger editing mode and initialize values
const startWidescreenEditing = () => {
  if (!selectedEntry) return;
  if (selectedGroup) {
    editWideEntryType = 'Login';
    editWideTitle = selectedGroup.title;
    editWideUsername = selectedEntry.username || '';
    editWidePassword = selectedEntry.password || '';
    editWideUrl = selectedGroup.urls?.[0] || '';
    editWideTotpSecret = selectedEntry.totp_secret || '';
    editWideNotes = selectedEntry.notes || '';
    editWideTagsString = selectedGroup.tags
      ? selectedGroup.tags.join(', ')
      : '';
  } else {
    editWideEntryType = 'SecureNote';
    editWideTitle = selectedEntry.title;
    editWideUsername = '';
    editWidePassword = '';
    editWideUrl = '';
    editWideTotpSecret = '';
    editWideNotes = selectedEntry.notes || '';
    editWideTagsString = selectedEntry.tags
      ? selectedEntry.tags.join(', ')
      : '';
  }
  editWideShowPassword = false;
  editWideGlobalError = '';
  editWideTitleError = '';
  editWideUrlError = '';
  editWideTotpError = '';
  skipResetWidescreenEditing = true;
  isWidescreenEditing = true;
};

const cancelWidescreenEdit = () => {
  isWidescreenEditing = false;
  if (selectedGroup) {
    widescreenPanel = 'none';
  }
};

const handleSaveWidescreenEdit = async () => {
  if (!selectedEntry) return;

  editWideGlobalError = '';
  editWideTitleError = '';
  editWideUrlError = '';
  editWideTotpError = '';

  // Synchronous immediate validation checks
  if (!editWideTitle.trim()) {
    editWideTitleError = 'Title is required.';
    return;
  }

  let formattedUrl = '';
  if (editWideEntryType === 'Login' && editWideUrl.trim()) {
    const urlResult = processUrl(editWideUrl);
    if (!urlResult.isValid) {
      editWideUrlError = 'Please enter a valid website URL.';
      return;
    }
    formattedUrl = urlResult.formatted;
  }

  if (editWideEntryType === 'Login' && editWideTotpSecret.trim()) {
    if (!isValidBase32(editWideTotpSecret)) {
      editWideTotpError =
        'Invalid TOTP Secret. Base32 keys only use A-Z and 2-7.';
      return;
    }
  }

  // Parse comma-separated tags
  const tags = editWideTagsString
    .split(',')
    .map((t) => t.trim())
    .filter((t) => t.length > 0);

  const success = await vaultState.updateEntry(
    selectedEntry.id,
    editWideTitle,
    editWideUsername,
    editWidePassword,
    formattedUrl,
    editWideNotes,
    editWideEntryType,
    editWideTotpSecret,
    tags,
  );

  if (success) {
    if (editWideEntryType === 'SecureNote') {
      const updated = vaultState.vault.items.find(
        (e: VaultItem) => e.type === 'SecureNote' && e.id === selectedEntry?.id,
      );
      if (updated) {
        selectedEntry = updated;
      }
      isWidescreenEditing = false;
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
      isWidescreenEditing = false;
      widescreenPanel = 'none';
    }
  } else {
    editWideGlobalError = vaultState.error;
  }
};

const handleQuickGenerateEditWide = () => {
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
    editWidePassword = result.credential;
    editWideShowPassword = true;
  }
};

let skipResetWidescreenEditing = false;

$effect(() => {
  // Cancel edit mode and hide history when selectedEntry changes
  const _ = selectedEntry;
  if (skipResetWidescreenEditing) {
    skipResetWidescreenEditing = false;
    return;
  }
  isWidescreenEditing = false;
  showWidescreenHistory = false;
  visibleWidescreenHistoryKeys = {};
});

const handleRestoreTrashEntry = async (id: string) => {
  const success = await vaultState.restoreEntry(id);
  if (success) {
    selectedTrashEntry = null;
  }
};

const handlePurgeTrashEntry = async (id: string) => {
  if (
    confirm(
      'Are you sure you want to permanently delete this credential? This action cannot be undone.',
    )
  ) {
    const success = await vaultState.purgeEntry(id);
    if (success) {
      selectedTrashEntry = null;
    }
  }
};

const handleEmptyTrash = async () => {
  if (
    confirm(
      'Are you sure you want to permanently empty the trash? All deleted credentials will be destroyed forever.',
    )
  ) {
    if (vaultState.vault.trash) {
      for (const entry of [...vaultState.vault.trash]) {
        await vaultState.purgeEntry(entry.id);
      }
      selectedTrashEntry = null;
    }
  }
};

const handleAddEntry = async () => {
  addError = '';
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
    widescreenPanel = 'none';
  } else {
    addError = vaultState.error;
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

const handleDeleteEntry = async (id: string) => {
  let success = false;
  if (selectedGroup && selectedEntry && selectedEntry.id === id) {
    success = await vaultState.deleteAccount(selectedGroup.id, id);
  } else {
    success = await vaultState.deleteEntry(id);
  }
  if (success) {
    selectedEntry = null;
    selectedGroup = null;
    widescreenPanel = 'none';
  }
};

// Settings actions
let showResetConfirm = $state(false);
let resetConfirmText = $state('');
let resetError = $state('');

let showExportConfirm = $state(false);
let exportType = $state<'json' | 'csv' | null>(null);
let exportPassword = $state('');
let exportError = $state('');
let showExportPassword = $state(false);

const requestExport = (type: 'json' | 'csv') => {
  exportType = type;
  exportPassword = '';
  exportError = '';
  showExportPassword = false;
  showExportConfirm = true;
};

const handleConfirmExport = async () => {
  exportError = '';
  if (!exportPassword) {
    exportError = 'Master password is required.';
    return;
  }

  const isValid = await vaultState.verifyPassword(exportPassword);
  if (!isValid) {
    exportError = 'Incorrect master password.';
    return;
  }

  if (exportType === 'json') {
    executeExportJson();
  } else if (exportType === 'csv') {
    executeExportCsv();
  }

  showExportConfirm = false;
  exportPassword = '';
  exportType = null;
};

const executeExportJson = () => {
  if (!vaultState.vaultJson) return;
  const blob = new Blob([vaultState.vaultJson], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `keyvault-export-${new Date().toISOString().split('T')[0]}.json`;
  a.click();
  URL.revokeObjectURL(url);
};

const executeExportCsv = () => {
  if (!vaultState.isUnlocked) return;

  try {
    const csvContent = vaultState.exportCsv();
    if (!csvContent) return;
    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `keyvault-export-${new Date().toISOString().split('T')[0]}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  } catch (err) {
    console.error('Failed to export CSV:', err);
  }
};

let importSuccessMsg = $state('');
let importErrorMsg = $state('');

const handleImportCsv = async (event: Event) => {
  importSuccessMsg = '';
  importErrorMsg = '';

  const target = event.target as HTMLInputElement;
  const file = target.files?.[0];
  if (!file) return;

  try {
    const text = await file.text();
    const success = await vaultState.importCsv(text);
    if (success) {
      importSuccessMsg = 'Credentials imported successfully!';
    } else {
      importErrorMsg = vaultState.error || 'Failed to import CSV.';
    }
  } catch (err) {
    importErrorMsg = err instanceof Error ? err.message : String(err);
  } finally {
    target.value = '';
  }
};

const handleResetVault = async () => {
  resetError = '';
  if (resetConfirmText.trim() !== 'RESET') {
    resetError = "Please type 'RESET' to confirm.";
    return;
  }
  const success = await vaultState.resetVault();
  if (success) {
    showResetConfirm = false;
    resetConfirmText = '';
    activeTab = 'vault';
  } else {
    resetError = vaultState.error || 'Failed to reset vault.';
  }
};

// Biometrics settings
let bioSupported = $state(false);
let bioEnabled = $state(false);
let bioError = $state('');

const handleToggleBiometrics = async () => {
  try {
    bioError = '';
    if (bioEnabled) {
      await disableBiometrics();
      bioEnabled = false;
    } else {
      const success = await registerBiometrics();
      if (success) {
        bioEnabled = true;
      }
    }
  } catch (e) {
    console.error('Biometric toggle failed:', e);
    bioError = e instanceof Error ? e.message : 'Verification failed.';
    bioEnabled = await isBiometricsEnabled();
  }
};

onMount(async () => {
  await vaultState.initWasm();
  await checkConnection();
  bioSupported = await isBiometricsSupported();
  bioEnabled = await isBiometricsEnabled();
});
</script>

<div
    class="flex h-screen w-screen bg-[#09090b] text-zinc-100 overflow-hidden font-sans select-none"
>
    <!-- 1. LEFT SIDEBAR -->
    <aside
        class="w-64 bg-zinc-950/50 border-r border-zinc-900 flex flex-col justify-between p-4 shrink-0"
    >
        <div class="space-y-6">
            <!-- App Header Branding -->
            <div class="flex items-center gap-2 px-2 py-1">
                <Lock class="h-5 w-5 text-zinc-200" />
                <span class="font-bold text-base tracking-tight text-white"
                    >KeyVault</span
                >
            </div>

            <!-- Navigation Links -->
            <nav class="space-y-1">
                <button
                    class="w-full text-left px-3 py-2 rounded-lg text-xs font-semibold flex items-center gap-2.5 transition-colors"
                    class:bg-zinc-800={activeTab === "vault"}
                    class:text-white={activeTab === "vault"}
                    class:text-zinc-400={activeTab !== "vault"}
                    class:hover:text-zinc-200={activeTab !== "vault"}
                    onclick={() => (activeTab = "vault")}
                >
                    <FolderLock class="h-4 w-4" /> My Vault
                </button>
                <button
                    class="w-full text-left px-3 py-2 rounded-lg text-xs font-semibold flex items-center gap-2.5 transition-colors"
                    class:bg-zinc-800={activeTab === "generator"}
                    class:text-white={activeTab === "generator"}
                    class:text-zinc-400={activeTab !== "generator"}
                    class:hover:text-zinc-200={activeTab !== "generator"}
                    onclick={() => (activeTab = "generator")}
                >
                    <Sparkles class="h-4 w-4 text-accent" /> Generator
                </button>
                <button
                    class="w-full text-left px-3 py-2 rounded-lg text-xs font-semibold flex items-center gap-2.5 transition-colors"
                    class:bg-zinc-800={activeTab === "trash"}
                    class:text-white={activeTab === "trash"}
                    class:text-zinc-400={activeTab !== "trash"}
                    class:hover:text-zinc-200={activeTab !== "trash"}
                    onclick={() => {
                        activeTab = "trash";
                        selectedTrashEntry = null;
                    }}
                >
                    <Trash2 class="h-4 w-4 text-red-400/80" /> Trash
                </button>
                <button
                    class="w-full text-left px-3 py-2 rounded-lg text-xs font-semibold flex items-center gap-2.5 transition-colors"
                    class:bg-zinc-800={activeTab === "sync"}
                    class:text-white={activeTab === "sync"}
                    class:text-zinc-400={activeTab !== "sync"}
                    class:hover:text-zinc-200={activeTab !== "sync"}
                    onclick={() => (activeTab = "sync")}
                >
                    <Globe class="h-4 w-4" /> Sync Config
                </button>
                <button
                    class="w-full text-left px-3 py-2 rounded-lg text-xs font-semibold flex items-center gap-2.5 transition-colors"
                    class:bg-zinc-800={activeTab === "settings"}
                    class:text-white={activeTab === "settings"}
                    class:text-zinc-400={activeTab !== "settings"}
                    class:hover:text-zinc-200={activeTab !== "settings"}
                    onclick={() => (activeTab = "settings")}
                >
                    <RefreshCw class="h-4 w-4" /> Settings
                </button>
                <button
                    class="w-full text-left px-3 py-2 rounded-lg text-xs font-semibold flex items-center gap-2.5 transition-colors"
                    class:bg-zinc-800={activeTab === "help"}
                    class:text-white={activeTab === "help"}
                    class:text-zinc-400={activeTab !== "help"}
                    class:hover:text-zinc-200={activeTab !== "help"}
                    onclick={() => (activeTab = "help")}
                >
                    <CircleHelp class="h-4 w-4 text-[#06b6d4]" /> Help
                </button>
            </nav>
        </div>

        <!-- Sidebar Bottom Controls -->
        <div class="border-t border-zinc-900 pt-4 space-y-3">
            {#if googleUser}
                <div class="px-2">
                    <p
                        class="text-[10px] text-zinc-400 uppercase tracking-wider font-semibold"
                    >
                        Cloud Connected
                    </p>
                    <p
                        class="text-xs text-zinc-300 truncate"
                        title={googleUser.email}
                    >
                        {googleUser.email}
                    </p>
                </div>
            {/if}

            <div
                class="flex items-center justify-between px-2 text-[10px] text-zinc-400 font-medium select-none"
            >
                <span class="flex items-center gap-1"
                    ><ShieldCheck class="h-3 w-3 text-green-500" /> Secure WASM</span
                >
            </div>

            {#if vaultState.isUnlocked}
                <Button
                    variant="outline"
                    class="w-full bg-zinc-900/50 border-zinc-800 hover:bg-zinc-800 text-zinc-400 hover:text-white text-xs h-8.5 font-medium flex items-center gap-2"
                    onclick={() => {
                        vaultState.lock();
                        widescreenPanel = "none";
                        selectedEntry = null;
                    }}
                >
                    <LogOut class="h-3.5 w-3.5" /> Lock Vault
                </Button>
            {/if}
        </div>
    </aside>

    <!-- 2. MAIN CONTENT AREA -->
    <main class="grow overflow-hidden flex flex-col">
        {#if !vaultState.wasmReady}
            <!-- Loading view -->
            <div class="grow flex flex-col items-center justify-center gap-4">
                <div
                    class="animate-spin rounded-full h-8 w-8 border-2 border-zinc-800 border-t-zinc-400"
                ></div>
                <p class="text-xs text-zinc-400">
                    Loading secure cryptographic engine...
                </p>
            </div>
        {:else if !vaultState.isRegistered}
            <!-- Wizard Setup view -->
            <div class="grow flex items-center justify-center p-6 overflow-y-auto">
                <div class="w-full max-w-xl bg-zinc-900/20 border border-zinc-800/80 rounded-xl p-6 md:p-8 backdrop-blur-md shadow-2xl">
                    <SetupWizard onComplete={() => activeTab = "help"} />
                </div>
            </div>
        {:else if !vaultState.isUnlocked && activeTab !== "generator"}
            <!-- Locked view -->
            <div class="grow flex items-center justify-center p-6">
                <Card.Root
                    class="bg-zinc-900/40 border-zinc-800 backdrop-blur-md max-w-md w-full p-6 shadow-2xl"
                >
                    <AuthGate />
                </Card.Root>
            </div>
        {:else}
            <!-- Unlocked Views -->
            {#if activeTab === "vault"}
                <!-- Split Widescreen Vault layout -->
                <div class="grow flex overflow-hidden">
                    <!-- Split pane left: Search and List -->
                    <div
                        class="w-80 border-r border-zinc-900 flex flex-col justify-between shrink-0 bg-zinc-950/20"
                    >
                        <!-- Search & Actions bar -->
                        <div
                            class="p-4 border-b border-zinc-900 space-y-3 shrink-0"
                        >
                            <div class="relative flex items-center">
                                <Search
                                    class="absolute left-3 h-4 w-4 text-zinc-400"
                                />
                                <Input
                                    type="text"
                                    placeholder="Search credentials..."
                                    bind:value={searchQuery}
                                    class="bg-zinc-900 border-zinc-800 text-white pl-9 text-xs h-9 focus-visible:ring-1 focus-visible:ring-zinc-700"
                                />
                            </div>

                            <div
                                class="flex justify-between items-center text-[10px]"
                            >
                                <span
                                    class="text-zinc-400 font-semibold uppercase tracking-wider"
                                    >{filteredEntries.length} ITEMS</span
                                >
                                <Button
                                    variant="outline"
                                    class="bg-zinc-900 border-zinc-800 hover:bg-zinc-800 text-zinc-200 hover:text-white h-7 px-2.5 text-xs flex items-center gap-1"
                                    onclick={() => {
                                        selectedEntry = null;
                                        widescreenPanel = "add";
                                    }}
                                >
                                    <Plus class="h-3 w-3" /> Add Entry
                                </Button>
                            </div>
                        </div>

                        <!-- Credentials Listings -->
                        <div class="grow overflow-y-auto p-2.5 space-y-1.5">
                            {#if filteredEntries.length === 0}
                                <div
                                    class="flex flex-col items-center justify-center py-20 text-zinc-600 gap-2"
                                >
                                    <span class="text-3xl">🔍</span>
                                    <p class="text-xs font-semibold">
                                        No credentials found
                                    </p>
                                </div>
                            {:else}
                                {#each filteredEntries as item (item.id)}
                                    {#if item.type === "SecureNote"}
                                        <button
                                            class="w-full flex items-center gap-3 p-2.5 rounded-lg border text-left transition-all duration-250 select-none min-w-0 {selectedEntry?.id === item.id
                                                ? 'bg-zinc-900 border-zinc-800'
                                                : 'border-transparent hover:bg-zinc-900/50'}"
                                            onclick={() => {
                                                selectedGroup = null;
                                                selectedEntry = item;
                                                widescreenPanel = "detail";
                                                showDetailPassword = false;
                                            }}
                                        >
                                            <div class="h-8 w-8 rounded-full bg-zinc-800 border border-zinc-700 text-zinc-300 flex items-center justify-center font-bold uppercase text-xs shrink-0">
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
                                                    <span class="text-[10px] text-zinc-500 truncate mt-0.5">
                                                        {item.tags.join(', ')}
                                                    </span>
                                                {/if}
                                            </div>
                                        </button>
                                    {:else}
                                        <button
                                            class="w-full flex items-center gap-3 p-2.5 rounded-lg border text-left transition-all duration-250 select-none min-w-0 {selectedGroup?.id === item.id && !selectedEntry
                                                ? 'bg-zinc-900 border-zinc-800'
                                                : 'border-transparent hover:bg-zinc-900/50'}"
                                            onclick={() => {
                                                selectedGroup = item;
                                                selectedEntry = null;
                                                widescreenPanel = "none";
                                            }}
                                        >
                                            <div class="h-8 w-8 rounded-full bg-zinc-800 border border-zinc-700 text-zinc-300 flex items-center justify-center font-bold uppercase text-xs shrink-0">
                                                🌐
                                            </div>
                                            <div class="flex flex-col min-w-0">
                                                <div class="flex items-center gap-1.5 min-w-0">
                                                    <span class="text-xs font-semibold text-white truncate">{item.title}</span>
                                                    <span class="bg-cyan-500/10 text-cyan-400 border border-cyan-500/20 text-[8px] font-bold uppercase tracking-wider px-1 py-0.2 rounded select-none shrink-0">
                                                        {item.accounts.length} {item.accounts.length === 1 ? 'acc' : 'accs'}
                                                    </span>
                                                </div>
                                                {#if item.tags && item.tags.length > 0}
                                                    <span class="text-[10px] text-zinc-500 truncate mt-0.5">
                                                        {item.tags.join(', ')}
                                                    </span>
                                                {:else if item.urls[0]}
                                                    <span class="text-[10px] text-zinc-400 truncate mt-0.5">
                                                        {item.urls[0].replace(/^https?:\/\/(www\.)?/, "")}
                                                    </span>
                                                {/if}
                                            </div>
                                        </button>
                                    {/if}
                                {/each}
                            {/if}
                        </div>
                    </div>

                    <!-- Split pane right: Panel details or Add forms -->
                    <div
                        class="grow overflow-y-auto p-8 flex flex-col justify-between"
                    >
                        {#if widescreenPanel === "none"}
                            {#if selectedGroup}
                                <!-- Group Overview Widescreen View -->
                                <div class="max-w-xl mx-auto w-full space-y-6">
                                    <div class="border-b border-zinc-900 pb-3 flex items-center justify-between animate-fade-in">
                                        <div class="flex items-center gap-3">
                                            <div class="h-10 w-10 rounded-full bg-zinc-900 border border-zinc-800 flex items-center justify-center text-zinc-400">
                                                🌐
                                            </div>
                                            <div>
                                                <h2 class="text-base font-bold text-white leading-tight">
                                                    {selectedGroup.title}
                                                </h2>
                                                {#if selectedGroup.urls[0]}
                                                    <span class="text-xs text-zinc-400">{selectedGroup.urls[0]}</span>
                                                {/if}
                                            </div>
                                        </div>
                                        <Button
                                            variant="outline"
                                            size="sm"
                                            class="bg-zinc-900 border-zinc-800 text-zinc-300 hover:text-white h-8 text-xs flex items-center gap-1"
                                            onclick={() => {
                                                entryType = "Login";
                                                newUrl = selectedGroup.urls[0] || "";
                                                newTitle = selectedGroup.title;
                                                newUsername = "";
                                                newPassword = "";
                                                newNotes = "";
                                                newTagsString = selectedGroup.tags ? selectedGroup.tags.join(", ") : "";
                                                widescreenPanel = "add";
                                            }}
                                        >
                                            <Plus class="h-3.5 w-3.5" /> Add Account
                                        </Button>
                                    </div>

                                    {#if selectedGroup.tags && selectedGroup.tags.length > 0}
                                        <div class="flex flex-wrap gap-1.5 pt-0.5">
                                            {#each selectedGroup.tags as tag}
                                                <span class="bg-zinc-900 text-zinc-300 border border-zinc-800 px-2.5 py-0.5 rounded-full text-[9px] font-semibold">
                                                    {tag}
                                                </span>
                                            {/each}
                                        </div>
                                    {/if}

                                    <div class="space-y-4 pt-2">
                                        {#each selectedGroup.accounts as account (account.id)}
                                            <div class="p-4 bg-[#18181b] border border-[#27272a] rounded-lg space-y-4 shadow-sm">

                                                <!-- Username -->
                                                <div class="space-y-1.5">
                                                    <Label class="text-[9px] font-bold text-zinc-500 uppercase tracking-wider block">Username</Label>
                                                    <div class="relative flex items-center">
                                                        <Input
                                                            type="text"
                                                            readonly
                                                            value={account.username || ""}
                                                            placeholder="No Username"
                                                            class="bg-zinc-950 border-zinc-900 text-white pr-9 text-xs h-9"
                                                        />
                                                        {#if account.username}
                                                            <div class="absolute right-1 top-1/2 -translate-y-1/2 flex items-center">
                                                                <Button
                                                                    variant="ghost"
                                                                    size="icon"
                                                                    class="h-7 w-7 text-zinc-400 hover:text-white"
                                                                    onclick={() => triggerCopy(account.username, 'acc_u_' + account.id)}
                                                                    title="Copy Username"
                                                                >
                                                                    {#if copiedKey === 'acc_u_' + account.id}
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
                                                <div class="space-y-1.5">
                                                    <Label class="text-[9px] font-bold text-zinc-500 uppercase tracking-wider block">Password</Label>
                                                    <div class="relative flex items-center">
                                                        <Input
                                                            type={cardPasswordVisible['acc_p_' + account.id] ? "text" : "password"}
                                                            readonly
                                                            value={account.password || ""}
                                                            placeholder="No Password"
                                                            class="bg-zinc-950 border-zinc-900 text-white pr-16 text-xs h-9"
                                                        />
                                                        <div class="absolute right-1 top-1/2 -translate-y-1/2 flex items-center gap-0.5">
                                                            <Button
                                                                variant="ghost"
                                                                size="icon"
                                                                class="h-7 w-7 text-zinc-400 hover:text-white"
                                                                onclick={() => toggleCardPassword('acc_p_' + account.id)}
                                                                title={cardPasswordVisible['acc_p_' + account.id] ? "Hide password" : "Show password"}
                                                            >
                                                                {#if cardPasswordVisible['acc_p_' + account.id]}
                                                                    <EyeOff class="h-3.5 w-3.5" />
                                                                {:else}
                                                                    <Eye class="h-3.5 w-3.5" />
                                                                {/if}
                                                            </Button>
                                                            {#if account.password}
                                                                <Button
                                                                    variant="ghost"
                                                                    size="icon"
                                                                    class="h-7 w-7 text-zinc-400 hover:text-white"
                                                                    onclick={() => triggerCopySecure(account.password, 'acc_p_' + account.id)}
                                                                    title="Copy Password"
                                                                >
                                                                    {#if copiedKey === 'acc_p_' + account.id}
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
                                                    <div class="space-y-1.5">
                                                        <Label class="text-[9px] font-bold text-zinc-500 uppercase tracking-wider block">Sites</Label>
                                                        <a
                                                            class="text-xs text-accent hover:underline flex items-center gap-1 mt-0.5 min-w-0 max-w-full"
                                                            href={selectedGroup.urls[0]}
                                                            target="_blank"
                                                            rel="noreferrer"
                                                        >
                                                            <span class="truncate">{selectedGroup.urls[0]}</span>
                                                            <ExternalLink class="h-3.5 w-3.5 inline shrink-0" />
                                                        </a>
                                                    </div>
                                                {/if}

                                                <!-- TOTP Secret -->
                                                {#if account.totp_secret}
                                                    <!-- 6-digit TOTP Code Display -->
                                                    <div class="space-y-1.5">
                                                        <Label class="text-[9px] font-bold text-zinc-500 uppercase tracking-wider block">One-Time Password (TOTP)</Label>
                                                        <div class="relative flex items-center">
                                                            <div class="flex-1 bg-zinc-950 border border-zinc-900 rounded-lg px-2.5 py-1 flex items-center justify-between min-w-0 h-9 pr-10">
                                                                {#if totpCodes[account.id]}
                                                                    <span class="font-mono text-sm font-bold tracking-[0.2em] text-[#06b6d4] select-all leading-none">
                                                                        {totpCodes[account.id].slice(0, 3)} {totpCodes[account.id].slice(3)}
                                                                    </span>
                                                                {:else}
                                                                    <span class="text-[11px] text-zinc-500 italic animate-pulse">Generating...</span>
                                                                {/if}

                                                                <!-- Visual countdown bar/indicator -->
                                                                <div class="flex items-center gap-1.5 shrink-0">
                                                                    <div class="w-12 h-1.5 bg-zinc-800 rounded-full overflow-hidden">
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
                                                                class="h-7 w-7 text-zinc-400 hover:text-white shrink-0 absolute right-1"
                                                                onclick={() => triggerCopy(totpCodes[account.id], 'acc_totp_code_' + account.id)}
                                                                title="Copy TOTP Code"
                                                                disabled={!totpCodes[account.id]}
                                                            >
                                                                {#if copiedKey === 'acc_totp_code_' + account.id}
                                                                    <Check class="h-3.5 w-3.5 text-green-400" />
                                                                {:else}
                                                                    <Copy class="h-3.5 w-3.5" />
                                                                {/if}
                                                            </Button>
                                                        </div>
                                                    </div>

                                                    <div class="space-y-1.5">
                                                        <Label class="text-[9px] font-bold text-zinc-500 uppercase tracking-wider block">TOTP Secret</Label>
                                                        <div class="relative flex items-center">
                                                            <Input
                                                                type={cardTotpVisible['acc_t_' + account.id] ? "text" : "password"}
                                                                readonly
                                                                value={account.totp_secret}
                                                                class="bg-zinc-950 border-zinc-900 text-white pr-16 text-xs h-9"
                                                            />
                                                            <div class="absolute right-1 top-1/2 -translate-y-1/2 flex items-center gap-0.5">
                                                                <Button
                                                                    variant="ghost"
                                                                    size="icon"
                                                                    class="h-7 w-7 text-zinc-400 hover:text-white"
                                                                    onclick={() => toggleCardTotp('acc_t_' + account.id)}
                                                                    title={cardTotpVisible['acc_t_' + account.id] ? "Hide TOTP Secret" : "Show TOTP Secret"}
                                                                >
                                                                    {#if cardTotpVisible['acc_t_' + account.id]}
                                                                        <EyeOff class="h-3.5 w-3.5" />
                                                                    {:else}
                                                                        <Eye class="h-3.5 w-3.5" />
                                                                    {/if}
                                                                </Button>
                                                                <Button
                                                                    variant="ghost"
                                                                    size="icon"
                                                                    class="h-7 w-7 text-zinc-400 hover:text-white"
                                                                    onclick={() => triggerCopySecure(account.totp_secret, 'acc_t_' + account.id)}
                                                                    title="Copy TOTP Secret"
                                                                >
                                                                    {#if copiedKey === 'acc_t_' + account.id}
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
                                                <div class="space-y-1.5">
                                                    <Label class="text-[9px] font-bold text-zinc-500 uppercase tracking-wider block">Note</Label>
                                                    <div class="w-full bg-[#09090b] border border-[#27272a] rounded-lg text-zinc-300 p-3 text-xs min-h-12 overflow-y-auto whitespace-pre-wrap select-all font-sans leading-normal">
                                                        {account.notes || "No note added"}
                                                    </div>
                                                </div>

                                                <!-- Actions Row -->
                                                <div class="flex justify-between items-center pt-2 border-t border-zinc-800">
                                                    <div class="flex items-center gap-2">
                                                        <Button
                                                            variant="outline"
                                                            size="sm"
                                                            class="h-8 text-xs border-zinc-800 bg-zinc-900 text-zinc-300 hover:text-white px-3 cursor-pointer"
                                                            onclick={() => {
                                                                selectedEntry = account;
                                                                widescreenPanel = "detail";
                                                                isWidescreenEditing = false;
                                                                startWidescreenEditing();
                                                            }}
                                                        >
                                                            Edit
                                                        </Button>
                                                        <Button
                                                            variant="outline"
                                                            size="sm"
                                                            class="h-8 text-xs border-red-950 text-red-400 hover:bg-red-950/20 px-3 cursor-pointer"
                                                            onclick={() => handleDeleteAccountDirect(account)}
                                                        >
                                                            Delete
                                                        </Button>
                                                    </div>

                                                    {#if account.password_history && account.password_history.length > 0}
                                                        <Button
                                                            variant="ghost"
                                                            size="sm"
                                                            class="h-8 text-xs text-accent hover:text-white p-0 hover:bg-transparent cursor-pointer"
                                                            onclick={() => cardShowHistory[account.id] = !cardShowHistory[account.id]}
                                                        >
                                                            {cardShowHistory[account.id] ? "Hide History" : `History (${account.password_history.length})`}
                                                        </Button>
                                                    {/if}
                                                </div>

                                                <!-- Password History List -->
                                                {#if cardShowHistory[account.id] && account.password_history && account.password_history.length > 0}
                                                    <div class="mt-2 space-y-1.5 bg-zinc-950/40 p-2.5 border border-zinc-800 rounded-lg">
                                                        <Label class="text-[9px] font-bold text-zinc-500 uppercase tracking-wider block">Password History</Label>
                                                        <div class="space-y-2 max-h-32 overflow-y-auto pr-1">
                                                            {#each account.password_history.slice().reverse() as hist, index}
                                                                {@const histKey = `${account.id}_h_${index}`}
                                                                <div class="flex items-center justify-between gap-2 border-b border-zinc-900/50 pb-1.5 last:border-0 last:pb-0">
                                                                    <div class="flex flex-col min-w-0">
                                                                        <span class="text-xs font-mono text-zinc-300 truncate select-all">
                                                                            {cardHistoryPasswordVisible[histKey]
                                                                                ? hist.password
                                                                                : "•".repeat(Math.max(1, hist.password.length))}
                                                                        </span>
                                                                        <span class="text-[9px] text-zinc-500">
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
                                                                                {#if copiedKey === histKey}
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
                                </div>
                            {:else}
                                <!-- Placeholder details panel -->
                                <div
                                    class="m-auto flex flex-col items-center text-center max-w-sm space-y-4 py-20 select-none"
                                >
                                    <div
                                        class="h-12 w-12 rounded-full bg-zinc-900 border border-zinc-800 flex items-center justify-center text-zinc-400"
                                    >
                                        <ShieldCheck class="h-6 w-6" />
                                    </div>
                                    <div class="space-y-1">
                                        <h3
                                            class="text-sm font-semibold text-white"
                                        >
                                            No Group or Note Selected
                                        </h3>
                                        <p
                                            class="text-xs text-zinc-400 leading-relaxed"
                                        >
                                            Select a group or note from the sidebar
                                            to view its accounts, copy credentials, or edit
                                            records.
                                        </p>
                                    </div>
                                    <Button
                                        variant="outline"
                                        class="bg-zinc-900 border-zinc-800 text-white text-xs h-9 font-medium"
                                        onclick={() => {
                                            selectedEntry = null;
                                            selectedGroup = null;
                                            widescreenPanel = "add";
                                        }}
                                    >
                                        Create New Credential
                                    </Button>
                                </div>
                            {/if}
                        {:else if widescreenPanel === "add"}
                            <!-- Add record widescreen view -->
                            <div class="max-w-xl mx-auto w-full space-y-6">
                                <div
                                    class="border-b border-zinc-900 pb-3 flex items-center justify-between"
                                >
                                    <h2
                                        class="text-base font-bold text-white tracking-tight"
                                    >
                                        Create New Entry
                                    </h2>
                                    <Button
                                        variant="ghost"
                                        size="sm"
                                        class="text-xs text-zinc-400"
                                        onclick={() =>
                                            (widescreenPanel = "none")}
                                        >Cancel</Button
                                    >
                                </div>

                                <div class="space-y-4">
                                    <!-- Entry Type Selector -->
                                    <div class="grid grid-cols-2 gap-1 bg-zinc-900/50 p-1 border border-zinc-800 rounded-lg">
                                        <button
                                            type="button"
                                            class="text-xs py-1 h-7.5 font-semibold rounded-md transition-all outline-none {entryType === 'Login' ? 'bg-zinc-800 text-white' : 'text-zinc-400 hover:text-zinc-200'}"
                                            onclick={() => (entryType = 'Login')}
                                        >
                                            Login
                                        </button>
                                        <button
                                            type="button"
                                            class="text-xs py-1 h-7.5 font-semibold rounded-md transition-all outline-none {entryType === 'SecureNote' ? 'bg-zinc-800 text-white' : 'text-zinc-400 hover:text-zinc-200'}"
                                            onclick={() => (entryType = 'SecureNote')}
                                        >
                                            Secure Note
                                        </button>
                                    </div>

                                    <!-- Common Title Field -->
                                    <div class="space-y-1">
                                        <Label
                                            for="wide-title"
                                            class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                                            >Title *</Label
                                        >
                                        <Input
                                            id="wide-title"
                                            type="text"
                                            placeholder="e.g. Google Account or Note Title"
                                            bind:value={newTitle}
                                            class="bg-zinc-900/50 border-zinc-800 text-white text-xs h-9.5 focus-visible:ring-1 focus-visible:ring-accent {titleError ? 'border-red-500/80 focus-visible:ring-red-500' : ''}"
                                        />
                                        {#if titleError}
                                            <p class="text-[10px] text-red-400 font-semibold">{titleError}</p>
                                        {/if}
                                    </div>

                                    {#if entryType === 'Login'}
                                        <!-- Login Specific Fields -->
                                        <div class="space-y-1">
                                            <Label
                                                for="wide-username"
                                                class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                                                >Username / Email</Label
                                            >
                                            <Input
                                                id="wide-username"
                                                type="text"
                                                placeholder="e.g. user@gmail.com"
                                                bind:value={newUsername}
                                                class="bg-zinc-900/50 border-zinc-800 text-white text-xs h-9.5 focus-visible:ring-1 focus-visible:ring-accent"
                                            />
                                        </div>

                                        <div class="space-y-1">
                                            <Label
                                                for="wide-password"
                                                class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                                                >Password</Label
                                            >
                                            <div class="flex gap-2">
                                                <div class="relative grow">
                                                    <Input
                                                        id="wide-password"
                                                        type={showNewPassword ? 'text' : 'password'}
                                                        placeholder="Enter password"
                                                        bind:value={newPassword}
                                                        class="bg-zinc-900/50 border-zinc-800 text-white pr-9 text-xs h-9.5 focus-visible:ring-1 focus-visible:ring-accent"
                                                    />
                                                    <Button
                                                        variant="ghost"
                                                        size="icon"
                                                        class="h-8 w-8 absolute right-1 top-1/2 -translate-y-1/2 text-zinc-400 hover:text-white"
                                                        onclick={() => (showNewPassword = !showNewPassword)}
                                                    >
                                                        {#if showNewPassword}
                                                            <EyeOff class="h-4 w-4" />
                                                        {:else}
                                                            <Eye class="h-4 w-4" />
                                                        {/if}
                                                    </Button>
                                                </div>
                                                <Button
                                                    variant="outline"
                                                    class="bg-zinc-900/50 border-zinc-800 text-zinc-300 hover:text-white h-9.5 w-9.5 shrink-0"
                                                    onclick={handleQuickGenerate}
                                                    title="Quick Generate"
                                                >
                                                    <Sparkles class="h-4 w-4 text-accent" />
                                                </Button>
                                            </div>
                                        </div>

                                        <div class="space-y-1">
                                            <Label
                                                for="wide-url"
                                                class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                                                >Website URL</Label
                                            >
                                            <Input
                                                id="wide-url"
                                                type="text"
                                                placeholder="e.g. google.com"
                                                bind:value={newUrl}
                                                class="bg-zinc-900/50 border-zinc-800 text-white text-xs h-9.5 focus-visible:ring-1 focus-visible:ring-accent {urlError ? 'border-red-500/80 focus-visible:ring-red-500' : ''}"
                                            />
                                            {#if urlError}
                                                <p class="text-[10px] text-red-400 font-semibold">{urlError}</p>
                                            {/if}
                                        </div>

                                        <div class="space-y-1">
                                            <Label
                                                for="wide-totp"
                                                class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                                                >TOTP Secret (2FA)</Label
                                            >
                                            <Input
                                                id="wide-totp"
                                                type="text"
                                                placeholder="e.g. JBSWY3DPEHPK3PXP"
                                                bind:value={newTotpSecret}
                                                class="bg-zinc-900/50 border-zinc-800 text-white text-xs h-9.5 focus-visible:ring-1 focus-visible:ring-accent {totpError ? 'border-red-500/80 focus-visible:ring-red-500' : ''}"
                                            />
                                            {#if totpError}
                                                <p class="text-[10px] text-red-400 font-semibold">{totpError}</p>
                                            {/if}
                                        </div>

                                        <div class="space-y-1">
                                            <Label
                                                for="wide-notes"
                                                class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                                                >Notes</Label
                                            >
                                            <textarea
                                                id="wide-notes"
                                                placeholder="Add custom notes..."
                                                bind:value={newNotes}
                                                class="w-full bg-zinc-900/30 border border-zinc-800 rounded-lg text-white p-3 text-xs outline-none focus:border-zinc-700 h-24 resize-none"
                                            ></textarea>
                                        </div>
                                    {:else}
                                        <!-- Secure Note Specific Fields -->
                                        <div class="space-y-1">
                                            <Label
                                                for="wide-note-body"
                                                class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                                                >Secure Note Body</Label
                                            >
                                            <textarea
                                                id="wide-note-body"
                                                placeholder="Type your secure note here..."
                                                bind:value={newNotes}
                                                class="w-full bg-zinc-900/30 border border-zinc-800 rounded-lg text-white p-3 text-xs outline-none focus:border-zinc-700 h-48 resize-none"
                                            ></textarea>
                                        </div>
                                    {/if}

                                    <!-- Common Tags Field -->
                                    <div class="space-y-1">
                                        <Label
                                            for="wide-tags"
                                            class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                                            >Tags (comma-separated)</Label
                                        >
                                        <Input
                                            id="wide-tags"
                                            type="text"
                                            placeholder="e.g. work, personal, financial"
                                            bind:value={newTagsString}
                                            class="bg-zinc-900/50 border-zinc-800 text-white text-xs h-9.5 focus-visible:ring-1 focus-visible:ring-accent"
                                        />
                                    </div>

                                    {#if addError}
                                        <div
                                            class="p-3 bg-red-950/30 border border-red-800/50 text-red-400 rounded-md text-xs"
                                        >
                                            {addError}
                                        </div>
                                    {/if}
                                </div>

                                <div class="flex gap-3 pt-2">
                                    <Button
                                        variant="outline"
                                        class="w-28 bg-zinc-900 border-zinc-800 text-zinc-300 hover:text-white text-xs h-9.5"
                                        onclick={() =>
                                            (widescreenPanel = "none")}
                                        >Cancel</Button
                                    >
                                    <Button
                                        class="w-28 bg-zinc-50 text-zinc-950 hover:bg-zinc-200 text-xs h-9.5 font-bold"
                                        onclick={handleAddEntry}
                                        >Save Entry</Button
                                    >
                                </div>
                            </div>
                        {:else if widescreenPanel === "detail" && selectedEntry}
                            <!-- Detail record widescreen view -->
                            <div
                                class="max-w-xl mx-auto w-full flex flex-col justify-between h-full"
                            >
                                <div class="space-y-6">
                                    <!-- Header badge Row -->
                                    <div
                                        class="flex items-center justify-between border-b border-zinc-900 pb-3"
                                    >
                                        <div class="flex items-center gap-3">
                                            <div
                                                class="h-10 w-10 rounded-full bg-zinc-800 border border-zinc-700 text-zinc-200 flex items-center justify-center font-bold uppercase text-sm"
                                            >
                                                {editWideEntryType === 'SecureNote' && isWidescreenEditing ? "📝" : (isSecureNoteWide ? "📝" : (displayTitleWide.trim()[0] || "🔑"))}
                                            </div>
                                            <div>
                                                <div class="flex items-center gap-2">
                                                    <h2
                                                        class="text-base font-bold text-white leading-tight"
                                                    >
                                                        {isWidescreenEditing ? 'Edit Entry' : displayTitleWide}
                                                    </h2>
                                                    {#if !isWidescreenEditing && isSecureNoteWide}
                                                        <span class="bg-zinc-800/80 border border-zinc-700 text-zinc-400 text-[8px] font-bold uppercase tracking-wider px-1.5 py-0.5 rounded select-none">
                                                            Note
                                                        </span>
                                                    {/if}
                                                </div>
                                                {#if !isWidescreenEditing && !isSecureNoteWide && displayUrlWide}
                                                    <a
                                                        class="text-xs text-accent hover:underline flex items-center gap-1"
                                                        href={displayUrlWide}
                                                        target="_blank"
                                                        rel="noreferrer"
                                                    >
                                                        {displayUrlWide.replace(
                                                            /^https?:\/\/(www\.)?/,
                                                            "",
                                                        )}
                                                        <ExternalLink
                                                            class="h-3 w-3"
                                                        />
                                                    </a>
                                                {/if}
                                            </div>
                                        </div>
                                        <div class="flex items-center gap-2">
                                            {#if !isWidescreenEditing}
                                                {#if selectedEntry.password_history && selectedEntry.password_history.length > 0}
                                                    <Button
                                                        variant="outline"
                                                        size="sm"
                                                        class="text-xs h-8 border-zinc-800 bg-zinc-900 text-zinc-300 hover:text-white"
                                                        onclick={() => (showWidescreenHistory = !showWidescreenHistory)}
                                                    >
                                                        {showWidescreenHistory ? 'Hide History' : 'History'}
                                                    </Button>
                                                {/if}
                                                <Button
                                                    variant="outline"
                                                    size="sm"
                                                    class="text-xs h-8 border-zinc-800 bg-zinc-900 text-zinc-300 hover:text-white"
                                                    onclick={startWidescreenEditing}
                                                >
                                                    Edit
                                                </Button>
                                            {/if}
                                            <Button
                                                variant="ghost"
                                                size="sm"
                                                class="text-xs text-zinc-400 h-8"
                                                onclick={() => {
                                                    isWidescreenEditing = false;
                                                    widescreenPanel = "none";
                                                }}
                                            >Close</Button
                                            >
                                        </div>
                                    </div>

                                    <!-- Read-only fields vs Edit inputs -->
                                    <div class="space-y-4">
                                        {#if !isWidescreenEditing}
                                            <!-- VIEW MODE -->
                                            {#if !isSecureNoteWide}
                                                <!-- Login Specific Fields -->
                                                <div class="space-y-1">
                                                    <Label
                                                        class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                                                        >Username / Email</Label
                                                    >
                                                    <div class="relative">
                                                        <Input
                                                            type="text"
                                                            readonly
                                                            value={selectedEntry.username ||
                                                                "(No Username)"}
                                                            class="bg-zinc-900/30 border-zinc-800 text-white pr-9 text-xs h-9.5"
                                                        />
                                                        {#if selectedEntry.username}
                                                            <div class="absolute right-1 top-1/2 -translate-y-1/2 flex items-center">
                                                                <Button
                                                                    variant="ghost"
                                                                    size="icon"
                                                                    class="h-8 w-8 text-zinc-400 hover:text-white"
                                                                    onclick={() =>
                                                                        triggerCopy(
                                                                            selectedEntry?.username,
                                                                            "u",
                                                                        )}
                                                                >
                                                                    {#if copiedKey === "u"}
                                                                        <Check
                                                                            class="h-4 w-4 text-green-400"
                                                                        />
                                                                    {:else}
                                                                        <Copy
                                                                            class="h-4 w-4"
                                                                        />
                                                                    {/if}
                                                                </Button>
                                                            </div>
                                                        {/if}
                                                    </div>
                                                </div>

                                                <div class="space-y-1">
                                                    <Label
                                                        class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                                                        >Password</Label
                                                    >
                                                    <div class="relative">
                                                        <Input
                                                            type={showDetailPassword
                                                                ? "text"
                                                                : "password"}
                                                            readonly
                                                            value={selectedEntry.password ||
                                                                ""}
                                                            class="bg-zinc-950 border border-zinc-800 text-white pr-16 text-xs h-9.5"
                                                        />
                                                        <div
                                                            class="absolute right-1 top-1/2 -translate-y-1/2 flex items-center gap-0.5"
                                                        >
                                                            <Button
                                                                variant="ghost"
                                                                size="icon"
                                                                class="h-8 w-8 text-zinc-400 hover:text-white"
                                                                onclick={async () => {
                                                                    if (!showDetailPassword) {
                                                                        const authorized = await reauthController.requestReauth();
                                                                        if (authorized) {
                                                                            showDetailPassword = true;
                                                                        }
                                                                    } else {
                                                                        showDetailPassword = false;
                                                                    }
                                                                }}
                                                            >
                                                                {#if showDetailPassword}
                                                                    <EyeOff
                                                                        class="h-4 w-4"
                                                                    />
                                                                {:else}
                                                                    <Eye
                                                                        class="h-4 w-4"
                                                                    />
                                                                {/if}
                                                            </Button>
                                                            {#if selectedEntry.password}
                                                                <Button
                                                                    variant="ghost"
                                                                    size="icon"
                                                                    class="h-8 w-8 text-zinc-400 hover:text-white"
                                                                    onclick={async () => {
                                                                        const authorized = await reauthController.requestReauth();
                                                                        if (authorized) {
                                                                            triggerCopy(
                                                                                selectedEntry?.password,
                                                                                "p",
                                                                            );
                                                                        }
                                                                    }}
                                                                >
                                                                    {#if copiedKey === "p"}
                                                                        <Check
                                                                            class="h-4 w-4 text-green-400"
                                                                        />
                                                                    {:else}
                                                                        <Copy
                                                                            class="h-4 w-4"
                                                                        />
                                                                    {/if}
                                                                </Button>
                                                            {/if}
                                                        </div>
                                                    </div>
                                                </div>

                                                {#if showWidescreenHistory && selectedEntry.password_history && selectedEntry.password_history.length > 0}
                                                    <div class="space-y-1 bg-zinc-950/40 p-3 border border-zinc-800 rounded-lg">
                                                        <Label class="text-[10px] font-bold text-zinc-500 uppercase tracking-wider">Password History</Label>
                                                        <div class="space-y-2.5 max-h-40 overflow-y-auto pr-1">
                                                            {#each selectedEntry.password_history.slice().reverse() as hist, index}
                                                                {@const histKey = `wide_hist_${index}`}
                                                                <div class="flex items-center justify-between gap-3 border-b border-zinc-900/50 pb-2 last:border-0 last:pb-0">
                                                                    <div class="flex flex-col min-w-0">
                                                                        <span class="text-xs font-mono text-zinc-300 truncate select-all">
                                                                            {visibleWidescreenHistoryKeys[histKey] ? hist.password : '•'.repeat(Math.max(1, hist.password.length))}
                                                                        </span>
                                                                        <span class="text-[9px] text-zinc-500">{new Date(hist.changed_at).toLocaleString()}</span>
                                                                    </div>
                                                                    <div class="flex items-center gap-0.5 shrink-0">
                                                                        <Button
                                                                            variant="ghost"
                                                                            size="icon"
                                                                            class="h-7 w-7 text-zinc-400 hover:text-white"
                                                                            onclick={async () => {
                                                                                if (!visibleWidescreenHistoryKeys[histKey]) {
                                                                                    const authorized = await reauthController.requestReauth();
                                                                                    if (authorized) {
                                                                                        visibleWidescreenHistoryKeys[histKey] = true;
                                                                                    }
                                                                                } else {
                                                                                    visibleWidescreenHistoryKeys[histKey] = false;
                                                                                }
                                                                            }}
                                                                            title={visibleWidescreenHistoryKeys[histKey] ? "Hide password" : "Show password"}
                                                                        >
                                                                            {#if visibleWidescreenHistoryKeys[histKey]}
                                                                                <EyeOff class="h-3.5 w-3.5" />
                                                                            {:else}
                                                                                <Eye class="h-3.5 w-3.5" />
                                                                            {/if}
                                                                        </Button>
                                                                        <Button
                                                                            variant="ghost"
                                                                            size="icon"
                                                                            class="h-7 w-7 text-zinc-400 hover:text-white"
                                                                            onclick={async () => {
                                                                                const authorized = await reauthController.requestReauth();
                                                                                if (authorized) {
                                                                                    triggerCopy(hist.password, histKey);
                                                                                }
                                                                            }}
                                                                        >
                                                                            {#if copiedKey === histKey}
                                                                                <Check class="h-3.5 w-3.5 text-green-400" />
                                                                            {:else}
                                                                                <Copy class="h-3.5 w-3.5" />
                                                                            {/if}
                                                                        </Button>
                                                                    </div>
                                                                </div>
                                                            {/each}
                                                        </div>
                                                    </div>
                                                {/if}

                                                {#if selectedEntry.totp_secret}
                                                    <!-- 6-digit TOTP Code Display -->
                                                    <div class="space-y-1">
                                                        <Label class="text-xs font-semibold text-zinc-400 uppercase tracking-wider block">One-Time Password (TOTP)</Label>
                                                        <div class="relative flex items-center">
                                                            <div class="flex-1 bg-zinc-900/30 border border-zinc-800 rounded-lg px-3 py-1.5 flex items-center justify-between min-w-0 h-9.5 pr-10">
                                                                {#if totpCodes[selectedEntry.id]}
                                                                    <span class="font-mono text-sm font-bold tracking-[0.2em] text-[#06b6d4] select-all leading-none">
                                                                        {totpCodes[selectedEntry.id].slice(0, 3)} {totpCodes[selectedEntry.id].slice(3)}
                                                                    </span>
                                                                {:else}
                                                                    <span class="text-xs text-zinc-500 italic animate-pulse">Generating...</span>
                                                                {/if}

                                                                <!-- Visual countdown bar/indicator -->
                                                                <div class="flex items-center gap-1.5 shrink-0">
                                                                    <div class="w-12 h-1.5 bg-zinc-800 rounded-full overflow-hidden">
                                                                        <div
                                                                            class="h-full bg-[#06b6d4] transition-all duration-1000 ease-linear"
                                                                            style="width: {(remainingSeconds / 30) * 100}%"
                                                                        ></div>
                                                                    </div>
                                                                    <span class="text-[10px] font-mono text-zinc-400 w-4 text-right leading-none">{remainingSeconds}s</span>
                                                                </div>
                                                            </div>
                                                            <Button
                                                                variant="ghost"
                                                                size="icon"
                                                                class="h-8 w-8 text-zinc-400 hover:text-white shrink-0 absolute right-1"
                                                                onclick={() => triggerCopy(totpCodes[selectedEntry.id], 'selected_totp_code')}
                                                                title="Copy TOTP Code"
                                                                disabled={!totpCodes[selectedEntry.id]}
                                                            >
                                                                {#if copiedKey === 'selected_totp_code'}
                                                                    <Check class="h-4 w-4 text-green-400" />
                                                                {:else}
                                                                    <Copy class="h-4 w-4" />
                                                                {/if}
                                                            </Button>
                                                        </div>
                                                    </div>

                                                    <div class="space-y-1">
                                                        <Label
                                                            class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                                                            >TOTP Secret (2FA)</Label
                                                        >
                                                        <div class="relative">
                                                            <Input
                                                                type="text"
                                                                readonly
                                                                value={selectedEntry.totp_secret}
                                                                class="bg-zinc-900/30 border-zinc-800 text-white pr-9 text-xs h-9.5"
                                                            />
                                                            <Button
                                                                variant="ghost"
                                                                size="icon"
                                                                class="h-8 w-8 absolute right-1 top-1/2 -translate-y-1/2 text-zinc-400 hover:text-white"
                                                                onclick={() =>
                                                                    triggerCopy(
                                                                        selectedEntry?.totp_secret || '',
                                                                        "totp",
                                                                    )}
                                                            >
                                                                {#if copiedKey === "totp"}
                                                                    <Check
                                                                        class="h-4 w-4 text-green-400"
                                                                    />
                                                                {:else}
                                                                    <Copy
                                                                        class="h-4 w-4"
                                                                    />
                                                                {/if}
                                                            </Button>
                                                        </div>
                                                    </div>
                                                {/if}

                                                {#if selectedEntry.notes}
                                                    <div class="space-y-1">
                                                        <Label
                                                            class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                                                            >Notes</Label
                                                        >
                                                        <div
                                                            class="w-full bg-zinc-900/20 border border-zinc-800 rounded-lg text-zinc-300 p-3 text-xs h-24 overflow-y-auto whitespace-pre-wrap select-all"
                                                        >
                                                            {selectedEntry.notes}
                                                        </div>
                                                    </div>
                                                {/if}
                                            {:else}
                                                <!-- Secure Note Specific Display -->
                                                <div class="space-y-1">
                                                    <Label
                                                        class="text-xs font-semibold text-zinc-400 uppercase tracking-wider"
                                                        >Secure Note Body</Label
                                                    >
                                                    <div
                                                        class="w-full bg-zinc-900/20 border border-zinc-800 rounded-lg text-zinc-200 p-3 text-xs h-64 overflow-y-auto whitespace-pre-wrap select-all"
                                                    >
                                                        {selectedEntry.notes || "(Empty Note)"}
                                                    </div>
                                                </div>
                                            {/if}

                                            <!-- Tags Display -->
                                            {#if displayTagsWide && displayTagsWide.length > 0}
                                                <div class="space-y-1 pt-1">
                                                    <Label class="text-xs font-semibold text-zinc-400 uppercase tracking-wider">Tags</Label>
                                                    <div class="flex flex-wrap gap-1.5 pt-0.5">
                                                        {#each displayTagsWide as tag}
                                                            <span class="bg-zinc-900 text-zinc-300 border border-zinc-800 px-2.5 py-0.5 rounded-full text-[10px] font-semibold">
                                                                {tag}
                                                            </span>
                                                        {/each}
                                                    </div>
                                                </div>
                                            {/if}
                                        {:else}
                                            <!-- Title field -->
                                            <div class="space-y-1">
                                                <Label for="edit-wide-title" class="text-xs font-semibold text-zinc-400 uppercase tracking-wider">Title *</Label>
                                                <Input
                                                    id="edit-wide-title"
                                                    type="text"
                                                    placeholder="Title"
                                                    bind:value={editWideTitle}
                                                    class="bg-zinc-900/50 border-zinc-800 text-white text-xs h-9.5 focus-visible:ring-1 focus-visible:ring-accent {editWideTitleError ? 'border-red-500/80 focus-visible:ring-red-500' : ''}"
                                                />
                                                {#if editWideTitleError}
                                                    <p class="text-[10px] text-red-400 font-semibold">{editWideTitleError}</p>
                                                {/if}
                                            </div>

                                            {#if editWideEntryType === 'Login'}
                                                <!-- Login specific edit fields -->
                                                <div class="space-y-1">
                                                    <Label for="edit-wide-username" class="text-xs font-semibold text-zinc-400 uppercase tracking-wider">Username</Label>
                                                    <Input
                                                        id="edit-wide-username"
                                                        type="text"
                                                        placeholder="Username"
                                                        bind:value={editWideUsername}
                                                        class="bg-zinc-900/50 border-zinc-800 text-white text-xs h-9.5 focus-visible:ring-1 focus-visible:ring-accent"
                                                    />
                                                </div>

                                                <div class="space-y-1">
                                                    <Label for="edit-wide-password" class="text-xs font-semibold text-zinc-400 uppercase tracking-wider">Password</Label>
                                                    <div class="flex gap-2">
                                                        <div class="relative grow">
                                                            <Input
                                                                id="edit-wide-password"
                                                                type={editWideShowPassword ? 'text' : 'password'}
                                                                placeholder="Password"
                                                                bind:value={editWidePassword}
                                                                class="bg-zinc-900/50 border-zinc-800 text-white pr-9 text-xs h-9.5 focus-visible:ring-1 focus-visible:ring-accent"
                                                            />
                                                            <Button
                                                                variant="ghost"
                                                                size="icon"
                                                                class="h-8 w-8 absolute right-1 top-1/2 -translate-y-1/2 text-zinc-400 hover:text-white"
                                                                onclick={() => (editWideShowPassword = !editWideShowPassword)}
                                                            >
                                                                {#if editWideShowPassword}
                                                                    <EyeOff class="h-4 w-4" />
                                                                {:else}
                                                                    <Eye class="h-4 w-4" />
                                                                {/if}
                                                            </Button>
                                                        </div>
                                                        <Button
                                                            variant="outline"
                                                            class="bg-zinc-900/50 border-zinc-800 text-zinc-300 hover:text-white h-9.5 w-9.5 shrink-0"
                                                            onclick={handleQuickGenerateEditWide}
                                                            title="Quick Generate Password"
                                                        >
                                                            <Sparkles class="h-4 w-4 text-accent" />
                                                        </Button>
                                                    </div>
                                                </div>

                                                <div class="space-y-1">
                                                    <Label for="edit-wide-url" class="text-xs font-semibold text-zinc-400 uppercase tracking-wider">Website URL</Label>
                                                    <Input
                                                        id="edit-wide-url"
                                                        type="text"
                                                        placeholder="Website URL"
                                                        bind:value={editWideUrl}
                                                        class="bg-zinc-900/50 border-zinc-800 text-white text-xs h-9.5 focus-visible:ring-1 focus-visible:ring-accent {editWideUrlError ? 'border-red-500/80 focus-visible:ring-red-500' : ''}"
                                                    />
                                                    {#if editWideUrlError}
                                                        <p class="text-[10px] text-red-400 font-semibold">{editWideUrlError}</p>
                                                    {/if}
                                                </div>

                                                <div class="space-y-1">
                                                    <Label for="edit-wide-totp" class="text-xs font-semibold text-zinc-400 uppercase tracking-wider">TOTP Secret</Label>
                                                    <Input
                                                        id="edit-wide-totp"
                                                        type="text"
                                                        placeholder="Base32 key"
                                                        bind:value={editWideTotpSecret}
                                                        class="bg-zinc-900/50 border-zinc-800 text-white text-xs h-9.5 focus-visible:ring-1 focus-visible:ring-accent {editWideTotpError ? 'border-red-500/80 focus-visible:ring-red-500' : ''}"
                                                    />
                                                    {#if editWideTotpError}
                                                        <p class="text-[10px] text-red-400 font-semibold">{editWideTotpError}</p>
                                                    {/if}
                                                </div>

                                                <div class="space-y-1">
                                                    <Label for="edit-wide-notes" class="text-xs font-semibold text-zinc-400 uppercase tracking-wider">Notes</Label>
                                                    <textarea
                                                        id="edit-wide-notes"
                                                        placeholder="Notes..."
                                                        bind:value={editWideNotes}
                                                        class="w-full bg-zinc-900/30 border border-zinc-800 rounded-lg text-white p-3 text-xs outline-none focus:border-zinc-700 h-24 resize-none"
                                                    ></textarea>
                                                </div>
                                            {:else}
                                                <!-- Secure note edit fields -->
                                                <div class="space-y-1">
                                                    <Label for="edit-wide-note-body" class="text-xs font-semibold text-zinc-400 uppercase tracking-wider">Secure Note Body</Label>
                                                    <textarea
                                                        id="edit-wide-note-body"
                                                        placeholder="Type note content here..."
                                                        bind:value={editWideNotes}
                                                        class="w-full bg-zinc-900/30 border border-zinc-800 rounded-lg text-white p-3 text-xs outline-none focus:border-zinc-700 h-64 resize-none"
                                                    ></textarea>
                                                </div>
                                            {/if}

                                            <!-- Tags input fields -->
                                            <div class="space-y-1">
                                                <Label for="edit-wide-tags" class="text-xs font-semibold text-zinc-400 uppercase tracking-wider">Tags (comma-separated)</Label>
                                                <Input
                                                    id="edit-wide-tags"
                                                    type="text"
                                                    placeholder="Tags"
                                                    bind:value={editWideTagsString}
                                                    class="bg-zinc-900/50 border-zinc-800 text-white text-xs h-9.5 focus-visible:ring-1 focus-visible:ring-accent"
                                                />
                                            </div>

                                            {#if editWideGlobalError}
                                                <div class="p-3 bg-red-950/30 border border-red-800/50 text-red-400 rounded-md text-xs">
                                                    {editWideGlobalError}
                                                </div>
                                            {/if}
                                        {/if}
                                    </div>
                                </div>

                                <div class="border-t border-zinc-900 pt-4 mt-6">
                                    {#if !isWidescreenEditing}
                                        <Button
                                            class="w-full text-xs h-9.5 font-bold bg-red-950/40 border border-red-800/60 text-red-200 hover:bg-red-900/50 hover:text-white flex items-center justify-center gap-1.5 transition-colors"
                                            onclick={() =>
                                                selectedEntry &&
                                                handleDeleteEntry(selectedEntry.id)}
                                        >
                                            <Trash2 class="h-4 w-4" /> Delete Credential
                                        </Button>
                                    {:else}
                                        <div class="flex gap-3">
                                            <Button
                                                variant="outline"
                                                class="w-1/2 bg-zinc-900 border-zinc-800 text-zinc-300 hover:text-white text-xs h-9.5"
                                                onclick={cancelWidescreenEdit}
                                            >
                                                Cancel
                                            </Button>
                                            <Button
                                                class="w-1/2 bg-zinc-50 text-zinc-950 hover:bg-zinc-200 text-xs h-9.5 font-bold"
                                                onclick={handleSaveWidescreenEdit}
                                            >
                                                Save Changes
                                            </Button>
                                        </div>
                                    {/if}
                                </div>
                            </div>
                        {/if}
                    </div>
                </div>
            {:else}
                <!-- Sync & Settings tabs fullscreen layout -->
                <div class="grow overflow-y-auto p-8 space-y-6">
                    {#if activeTab === "sync"}
                        <div class="max-w-xl mx-auto space-y-4">
                            <div>
                                <h2 class="text-lg font-bold text-white mb-1">
                                    Google Drive Synchronization
                                </h2>
                                <p class="text-xs text-zinc-400">
                                    Link your Google Account to back up and sync
                                    your credentials across all devices.
                                </p>
                            </div>
                            <div class="space-y-4 pt-2">
                                <p class="text-sm text-zinc-300 leading-relaxed">
                                    Synchronization will copy your encrypted
                                    vault database securely to your own private
                                    Google Drive account inside the isolated
                                    appDataFolder space.
                                </p>

                                {#if !googleUser}
                                    <div
                                        class="p-6 bg-zinc-950/40 border border-zinc-900 rounded-lg flex flex-col items-center gap-4 py-8 text-center"
                                    >
                                        <div class="p-3 rounded-full bg-zinc-900/50 border border-zinc-800 text-zinc-500">
                                            <CloudOff class="h-6 w-6" />
                                        </div>
                                        <p class="text-xs text-zinc-400">
                                            Google Drive is not connected.
                                        </p>
                                        <button
                                             class="px-4 py-2 bg-zinc-50 text-zinc-950 hover:bg-zinc-200 disabled:opacity-50 rounded-lg text-xs font-semibold flex items-center gap-1.5 transition-colors focus-visible:ring-1 focus-visible:ring-zinc-700 cursor-pointer"
                                             onclick={handleSignIn}
                                             disabled={isConnecting}
                                         >
                                             {#if isConnecting}
                                                 <LoaderCircle class="h-3.5 w-3.5 animate-spin" />
                                                 Connecting...
                                             {:else}
                                                 Connect Google Drive Account
                                             {/if}
                                         </button>
                                    </div>
                                {:else}
                                    <div
                                        class="p-5 bg-zinc-950 border border-zinc-900 rounded-lg space-y-4"
                                    >
                                        <div
                                            class="flex justify-between items-start"
                                        >
                                            <div class="flex items-center gap-3">
                                                <div class="p-2.5 rounded-lg bg-zinc-900 border border-zinc-800">
                                                    <Cloud class="h-5 w-5 text-cyan-400" />
                                                </div>
                                                <div>
                                                    <p
                                                        class="text-[10px] text-zinc-400 uppercase tracking-wider font-semibold"
                                                    >
                                                        Connected Profile
                                                    </p>
                                                    <p
                                                        class="text-sm font-semibold text-white mt-0.5"
                                                    >
                                                        {googleUser.name ||
                                                            "Google User"}
                                                    </p>
                                                    <p
                                                        class="text-xs text-zinc-400 mt-0.5"
                                                    >
                                                        {googleUser.email}
                                                    </p>
                                                </div>
                                            </div>
                                            <span
                                                class="text-[11px] px-2.5 py-0.5 bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 rounded-full font-medium"
                                            >
                                                Active Sync
                                            </span>
                                        </div>

                                        <!-- COMPARISON GRID -->
                                        <div class="grid grid-cols-2 gap-3 pt-2">
                                            <!-- Local Vault Card -->
                                            <div class="p-3 bg-zinc-900/30 border border-zinc-850 rounded-lg space-y-2">
                                                <h4 class="text-[10px] font-bold text-zinc-400 uppercase tracking-wider">Local Database</h4>
                                                <div class="space-y-1.5 text-xs">
                                                    <div class="flex justify-between text-zinc-400">
                                                        <span>Entries:</span>
                                                        <span class="text-white font-medium">{localFileInfo.entryCount} ({localFileInfo.trashCount} in trash)</span>
                                                    </div>
                                                    <div class="text-[10px] text-zinc-500 pt-1 leading-normal">
                                                        Active database inside this browser profile.
                                                    </div>
                                                </div>
                                            </div>

                                            <!-- Remote Vault Card -->
                                            <div class="p-3 bg-zinc-900/30 border border-zinc-850 rounded-lg space-y-2">
                                                <h4 class="text-[10px] font-bold text-zinc-400 uppercase tracking-wider">Google Drive Backup</h4>
                                                {#if remoteFileInfo}
                                                    <div class="space-y-1.5 text-xs">
                                                        <div class="flex justify-between text-zinc-400">
                                                            <span>Size:</span>
                                                            <span class="text-white font-medium">{(remoteFileInfo.sizeBytes ? remoteFileInfo.sizeBytes / 1024 : 0).toFixed(1)} KB</span>
                                                        </div>
                                                        <div class="text-[10px] text-zinc-500 pt-1 truncate" title={new Date(remoteFileInfo.lastModified).toLocaleString()}>
                                                            Modified: {new Date(remoteFileInfo.lastModified).toLocaleDateString()}
                                                        </div>
                                                    </div>
                                                {:else}
                                                    <div class="flex flex-col items-center justify-center py-3 text-zinc-500 text-[10px] gap-1">
                                                        <div class="animate-spin rounded-full h-3 w-3 border-2 border-zinc-700 border-t-zinc-400"></div>
                                                        <span>Fetching backup details...</span>
                                                    </div>
                                                {/if}
                                            </div>
                                        </div>

                                        <!-- ACTION BUTTONS -->
                                        <div class="space-y-2 pt-2">
                                            <div class="flex gap-2">
                                                <button
                                                    class="flex-1 px-3.5 py-2 bg-zinc-50 hover:bg-zinc-200 text-zinc-950 disabled:opacity-50 rounded-lg text-xs font-semibold flex items-center justify-center gap-1.5 transition-colors focus-visible:ring-1 focus-visible:ring-zinc-700 cursor-pointer"
                                                    onclick={handleSyncNow}
                                                    disabled={isSyncing || isRestoring || isBackingUp}
                                                >
                                                    <RefreshCw class="h-3.5 w-3.5 {isSyncing ? 'animate-spin' : ''}" />
                                                    {isSyncing ? "Syncing..." : "Sync & Merge"}
                                                </button>
                                            </div>
                                            <div class="flex gap-2">
                                                <button
                                                    class="w-1/2 px-3.5 py-2 bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 disabled:opacity-50 rounded-lg text-xs font-semibold text-zinc-200 transition-all flex items-center justify-center gap-1.5 cursor-pointer"
                                                    onclick={() => showRestoreConfirm = true}
                                                    disabled={isSyncing || isRestoring || isBackingUp}
                                                >
                                                    Restore from Cloud (Overwrite Local)
                                                </button>
                                                <button
                                                    class="w-1/2 px-3.5 py-2 bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 disabled:opacity-50 rounded-lg text-xs font-semibold text-zinc-200 transition-all flex items-center justify-center gap-1.5 cursor-pointer"
                                                    onclick={() => showBackupConfirm = true}
                                                    disabled={isSyncing || isRestoring || isBackingUp}
                                                >
                                                    Backup to Cloud (Overwrite Cloud)
                                                </button>
                                            </div>
                                        </div>

                                        <!-- DISCONNECT BUTTONS -->
                                        <div
                                            class="flex gap-2 pt-4 border-t border-zinc-900"
                                        >
                                            <button
                                                class="w-1/2 px-3.5 py-2 bg-zinc-900 hover:bg-red-950/20 border border-zinc-800 hover:border-red-900/30 text-zinc-350 hover:text-red-400 rounded-lg text-xs font-semibold transition-all flex items-center justify-center gap-1.5 cursor-pointer"
                                                onclick={handleSignOut}
                                                disabled={isSyncing || isRestoring || isBackingUp}
                                            >
                                                <LogOut class="h-3.5 w-3.5" />
                                                Disconnect Sync
                                            </button>
                                            <button
                                                class="w-1/2 px-3.5 py-2 bg-zinc-900 hover:bg-red-950/20 border border-zinc-800 hover:border-red-900/30 text-zinc-350 hover:text-red-450 rounded-lg text-xs font-semibold transition-all flex items-center justify-center gap-1.5 cursor-pointer"
                                                onclick={() => showDeleteCloudConfirm = true}
                                                disabled={isSyncing || isRestoring || isBackingUp || isDeletingCloud}
                                            >
                                                Disconnect & Wipe Cloud Data
                                            </button>
                                        </div>
                                    </div>
                                {/if}

                                {#if vaultState.syncNeedsPassword}
                                    <div class="p-5 bg-zinc-950 border border-amber-500/20 rounded-lg space-y-4">
                                        <div class="flex items-center gap-2 text-amber-400">
                                            <Lock class="h-4 w-4" />
                                            <h3 class="text-xs font-semibold uppercase tracking-wider">Remote Vault Auth Required</h3>
                                        </div>
                                        <p class="text-xs text-zinc-400 leading-relaxed">
                                            The remote vault found on Google Drive was encrypted with a different key/salt. Please enter the master password for the remote vault to decrypt it.
                                        </p>
                                        <form onsubmit={(e) => e.preventDefault()} class="space-y-3">
                                            <div class="relative">
                                                <input
                                                    type={showSyncPassword ? "text" : "password"}
                                                    bind:value={syncPassword}
                                                    placeholder="Remote Master Password"
                                                    class="w-full bg-zinc-900 border border-zinc-800 text-white text-xs px-3 py-2 pr-10 rounded-lg focus:outline-none focus:border-amber-600 focus:ring-1 focus:ring-amber-500/30 transition-all"
                                                    required
                                                />
                                                <button
                                                    type="button"
                                                    class="absolute right-3 top-1/2 -translate-y-1/2 text-zinc-450 hover:text-white transition-colors cursor-pointer"
                                                    onclick={() => showSyncPassword = !showSyncPassword}
                                                >
                                                    {#if showSyncPassword}
                                                        <EyeOff class="h-3.5 w-3.5" />
                                                    {:else}
                                                        <Eye class="h-3.5 w-3.5" />
                                                    {/if}
                                                </button>
                                            </div>
                                            {#if syncMismatchError}
                                                <p class="text-[11px] text-red-400 flex items-center gap-1">
                                                    <span>⚠</span> {syncMismatchError}
                                                </p>
                                            {/if}
                                            <div class="flex gap-2 pt-1">
                                                <button
                                                    type="button"
                                                    class="px-3.5 py-1.5 bg-zinc-900 hover:bg-zinc-800 text-zinc-300 rounded-lg text-xs font-semibold transition-colors border border-zinc-800 cursor-pointer"
                                                    onclick={handleCancelSyncMismatch}
                                                    disabled={isSyncing}
                                                >
                                                    Cancel
                                                </button>
                                                <button
                                                    type="button"
                                                    class="px-3 py-1.5 bg-zinc-850 hover:bg-zinc-800 border border-zinc-800 text-zinc-200 rounded-lg text-xs font-semibold transition-colors cursor-pointer"
                                                    onclick={() => handleResolveSyncMismatch('merge')}
                                                    disabled={isSyncing}
                                                >
                                                    Verify & Merge
                                                </button>
                                                <button
                                                    type="button"
                                                    class="px-3 py-1.5 bg-zinc-50 text-zinc-950 hover:bg-zinc-200 rounded-lg text-xs font-semibold transition-colors cursor-pointer"
                                                    onclick={() => handleResolveSyncMismatch('restore')}
                                                    disabled={isSyncing}
                                                >
                                                    Verify & Overwrite Local
                                                </button>
                                            </div>
                                        </form>
                                    </div>
                                {/if}

                                <!-- CONFIRMATION MODALS -->
                                {#if showRestoreConfirm}
                                    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-xs p-4">
                                        <div class="w-full max-w-sm bg-zinc-950 border border-zinc-850 rounded-xl p-5 shadow-2xl space-y-4">
                                            <div class="flex items-center gap-2 text-amber-500">
                                                <Lock class="h-4.5 w-4.5" />
                                                <h3 class="text-sm font-semibold tracking-tight">Confirm Restore from Cloud</h3>
                                            </div>
                                            <p class="text-xs text-zinc-400 leading-normal">
                                                This action will replace your local database with the encrypted backup from Google Drive. Any local updates since the last sync will be permanently lost.
                                            </p>
                                            <div class="flex gap-2 pt-2">
                                                <button
                                                    type="button"
                                                    class="w-1/2 bg-zinc-900 border border-zinc-800 text-zinc-300 hover:text-white text-xs h-9.5 rounded-lg font-semibold cursor-pointer"
                                                    onclick={() => showRestoreConfirm = false}
                                                    disabled={isRestoring}
                                                >
                                                    Cancel
                                                </button>
                                                <button
                                                    type="button"
                                                    class="w-1/2 bg-zinc-50 text-zinc-950 hover:bg-zinc-200 text-xs h-9.5 rounded-lg font-bold flex items-center justify-center cursor-pointer"
                                                    onclick={handleRestoreRemote}
                                                    disabled={isRestoring}
                                                >
                                                    {#if isRestoring}
                                                        <div class="animate-spin rounded-full h-3.5 w-3.5 border-2 border-zinc-950 border-t-transparent"></div>
                                                    {:else}
                                                        Confirm Restore
                                                    {/if}
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                {/if}

                                {#if showBackupConfirm}
                                    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-xs p-4">
                                        <div class="w-full max-w-sm bg-zinc-950 border border-zinc-850 rounded-xl p-5 shadow-2xl space-y-4">
                                            <div class="flex items-center gap-2 text-amber-500">
                                                <Lock class="h-4.5 w-4.5" />
                                                <h3 class="text-sm font-semibold tracking-tight">Confirm Overwrite Cloud Backup</h3>
                                            </div>
                                            <p class="text-xs text-zinc-400 leading-normal">
                                                This will completely replace the current backup file on Google Drive with your local database. Any remote credentials that haven't been downloaded yet will be overwritten.
                                            </p>
                                            <div class="flex gap-2 pt-2">
                                                <button
                                                    type="button"
                                                    class="w-1/2 bg-zinc-900 border border-zinc-800 text-zinc-300 hover:text-white text-xs h-9.5 rounded-lg font-semibold cursor-pointer"
                                                    onclick={() => showBackupConfirm = false}
                                                    disabled={isBackingUp}
                                                >
                                                    Cancel
                                                </button>
                                                <button
                                                    type="button"
                                                    class="w-1/2 bg-zinc-50 text-zinc-950 hover:bg-zinc-200 text-xs h-9.5 rounded-lg font-bold flex items-center justify-center cursor-pointer"
                                                    onclick={handleBackupLocal}
                                                    disabled={isBackingUp}
                                                >
                                                    {#if isBackingUp}
                                                        <div class="animate-spin rounded-full h-3.5 w-3.5 border-2 border-zinc-950 border-t-transparent"></div>
                                                    {:else}
                                                        Confirm Backup
                                                    {/if}
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                {/if}

                                {#if showDeleteCloudConfirm}
                                    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-xs p-4">
                                        <div class="w-full max-w-sm bg-zinc-950 border border-zinc-850 rounded-xl p-5 shadow-2xl space-y-4">
                                            <div class="flex items-center gap-2 text-red-500">
                                                <Lock class="h-4.5 w-4.5" />
                                                <h3 class="text-sm font-semibold tracking-tight">Confirm Wipe Cloud Data</h3>
                                            </div>
                                            <p class="text-xs text-zinc-400 leading-normal">
                                                This action will permanently delete the <code>vault.db</code> file from your Google Drive and disconnect sync. This cannot be undone.
                                            </p>
                                            <div class="flex gap-2 pt-2">
                                                <button
                                                    type="button"
                                                    class="w-1/2 bg-zinc-900 border border-zinc-800 text-zinc-300 hover:text-white text-xs h-9.5 rounded-lg font-semibold cursor-pointer"
                                                    onclick={() => showDeleteCloudConfirm = false}
                                                    disabled={isDeletingCloud}
                                                >
                                                    Cancel
                                                </button>
                                                <button
                                                    type="button"
                                                    class="w-1/2 bg-red-600 hover:bg-red-500 text-white text-xs h-9.5 rounded-lg font-bold flex items-center justify-center cursor-pointer"
                                                    onclick={handleSignOutAndDelete}
                                                    disabled={isDeletingCloud}
                                                >
                                                    {#if isDeletingCloud}
                                                        <div class="animate-spin rounded-full h-3.5 w-3.5 border-2 border-white border-t-transparent"></div>
                                                    {:else}
                                                        Wipe & Disconnect
                                                    {/if}
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                {/if}

                                {#if syncSuccess}
                                    <div
                                        class="p-3 bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 rounded-lg text-xs flex items-center gap-2"
                                    >
                                        <Check class="h-4 w-4 shrink-0 text-emerald-400" />
                                        <span>Vault successfully synchronized and merged!</span>
                                    </div>
                                {/if}

                                {#if syncError}
                                    <div
                                        class="p-3 bg-red-500/10 border border-red-500/20 text-red-400 rounded-lg text-xs flex items-center gap-2"
                                    >
                                        <CloudOff class="h-4 w-4 shrink-0 text-red-400" />
                                        <span>{syncError}</span>
                                    </div>
                                {/if}
                            </div>
                        </div>
                    {:else if activeTab === "generator"}
                        <div class="max-w-xl mx-auto py-4">
                            <PasswordGenerator />
                        </div>
                    {:else if activeTab === "trash"}
                        <!-- Split Trash Layout -->
                        <div class="grow flex overflow-hidden">
                            <!-- Split pane left: Deleted Items List -->
                            <div
                                class="w-80 border-r border-zinc-900 flex flex-col justify-between shrink-0 bg-zinc-950/20"
                            >
                                <div class="p-4 border-b border-zinc-900 flex justify-between items-center shrink-0">
                                    <span class="text-zinc-400 font-semibold uppercase tracking-wider text-xs">
                                        {vaultState.vault.trash?.length || 0} DELETED ITEMS
                                    </span>
                                    {#if vaultState.vault.trash && vaultState.vault.trash.length > 0}
                                        <Button
                                            class="bg-red-950/30 border border-red-900/50 text-red-200 hover:bg-red-900/50 hover:text-white h-7 px-2 text-[10px] uppercase font-bold flex items-center gap-1 transition-colors"
                                            onclick={handleEmptyTrash}
                                        >
                                            Empty Trash
                                        </Button>
                                    {/if}
                                </div>

                                <div class="grow overflow-y-auto p-2.5 space-y-1.5">
                                    {#if !vaultState.vault.trash || vaultState.vault.trash.length === 0}
                                        <div class="flex flex-col items-center justify-center py-20 text-zinc-600 gap-2 select-none">
                                            <span class="text-3xl">🗑️</span>
                                            <p class="text-xs font-semibold">Trash is empty</p>
                                        </div>
                                    {:else}
                                        {#each vaultState.vault.trash as entry (entry.id)}
                                            <button
                                                class="w-full flex items-center gap-3 p-2.5 rounded-lg border border-transparent text-left transition-all duration-250 select-none min-w-0 {selectedTrashEntry?.id === entry.id ? 'bg-zinc-900 border-zinc-800' : 'hover:bg-zinc-900/50'}"
                                                onclick={() => {
                                                    selectedTrashEntry = entry;
                                                }}
                                            >
                                                <div class="h-8 w-8 rounded-full bg-zinc-800 border border-zinc-700 text-zinc-400 flex items-center justify-center font-bold uppercase text-xs shrink-0">
                                                    {entry.type === 'SecureNote' ? '📝' : (entry.title.trim()[0] || "🔑")}
                                                </div>
                                                <div class="flex flex-col min-w-0">
                                                    <div class="flex items-center gap-1.5 min-w-0">
                                                        <span class="text-xs font-semibold text-zinc-400 truncate">{entry.title}</span>
                                                        {#if entry.type === 'SecureNote'}
                                                            <span class="bg-amber-500/10 text-amber-500 border border-amber-500/10 text-[8px] font-bold uppercase tracking-wider px-1 py-0.2 rounded select-none shrink-0 opacity-70">
                                                                Note
                                                            </span>
                                                        {:else}
                                                            <span class="bg-cyan-500/10 text-cyan-500 border border-cyan-500/10 text-[8px] font-bold uppercase tracking-wider px-1 py-0.2 rounded select-none shrink-0 opacity-70">
                                                                Login
                                                            </span>
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

                            <!-- Split pane right: Deleted Item Details & Operations -->
                            <div class="grow overflow-y-auto p-8 flex flex-col justify-between">
                                {#if !selectedTrashEntry}
                                    <div class="m-auto flex flex-col items-center text-center max-w-sm space-y-4 py-20 select-none">
                                        <div class="h-12 w-12 rounded-full bg-zinc-900 border border-zinc-800 flex items-center justify-center text-zinc-500">
                                            <Trash2 class="h-6 w-6" />
                                        </div>
                                        <div class="space-y-1">
                                            <h3 class="text-sm font-semibold text-white">No Deleted Item Selected</h3>
                                            <p class="text-xs text-zinc-500 leading-relaxed">
                                                Select a soft-deleted item from the sidebar to view details, restore it, or permanently destroy it.
                                            </p>
                                        </div>
                                    </div>
                                {:else}
                                    <div class="max-w-xl mx-auto w-full space-y-6">
                                        <!-- Warning Banner -->
                                        <div class="p-3 bg-red-950/20 border border-red-800/40 text-red-300 rounded-lg text-xs flex items-center gap-2">
                                            <Trash2 class="h-4 w-4 text-red-400" />
                                            <span>This item is in the Trash and will not auto-fill.</span>
                                        </div>

                                        <!-- Header Title -->
                                        <div>
                                            <h2 class="text-lg font-bold text-white tracking-tight">{selectedTrashEntry.title}</h2>
                                            <p class="text-xs text-zinc-500 mt-1 font-mono">ID: {selectedTrashEntry.id}</p>
                                        </div>

                                        <!-- Static details fields -->
                                        <div class="space-y-4">
                                            {#if selectedTrashEntry.type === 'DomainGroup'}
                                                {#each selectedTrashEntry.accounts || [] as account (account.id)}
                                                    <div class="p-3 bg-zinc-900/40 border border-zinc-800 rounded-lg space-y-2">
                                                        <div class="flex justify-between items-center text-xs">
                                                            <span class="text-zinc-400 font-medium">Username: {account.username || "(No Username)"}</span>
                                                        </div>
                                                        <div class="flex justify-between items-center text-xs">
                                                            <span class="text-zinc-400 font-medium">Password: ••••••••</span>
                                                        </div>
                                                        {#if account.notes}
                                                            <p class="text-[10px] text-zinc-500 font-sans italic">{account.notes}</p>
                                                        {/if}
                                                    </div>
                                                {/each}
                                                {#if selectedTrashEntry.urls?.[0]}
                                                    <div class="space-y-1">
                                                        <Label class="text-xs font-semibold text-zinc-500 uppercase tracking-wider">Website URL</Label>
                                                        <Input readonly value={selectedTrashEntry.urls[0]} class="bg-zinc-900/10 border-zinc-900 text-zinc-400 text-xs h-9.5" />
                                                    </div>
                                                {/if}
                                            {:else}
                                                <div class="space-y-1">
                                                    <Label class="text-xs font-semibold text-zinc-500 uppercase tracking-wider">Secure Note Body</Label>
                                                    <textarea readonly class="w-full bg-zinc-900/10 border border-zinc-900 rounded-lg text-zinc-400 p-3 text-xs h-64 resize-none focus:outline-none">{selectedTrashEntry.notes}</textarea>
                                                </div>
                                            {/if}
                                        </div>

                                        <!-- Actions -->
                                        <div class="border-t border-zinc-900 pt-6 flex gap-4">
                                            <Button
                                                class="w-1/2 bg-zinc-50 text-zinc-950 hover:bg-zinc-200 text-xs h-9.5 font-bold flex items-center justify-center gap-1.5"
                                                onclick={() => selectedTrashEntry && handleRestoreTrashEntry(selectedTrashEntry.id)}
                                            >
                                                Restore Entry
                                            </Button>
                                            <Button
                                                class="w-1/2 text-xs h-9.5 font-bold bg-red-950/40 border border-red-800/60 text-red-200 hover:bg-red-900/50 hover:text-white flex items-center justify-center gap-1.5 transition-colors"
                                                onclick={() => selectedTrashEntry && handlePurgeTrashEntry(selectedTrashEntry.id)}
                                            >
                                                Delete Permanently
                                            </Button>
                                        </div>
                                    </div>
                                {/if}
                            </div>
                        </div>
                    {:else if activeTab === "settings"}
                        <div class="max-w-xl mx-auto space-y-6">
                            <div>
                                <h2 class="text-lg font-bold text-white mb-1">
                                    Extension Settings
                                </h2>
                                <p class="text-xs text-zinc-400">
                                    Manage your local database backup exports and security settings.
                                </p>
                            </div>

                            <div class="space-y-4">
                                <!-- BACKUP & EXPORT CARD -->
                                <div class="p-4 rounded-lg bg-zinc-950 border border-zinc-900 space-y-3">
                                    <div class="flex items-center gap-2">
                                        <Download class="h-4 w-4 text-zinc-200" />
                                        <h3 class="text-sm font-semibold text-white">Backup & Export Data</h3>
                                    </div>
                                    <p class="text-xs text-zinc-400 leading-relaxed">
                                        Download your decrypted vault credentials in CSV or JSON formats. WARNING: These files will contain your passwords in plain text. Please handle them with extreme caution and store them securely.
                                    </p>
                                    <div class="pt-2">
                                        {#if showExportConfirm}
                                            <div class="p-4 bg-zinc-900/50 border border-zinc-800/80 rounded-lg space-y-3">
                                                <p class="text-xs text-zinc-300">
                                                    Enter your master password to authorize the decrypted export ({exportType?.toUpperCase()}):
                                                </p>
                                                <div class="relative">
                                                    <input
                                                        type={showExportPassword ? "text" : "password"}
                                                        bind:value={exportPassword}
                                                        placeholder="Master password"
                                                        class="w-full bg-zinc-950 border border-zinc-800 text-white text-xs px-3 py-2 pr-10 rounded-lg focus:outline-none focus:border-zinc-700 focus:ring-1 focus:ring-zinc-700/50 transition-all"
                                                        onkeydown={(e) => e.key === "Enter" && handleConfirmExport()}
                                                    />
                                                    <button
                                                        type="button"
                                                        class="absolute right-3 top-1/2 -translate-y-1/2 text-zinc-450 hover:text-white transition-colors"
                                                        onclick={() => showExportPassword = !showExportPassword}
                                                    >
                                                        {#if showExportPassword}
                                                            <EyeOff class="h-3.5 w-3.5" />
                                                        {:else}
                                                            <Eye class="h-3.5 w-3.5" />
                                                        {/if}
                                                    </button>
                                                </div>
                                                {#if exportError}
                                                    <p class="text-[11px] text-red-400 flex items-center gap-1">
                                                        <span>⚠</span> {exportError}
                                                    </p>
                                                {/if}
                                                <div class="flex gap-2 pt-1">
                                                    <button
                                                        class="px-3.5 py-1.5 bg-zinc-50 hover:bg-zinc-200 text-zinc-950 rounded-lg text-xs font-semibold transition-colors focus-visible:ring-1 focus-visible:ring-zinc-700"
                                                        onclick={handleConfirmExport}
                                                    >
                                                        Confirm Export
                                                    </button>
                                                    <button
                                                        class="px-3.5 py-1.5 bg-zinc-900 hover:bg-zinc-800 text-zinc-300 rounded-lg text-xs font-semibold transition-colors border border-zinc-800"
                                                        onclick={() => { showExportConfirm = false; exportPassword = ''; exportError = ''; }}
                                                    >
                                                        Cancel
                                                    </button>
                                                </div>
                                            </div>
                                        {:else}
                                            <div class="flex flex-wrap gap-2">
                                                <button
                                                    class="px-3.5 py-2 bg-zinc-50 text-zinc-950 hover:bg-zinc-200 disabled:opacity-50 disabled:hover:bg-zinc-50 rounded-lg text-xs font-semibold flex items-center gap-1.5 transition-colors focus-visible:ring-1 focus-visible:ring-zinc-700"
                                                    onclick={() => requestExport('json')}
                                                    disabled={!vaultState.isUnlocked}
                                                >
                                                    <Download class="h-3.5 w-3.5" />
                                                    Export All Data
                                                </button>
                                                <button
                                                    class="px-3.5 py-2 bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 disabled:opacity-50 disabled:hover:bg-zinc-900 rounded-lg text-xs font-semibold text-zinc-100 flex items-center gap-1.5 transition-colors focus-visible:ring-1 focus-visible:ring-zinc-700"
                                                    onclick={() => requestExport('csv')}
                                                    disabled={!vaultState.isUnlocked}
                                                >
                                                    <FileSpreadsheet class="h-3.5 w-3.5" />
                                                    Export Passwords (CSV)
                                                </button>
                                            </div>
                                        {/if}
                                        {#if !vaultState.isUnlocked}
                                            <p class="text-[11px] text-amber-500 mt-2">
                                                ⚠ Vault must be unlocked to export a decrypted backup.
                                            </p>
                                        {/if}
                                    </div>
                                </div>

                                <!-- IMPORT DATA CARD -->
                                <div class="p-4 rounded-lg bg-zinc-950 border border-zinc-900 space-y-3">
                                    <div class="flex items-center gap-2">
                                        <Upload class="h-4 w-4 text-zinc-200" />
                                        <h3 class="text-sm font-semibold text-white">Import Data</h3>
                                    </div>
                                    <p class="text-xs text-zinc-400 leading-relaxed">
                                        Import credentials from an RFC 4180 CSV file. Columns must contain a header for <code>name</code> or <code>title</code>.
                                    </p>
                                    <div class="pt-2">
                                        <label
                                            class="inline-flex px-3.5 py-2 bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-zinc-100 disabled:opacity-50 disabled:hover:bg-zinc-900 rounded-lg text-xs font-semibold items-center gap-1.5 transition-colors cursor-pointer focus-within:ring-1 focus-within:ring-zinc-700"
                                            class:opacity-50={!vaultState.isUnlocked}
                                            class:cursor-not-allowed={!vaultState.isUnlocked}
                                        >
                                            <Upload class="h-3.5 w-3.5" />
                                            Import from CSV
                                            <input
                                                type="file"
                                                accept=".csv"
                                                class="hidden"
                                                disabled={!vaultState.isUnlocked}
                                                onchange={handleImportCsv}
                                            />
                                        </label>
                                        {#if importSuccessMsg}
                                            <p class="text-[11px] text-emerald-400 mt-2 flex items-center gap-1">
                                                <span>✓</span> {importSuccessMsg}
                                            </p>
                                        {/if}
                                        {#if importErrorMsg}
                                            <p class="text-[11px] text-red-400 mt-2 flex items-center gap-1">
                                                <span>⚠</span> {importErrorMsg}
                                            </p>
                                        {/if}
                                        {#if !vaultState.isUnlocked}
                                            <p class="text-[11px] text-amber-500 mt-2">
                                                ⚠ Vault must be unlocked to import.
                                            </p>
                                        {/if}
                                    </div>
                                </div>

                                <!-- SECURITY SETTINGS CARD -->
                                <div class="p-4 rounded-lg bg-zinc-950 border border-zinc-900 space-y-3">
                                    <div class="flex items-center gap-2">
                                        <ShieldCheck class="h-4 w-4 text-zinc-200" />
                                        <h3 class="text-sm font-semibold text-white">Security Settings</h3>
                                    </div>
                                    <p class="text-xs text-zinc-400 leading-relaxed">
                                        Configure re-authentication requirements for viewing, copying, or autofilling passwords.
                                    </p>
                                    <div class="pt-2 space-y-2">
                                        {#if bioSupported}
                                            <div class="flex items-center justify-between p-3 bg-zinc-900/40 border border-zinc-800/80 rounded-lg">
                                                <div class="space-y-0.5">
                                                    <span class="text-xs font-semibold text-zinc-200">Biometric Re-authentication</span>
                                                    <p class="text-[10px] text-zinc-500 max-w-[320px]">
                                                        Use Touch ID / Windows Hello to reveal, copy, or autofill passwords on this device.
                                                    </p>
                                                </div>
                                                <button
                                                    type="button"
                                                    onclick={handleToggleBiometrics}
                                                    class="px-3 py-1 bg-zinc-800 border border-zinc-700 hover:bg-zinc-700 text-zinc-100 text-xs font-semibold rounded-lg transition-colors cursor-pointer focus:outline-none"
                                                >
                                                    {bioEnabled ? 'Disable' : 'Enable'}
                                                </button>
                                            </div>
                                        {:else}
                                            <div class="p-3 bg-zinc-900/20 border border-zinc-800/40 rounded-lg">
                                                <p class="text-[11px] text-zinc-500">
                                                    Biometric authentication (Touch ID / Windows Hello) is not supported or enabled on this device/browser. Master Password verification will be used as the default fallback.
                                                </p>
                                            </div>
                                        {/if}

                                        {#if bioError}
                                            <p class="text-[11px] text-red-400 flex items-center gap-1">
                                                <span>⚠</span> {bioError}
                                            </p>
                                        {/if}
                                    </div>
                                </div>

                                <!-- RESET VAULT CARD -->
                                <div class="p-4 rounded-lg bg-red-950/10 border border-red-900/20 space-y-3">
                                    <div class="flex items-center gap-2">
                                        <Database class="h-4 w-4 text-red-400" />
                                        <h3 class="text-sm font-semibold text-red-300">Reset Vault Database</h3>
                                    </div>
                                    <p class="text-xs text-zinc-450 leading-relaxed">
                                        Permanently delete the local encryption payload and keys. This action will completely erase all stored credentials on this browser instance. This cannot be undone.
                                    </p>
                                    <div class="pt-2">
                                        {#if showResetConfirm}
                                            <div class="p-4 bg-red-950/20 border border-red-900/30 rounded-lg space-y-3">
                                                <p class="text-xs text-red-200">
                                                    Warning: This will wipe all local data. Type <strong class="text-red-400 font-mono">RESET</strong> to confirm:
                                                </p>
                                                <input
                                                    type="text"
                                                    bind:value={resetConfirmText}
                                                    placeholder="Type RESET"
                                                    class="w-full bg-zinc-950 border border-red-900/30 text-red-200 text-xs px-3 py-2 rounded-lg focus:outline-none focus:border-red-650 focus:ring-1 focus:ring-red-900/40 transition-all"
                                                />
                                                {#if resetError}
                                                    <p class="text-[11px] text-red-400 flex items-center gap-1">
                                                        <span>⚠</span> {resetError}
                                                    </p>
                                                {/if}
                                                <div class="flex gap-2 pt-1">
                                                    <button
                                                        class="px-3.5 py-1.5 bg-red-600 hover:bg-red-500 text-white rounded-lg text-xs font-semibold transition-colors focus-visible:ring-1 focus-visible:ring-red-700"
                                                        onclick={handleResetVault}
                                                    >
                                                        Confirm Complete Reset
                                                    </button>
                                                    <button
                                                        class="px-3.5 py-1.5 bg-zinc-900 hover:bg-zinc-800 text-zinc-300 rounded-lg text-xs font-semibold transition-colors border border-zinc-800"
                                                        onclick={() => { showResetConfirm = false; resetConfirmText = ''; resetError = ''; }}
                                                    >
                                                        Cancel
                                                    </button>
                                                </div>
                                            </div>
                                        {:else}
                                            <button
                                                class="px-3.5 py-2 bg-red-500/10 hover:bg-red-500/20 border border-red-500/20 text-red-400 rounded-lg text-xs font-semibold flex items-center gap-1.5 transition-all focus-visible:ring-1 focus-visible:ring-red-700"
                                                onclick={() => (showResetConfirm = true)}
                                            >
                                                <Trash2 class="h-3.5 w-3.5" />
                                                Reset Local Vault
                                            </button>
                                        {/if}
                                    </div>
                                </div>
                            </div>
                        </div>
                    {:else if activeTab === "help"}
                        <div class="grow overflow-y-auto p-8 space-y-6">
                            <div class="max-w-xl mx-auto space-y-6">
                                <div>
                                    <h2 class="text-lg font-bold text-white mb-1 flex items-center gap-2">
                                        <CircleHelp class="h-5 w-5 text-[#06b6d4]" />
                                        Help & Guide
                                    </h2>
                                    <p class="text-xs text-zinc-400">
                                        Learn how to get the most out of your zero-knowledge password manager.
                                    </p>
                                </div>

                                <div class="space-y-4">
                                    <!-- SECTION 1: AUTOFILL -->
                                    <div class="p-4 rounded-lg bg-zinc-950 border border-zinc-900 space-y-2">
                                        <div class="flex items-center gap-2">
                                            <div class="p-1.5 bg-[#06b6d4]/10 rounded-md text-[#06b6d4]">
                                                <Fingerprint class="h-4 w-4" />
                                            </div>
                                            <h3 class="text-sm text-white font-semibold">1. Extension Autofill</h3>
                                        </div>
                                        <p class="text-xs text-zinc-400 leading-relaxed text-left">
                                            KeyVault injects an <strong>Inline Autofill Overlay</strong> directly into detected input fields on login screens.
                                            Click the <strong>Autofill Badge</strong> (the key icon inside the text input) to open the <strong>Autofill Dropdown Panel</strong> and instantly select your account credentials.
                                        </p>
                                    </div>

                                    <!-- SECTION 2: SHORTCUTS -->
                                    <div class="p-4 rounded-lg bg-zinc-950 border border-zinc-900 space-y-2">
                                        <div class="flex items-center gap-2">
                                            <div class="p-1.5 bg-yellow-500/10 rounded-md text-yellow-400">
                                                <KeyRound class="h-4 w-4" />
                                            </div>
                                            <h3 class="text-sm text-white font-semibold">2. Keyboard Shortcuts</h3>
                                        </div>
                                        <p class="text-xs text-zinc-400 leading-relaxed text-left">
                                            Navigate quickly using your keyboard:
                                        </p>
                                        <div class="grid grid-cols-2 gap-2 text-[10px] text-zinc-300 font-mono pt-1 text-left">
                                            <div class="flex justify-between p-1.5 bg-zinc-900/40 rounded border border-zinc-900">
                                                <span>Open Popup:</span>
                                                <span class="text-[#06b6d4]">Cmd+Shift+K</span>
                                            </div>
                                            <div class="flex justify-between p-1.5 bg-zinc-900/40 rounded border border-zinc-900">
                                                <span>Search Vault:</span>
                                                <span class="text-[#06b6d4]">/</span>
                                            </div>
                                            <div class="flex justify-between p-1.5 bg-zinc-900/40 rounded border border-zinc-900">
                                                <span>Focus Next:</span>
                                                <span class="text-[#06b6d4]">Tab</span>
                                            </div>
                                            <div class="flex justify-between p-1.5 bg-zinc-900/40 rounded border border-zinc-900">
                                                <span>Lock Vault:</span>
                                                <span class="text-[#06b6d4]">Esc</span>
                                            </div>
                                        </div>
                                    </div>

                                    <!-- SECTION 3: SYNC -->
                                    <div class="p-4 rounded-lg bg-zinc-950 border border-zinc-900 space-y-2">
                                        <div class="flex items-center gap-2">
                                            <div class="p-1.5 bg-green-500/10 rounded-md text-green-400">
                                                <Globe class="h-4 w-4" />
                                            </div>
                                            <h3 class="text-sm text-white font-semibold">3. Cloud Synchronisation</h3>
                                        </div>
                                        <p class="text-xs text-zinc-400 leading-relaxed text-left">
                                            Sync keeps your database updated across all devices. We use client-side zero-knowledge encryption where credentials are encrypted locally with your derived Vault Key before uploading to Google Drive's secure <code>appDataFolder</code>.
                                        </p>
                                        <p class="text-[10px] text-zinc-500 leading-normal text-left">
                                            * A silent sync merges local/remote databases every time you add, update, or unlock credentials.
                                        </p>
                                    </div>

                                    <!-- SECTION 4: BEST PRACTICES -->
                                    <div class="p-4 rounded-lg bg-zinc-950 border border-zinc-900 space-y-2">
                                        <div class="flex items-center gap-2">
                                            <div class="p-1.5 bg-[#06b6d4]/10 rounded-md text-[#06b6d4]">
                                                <ShieldCheck class="h-4 w-4" />
                                            </div>
                                            <h3 class="text-sm text-white font-semibold">4. Security Best Practices</h3>
                                        </div>
                                        <ul class="text-xs text-zinc-400 leading-relaxed text-left list-disc pl-4 space-y-1">
                                            <li><strong>Generator</strong>: Always use the built-in Password Generator to create high-entropy, unique passwords.</li>
                                            <li><strong>Recovery Key</strong>: Keep your emergency recovery key written down offline in a physically secure location.</li>
                                            <li>
                                                <strong>Custom Client ID</strong>: For advanced control, you can configure your own OAuth Client ID under settings to sync directly to your personal API developer console.
                                                <details class="mt-2 border border-zinc-850 bg-zinc-900/40 rounded-md overflow-hidden">
                                                    <summary class="flex justify-between items-center p-2 text-[10px] font-medium text-zinc-300 cursor-pointer hover:bg-zinc-800/40 select-none">
                                                        <span>Guide: How to create a Custom Client ID</span>
                                                    </summary>
                                                    <div class="p-3 border-t border-zinc-850 text-[10px] text-zinc-400 font-sans space-y-2 text-left leading-relaxed">
                                                        <p>To use your own Google Cloud configuration for extension sync:</p>
                                                        <ol class="list-decimal pl-4 space-y-1">
                                                            <li>Go to the <a href="https://console.cloud.google.com/" target="_blank" rel="noopener noreferrer" class="text-[#06b6d4] hover:underline">Google Cloud Console</a>.</li>
                                                            <li>Create a new project (e.g. <code>KeyVault-Extension</code>).</li>
                                                            <li>Go to <strong>APIs & Services &gt; Library</strong>, search for <strong>Google Drive API</strong>, and click <strong>Enable</strong>.</li>
                                                            <li>Go to <strong>OAuth consent screen</strong>, select <strong>External</strong>, and click <strong>Create</strong>. Fill in app details, add scope <code>https://www.googleapis.com/auth/drive.appdata</code>, and add your email as a <strong>Test User</strong>.</li>
                                                            <li>Go to <strong>Credentials</strong>, click <strong>Create Credentials &gt; OAuth Client ID</strong>.</li>
                                                            <li>Select <strong>Web application</strong> as the application type.</li>
                                                            <li>Under <strong>Authorized redirect URIs</strong>, add <code>https://&lt;your-extension-id&gt;.chromiumapp.org/</code> (find your Extension ID in <code>chrome://extensions</code>).</li>
                                                            <li>Copy the generated <strong>Client ID</strong> and paste it in KeyVault's <strong>Settings &gt; Custom Google Client ID</strong>.</li>
                                                        </ol>
                                                    </div>
                                                </details>
                                            </li>
                                        </ul>
                                    </div>
                                </div>
                            </div>
                        </div>
                    {/if}
                </div>
            {/if}
        {/if}
        <ReauthModal />
    </main>
</div>
