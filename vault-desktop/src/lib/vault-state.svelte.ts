import { GoogleDriveProvider } from './sync/google-drive.js';
import type {
  Account,
  DomainGroup,
  RemoteVaultMetadata,
  SecureNote,
  Vault,
  VaultItem,
} from './types.js';

function getBaseDomain(url: string): string | null {
  if (!url) return null;
  try {
    const cleaned = url.trim().toLowerCase();
    const withoutProtocol = cleaned
      .replace(/^https?:\/\//, '')
      .replace(/^ftp:\/\//, '');
    const host = withoutProtocol.split('/')[0].split(':')[0];
    const base = host.replace(/^www\./, '');
    return base || null;
  } catch {
    return null;
  }
}

import { createContext } from 'svelte';
import { getErrorMsg, safeInvoke } from './tauri-ipc.js';

const invoke = safeInvoke as any;

class VaultState {
  // Authentication & Session State
  isUnlocked = $state(false);
  vault = $state<Vault | null>(null);
  error = $state('');
  loading = $state(false);

  // Search & Navigation State
  searchQuery = $state('');
  activeTab = $state<'all' | 'logins' | 'notes' | 'trash'>('all');
  selectedItem = $state<VaultItem | null>(null);
  selectedTag = $state<string | null>(null);

  // Sync Provider & State
  syncProvider = new GoogleDriveProvider();
  syncing = $state(false);
  syncError = $state('');
  syncNeedsPassword = $state(false);
  pendingRemotePayload = $state<Uint8Array | null>(null);
  pendingRemoteMetadata = $state<RemoteVaultMetadata | null>(null);
  pendingRemoteSalt = $state<Uint8Array | null>(null);

  // Google OAuth Profile
  gdriveAuthenticated = $state(false);
  gdriveEmail = $state('');

  // Vault file existence
  vaultExists = $state(false);

  // Biometrics State
  bioSupported = $state(false);
  bioEnabled = $state(localStorage.getItem('biometrics_enabled') === 'true');

  constructor() {
    this.checkGDriveAuth();
    this.checkVaultExists();
    this.checkBiometricsSupport();
  }

  async checkVaultExists() {
    if (!invoke) return;
    try {
      this.vaultExists = await invoke('check_vault_exists');
    } catch {
      this.vaultExists = false;
    }
  }

  async checkGDriveAuth() {
    const customClientId = localStorage.getItem('gdrive_custom_client_id');
    if (customClientId) {
      this.syncProvider.setClientId(customClientId);
    }
    const authenticated = await this.syncProvider.isAuthenticated();
    this.gdriveAuthenticated = authenticated;
    if (authenticated) {
      const info = await this.syncProvider.getUserInfo();
      if (info) {
        this.gdriveEmail = info.email;
      }
    }
  }

  async signInGDrive() {
    try {
      this.loading = true;
      this.syncError = '';
      const customClientId = localStorage.getItem('gdrive_custom_client_id');
      if (customClientId) {
        this.syncProvider.setClientId(customClientId);
      }

      const success = await this.syncProvider.signIn(true);
      if (success) {
        this.gdriveAuthenticated = true;
        const info = await this.syncProvider.getUserInfo();
        if (info) {
          this.gdriveEmail = info.email;
        }
        // Run initial sync after successful sign-in
        await this.syncVault();
      } else {
        this.syncError = 'Google Drive sign-in failed.';
      }
    } catch (err) {
      this.syncError = getErrorMsg(err);
    } finally {
      this.loading = false;
    }
  }

  async signOutGDrive() {
    try {
      this.loading = true;
      await this.syncProvider.signOut();
      this.gdriveAuthenticated = false;
      this.gdriveEmail = '';
    } catch (err) {
      console.error('Sign-out failed:', err);
    } finally {
      this.loading = false;
    }
  }

  async unlock(password: string): Promise<boolean> {
    if (!invoke) {
      this.error = 'Tauri backend not available.';
      return false;
    }

    try {
      this.loading = true;
      this.error = '';
      await invoke('unlock_vault', { password });
      this.isUnlocked = true;
      await this.loadVault();
      await this.checkVaultExists();

      // Auto-trigger sync on unlock if signed in
      if (this.gdriveAuthenticated) {
        this.syncVault();
      }
      return true;
    } catch (err) {
      this.error = getErrorMsg(err);
      return false;
    } finally {
      this.loading = false;
    }
  }

  async lock() {
    if (!invoke) return;
    try {
      await invoke('lock_vault');
    } catch (err) {
      console.error('Failed to lock vault in backend:', err);
    }
    this.isUnlocked = false;
    this.vault = null;
    this.selectedItem = null;
    await this.checkVaultExists();
  }

  async resetVault() {
    if (!invoke) return false;
    try {
      this.loading = true;
      this.error = '';
      await invoke('reset_vault');
      this.isUnlocked = false;
      this.vault = null;
      this.selectedItem = null;
      await this.checkVaultExists();
      return true;
    } catch (err) {
      this.error = `Reset failed: ${getErrorMsg(err)}`;
      return false;
    } finally {
      this.loading = false;
    }
  }

  async loadVault() {
    if (!invoke) return;
    try {
      const v = await invoke('get_vault');
      this.vault = v as Vault;
      // Refresh selected item if it exists
      if (this.selectedItem) {
        const id = this.selectedItem.id;
        const activeItem = this.vault.items.find((i) => i.id === id);
        const trashItem = this.vault.trash.find((i) => i.id === id);
        this.selectedItem = activeItem || trashItem || null;
      }
    } catch (err) {
      this.error = getErrorMsg(err);
    }
  }

  async saveItem(item: VaultItem) {
    if (!invoke) return;
    try {
      this.error = '';
      this.loading = true;
      await invoke('save_vault_item', { item });
      await this.loadVault();

      // Sync in background after save
      if (this.gdriveAuthenticated) {
        this.syncVault();
      }
    } catch (err) {
      this.error = getErrorMsg(err);
    } finally {
      this.loading = false;
    }
  }

  async deleteItem(id: string) {
    if (!invoke) {
      console.warn('[VaultState] deleteItem: invoke not available');
      return;
    }
    try {
      this.error = '';
      this.loading = true;
      const now = new Date().toISOString();
      await invoke('delete_vault_item', { id, deletedAt: now });
      if (this.selectedItem?.id === id) {
        this.selectedItem = null;
      }
      await this.loadVault();

      if (this.gdriveAuthenticated) {
        this.syncVault();
      }
    } catch (err) {
      console.error('[VaultState] deleteItem error:', err);
      this.error = getErrorMsg(err);
    } finally {
      this.loading = false;
    }
  }

  async restoreItem(id: string) {
    if (!invoke) return;
    try {
      this.error = '';
      this.loading = true;
      const now = new Date().toISOString();
      await invoke('restore_vault_item', { id, restoredAt: now });
      await this.loadVault();

      if (this.gdriveAuthenticated) {
        this.syncVault();
      }
    } catch (err) {
      this.error = getErrorMsg(err);
    } finally {
      this.loading = false;
    }
  }

  async purgeItem(id: string) {
    if (!invoke) return;
    try {
      this.error = '';
      this.loading = true;
      const now = new Date().toISOString();
      await invoke('purge_vault_item', { id, purgedAt: now });
      if (this.selectedItem?.id === id) {
        this.selectedItem = null;
      }
      await this.loadVault();

      if (this.gdriveAuthenticated) {
        this.syncVault();
      }
    } catch (err) {
      this.error = getErrorMsg(err);
    } finally {
      this.loading = false;
    }
  }

  // --- Google Drive Sync Workflows ---

  async syncVault() {
    if (!this.gdriveAuthenticated || this.syncing) return;

    try {
      this.syncing = true;
      this.syncError = '';

      const downloadResult = await this.syncProvider.downloadVault();
      if (!downloadResult) {
        // No backup found, perform initial backup
        await this.backupLocal();
        return;
      }

      const { payload, metadata } = downloadResult;

      // Attempt to decrypt remote vault using session key
      try {
        const [remoteVault] = await invoke('decrypt_remote_vault', {
          payload: Array.from(payload),
          password: null,
        });

        // Success! Perform merge
        const merged = await invoke('merge_vaults', {
          remote_vault: remoteVault,
        });
        this.vault = merged as Vault;

        // Fetch local encrypted payload and upload it back
        const updatedPayload = await invoke('get_encrypted_vault_payload');
        const newMetadata = await this.syncProvider.uploadVault(
          new Uint8Array(updatedPayload),
          metadata,
        );

        console.warn('Sync complete, new ETag:', newMetadata.eTag);
      } catch (err) {
        // Mismatch or decryption failure
        console.warn(
          'Sync salt/key mismatch detected, entering resolution flow:',
          err,
        );

        this.pendingRemotePayload = payload;
        this.pendingRemoteMetadata = metadata;
        this.pendingRemoteSalt = payload.slice(4, 20); // Extract salt from KV01 header
        this.syncNeedsPassword = true;
      }
    } catch (err) {
      this.syncError = `Sync failed: ${getErrorMsg(err)}`;
    } finally {
      this.syncing = false;
    }
  }

  async restoreRemoteFirstTime(password: string): Promise<boolean> {
    if (!this.pendingRemotePayload) return false;
    try {
      this.syncError = '';
      this.loading = true;
      // Decrypt remote vault using password
      const [remoteVault, _, remoteKey] = await invoke('decrypt_remote_vault', {
        payload: Array.from(this.pendingRemotePayload),
        password,
      });
      // Overwrite local vault
      await invoke('overwrite_local_vault', {
        remote_vault: remoteVault,
        remote_key: remoteKey,
      });
      this.isUnlocked = true;
      this.vault = remoteVault;
      await this.checkVaultExists();

      // Clear pending state
      this.syncNeedsPassword = false;
      this.pendingRemotePayload = null;
      this.pendingRemoteMetadata = null;
      this.pendingRemoteSalt = null;
      return true;
    } catch (err) {
      this.syncError = `Restore failed: ${getErrorMsg(err)}`;
      return false;
    } finally {
      this.loading = false;
    }
  }

  async resolveSyncSaltMismatch(password: string): Promise<boolean> {
    if (!this.pendingRemotePayload) return false;

    try {
      this.syncError = '';
      this.loading = true;

      // Attempt decryption on the backend with the provided password
      const [remoteVault, _, remoteKey] = await invoke('decrypt_remote_vault', {
        payload: Array.from(this.pendingRemotePayload),
        password,
      });

      // Merge local with remote
      const merged = await invoke('merge_vaults', {
        remote_vault: remoteVault,
      });
      this.vault = merged as Vault;

      // Overwrite local credentials key and salt with remote key/salt
      await invoke('overwrite_local_vault', {
        remote_vault: merged,
        remote_key: remoteKey,
      });

      // Re-encrypt and upload to Google Drive
      const updatedPayload = await invoke('get_encrypted_vault_payload');
      await this.syncProvider.uploadVault(
        new Uint8Array(updatedPayload),
        this.pendingRemoteMetadata || undefined,
      );

      // Clean mismatch state
      this.syncNeedsPassword = false;
      this.pendingRemotePayload = null;
      this.pendingRemoteMetadata = null;
      this.pendingRemoteSalt = null;
      return true;
    } catch {
      this.syncError = 'Decryption failed: Invalid password.';
      return false;
    } finally {
      this.loading = false;
    }
  }

  async signInAndFetchRemoteVault(): Promise<{
    exists: boolean;
    error?: string;
  }> {
    try {
      this.error = '';
      this.syncError = '';
      this.loading = true;

      // 1. Initialize Google Drive sign-in (interactive = true)
      const customClientId = localStorage.getItem('gdrive_custom_client_id');
      if (customClientId) {
        this.syncProvider.setClientId(customClientId);
      }

      const authenticated = await this.syncProvider.signIn(true);
      if (!authenticated) {
        return { exists: false, error: 'Google Drive authentication failed.' };
      }
      this.gdriveAuthenticated = true;
      const info = await this.syncProvider.getUserInfo();
      if (info) {
        this.gdriveEmail = info.email;
      }

      // 2. Search and download remote vault
      const result = await this.syncProvider.downloadVault();
      if (!result) {
        // No remote vault exists
        return { exists: false };
      }

      const { payload, metadata } = result;

      // 3. Extract remote salt (KV01 header: bytes 4 to 20)
      let remoteSalt: Uint8Array;
      if (payload.length >= 20) {
        remoteSalt = payload.slice(4, 20);
      } else {
        return { exists: false, error: 'Invalid remote vault file format.' };
      }

      // Set pending remote details
      this.pendingRemoteSalt = remoteSalt;
      this.pendingRemotePayload = payload;
      this.pendingRemoteMetadata = metadata;
      this.syncNeedsPassword = true;

      return { exists: true };
    } catch (e) {
      const errMsg = e instanceof Error ? e.message : getErrorMsg(e);
      this.syncError = errMsg;
      return { exists: false, error: errMsg };
    } finally {
      this.loading = false;
    }
  }

  async restoreRemote() {
    if (!this.gdriveAuthenticated) return;
    try {
      this.syncing = true;
      this.syncError = '';
      const downloadResult = await this.syncProvider.downloadVault();
      if (!downloadResult) {
        this.syncError = 'No remote backup found.';
        return;
      }

      const { payload, metadata } = downloadResult;

      // Decrypt using current session key
      try {
        const [remoteVault, _, remoteKey] = await invoke(
          'decrypt_remote_vault',
          { payload: Array.from(payload), password: null },
        );
        await invoke('overwrite_local_vault', {
          remote_vault: remoteVault,
          remote_key: remoteKey,
        });
        await this.loadVault();
      } catch {
        // Password required
        this.pendingRemotePayload = payload;
        this.pendingRemoteMetadata = metadata;
        this.pendingRemoteSalt = payload.slice(4, 20);
        this.syncNeedsPassword = true;
        this.syncError =
          'Remote vault is encrypted with a different key. Please enter its password.';
      }
    } catch (err) {
      this.syncError = `Restore failed: ${getErrorMsg(err)}`;
    } finally {
      this.syncing = false;
    }
  }

  async backupLocal() {
    if (!this.gdriveAuthenticated) return;
    try {
      this.syncing = true;
      this.syncError = '';

      // Fetch metadata first to get ETag
      const downloadResult = await this.syncProvider.downloadVault();
      const metadata = downloadResult?.metadata;

      const payload = await invoke('get_encrypted_vault_payload');
      await this.syncProvider.uploadVault(
        new Uint8Array(payload),
        metadata || undefined,
      );
    } catch (err) {
      this.syncError = `Backup failed: ${getErrorMsg(err)}`;
    } finally {
      this.syncing = false;
    }
  }

  async wipeCloud() {
    if (!this.gdriveAuthenticated) return;
    try {
      this.syncing = true;
      this.syncError = '';
      const success = await this.syncProvider.deleteVault?.();
      if (!success) {
        this.syncError = 'Failed to delete remote vault backup.';
      }
    } catch (err) {
      this.syncError = `Wipe failed: ${getErrorMsg(err)}`;
    } finally {
      this.syncing = false;
    }
  }

  // Helper selectors
  get filteredItems() {
    if (!this.vault) return [];

    // Select collection (items or trash)
    const list =
      this.activeTab === 'trash' ? this.vault.trash : this.vault.items;

    return list.filter((item) => {
      // 1. Search Query Filter
      if (this.searchQuery.trim()) {
        const q = this.searchQuery.toLowerCase();
        const matchesTitle = item.title?.toLowerCase().includes(q);

        let matchesDetail = false;
        if (item.type === 'DomainGroup') {
          matchesDetail =
            item.accounts.some(
              (acc) =>
                acc.username?.toLowerCase().includes(q) ||
                acc.notes?.toLowerCase().includes(q),
            ) || item.urls.some((url) => url.toLowerCase().includes(q));
        } else {
          matchesDetail = item.notes?.toLowerCase().includes(q);
        }

        if (!matchesTitle && !matchesDetail) return false;
      }

      // 2. Tab Category Filter
      if (this.activeTab === 'logins' && item.type !== 'DomainGroup')
        return false;
      if (this.activeTab === 'notes' && item.type !== 'SecureNote')
        return false;

      // 3. Tag Filter
      if (this.selectedTag) {
        if (!item.tags?.includes(this.selectedTag)) return false;
      }

      return true;
    });
  }

  get allTags() {
    if (!this.vault) return [];
    const list =
      this.activeTab === 'trash' ? this.vault.trash : this.vault.items;
    const tagSet = new Set<string>();
    for (const item of list) {
      if (item.tags) {
        for (const tag of item.tags) {
          tagSet.add(tag);
        }
      }
    }
    return Array.from(tagSet).sort();
  }

  // New settings & biometrics support methods
  async checkBiometricsSupport() {
    if (!invoke) return;
    try {
      this.bioSupported = await invoke('is_biometrics_supported');
    } catch {
      this.bioSupported = false;
    }
  }

  async verifyPassword(password: string): Promise<boolean> {
    if (!invoke) return false;
    try {
      return await invoke('verify_password', { password });
    } catch {
      return false;
    }
  }

  async generateCredential(config: Record<string, unknown>) {
    if (!invoke) return null;
    try {
      return await invoke('generate_credential', { config });
    } catch (e) {
      console.error('Failed to generate credential:', e);
      return null;
    }
  }

  async exportCsv(): Promise<string> {
    if (!invoke) return '';
    try {
      return await invoke('export_vault_csv');
    } catch (e) {
      console.error('Failed to export CSV:', e);
      throw e;
    }
  }

  async importCsv(csvText: string, currentTime: string): Promise<number> {
    if (!invoke) return 0;
    try {
      const count = await invoke('import_vault_csv', {
        csvText,
        currentTime,
      });
      await this.loadVault();
      return count as number;
    } catch (e) {
      console.error('Failed to import CSV:', e);
      throw e;
    }
  }

  async selectAndImportCsv(currentTime: string): Promise<number> {
    if (!invoke) return 0;
    try {
      const count = await invoke('select_and_import_csv', {
        currentTime,
      });
      await this.loadVault();
      return count as number;
    } catch (e) {
      console.error('Failed to select and import CSV:', e);
      throw e;
    }
  }

  async addEntry(
    title: string,
    username?: string,
    password?: string,
    url?: string,
    notes?: string,
    entryType: 'Login' | 'SecureNote' = 'Login',
    totpSecret?: string,
    tags?: string[],
  ): Promise<boolean> {
    try {
      this.error = '';
      this.loading = true;
      const currentTime = new Date().toISOString();

      if (entryType === 'SecureNote') {
        const note: VaultItem = {
          type: 'SecureNote',
          id: crypto.randomUUID(),
          title: title.trim(),
          notes: notes?.trim() || '',
          tags: tags || [],
          created_at: currentTime,
          updated_at: currentTime,
        };
        await this.saveItem(note);
      } else {
        const targetUrl = url?.trim() || '';
        const baseDomain = getBaseDomain(targetUrl) || title.trim();

        // Find existing group with same base domain or URL match
        const existingGroup = this.vault?.items.find((i) => {
          if (i.type !== 'DomainGroup') return false;
          if (i.title.toLowerCase() === baseDomain.toLowerCase()) return true;
          if (targetUrl) {
            return i.urls.some((u) => {
              const b = getBaseDomain(u);
              return b && b.toLowerCase() === baseDomain.toLowerCase();
            });
          }
          return false;
        }) as DomainGroup | undefined;

        const newAccount: Account = {
          id: crypto.randomUUID(),
          username: username?.trim() || '',
          password: password || '',
          totp_secret: totpSecret?.trim() || null,
          notes: notes?.trim() || '',
          custom_fields: [],
          password_history: [],
          created_at: currentTime,
          updated_at: currentTime,
        };

        if (existingGroup) {
          const groupCopy: VaultItem = {
            type: 'DomainGroup',
            ...existingGroup,
            accounts: [...existingGroup.accounts, newAccount],
            urls:
              targetUrl && !existingGroup.urls.includes(targetUrl)
                ? [...existingGroup.urls, targetUrl]
                : existingGroup.urls,
            tags:
              tags && tags.length > 0
                ? Array.from(new Set([...(existingGroup.tags || []), ...tags]))
                : existingGroup.tags,
            updated_at: currentTime,
          };
          await this.saveItem(groupCopy);
        } else {
          const newGroup: VaultItem = {
            type: 'DomainGroup',
            id: crypto.randomUUID(),
            title: baseDomain,
            urls: targetUrl ? [targetUrl] : [],
            accounts: [newAccount],
            tags: tags || [],
            created_at: currentTime,
            updated_at: currentTime,
          };
          await this.saveItem(newGroup);
        }
      }
      return true;
    } catch (e) {
      this.error = getErrorMsg(e);
      return false;
    } finally {
      this.loading = false;
    }
  }

  async updateEntry(
    id: string,
    title: string,
    username?: string,
    password?: string,
    url?: string,
    notes?: string,
    entryType: 'Login' | 'SecureNote' = 'Login',
    totpSecret?: string,
    tags?: string[],
  ): Promise<boolean> {
    try {
      this.error = '';
      this.loading = true;
      const currentTime = new Date().toISOString();

      if (entryType === 'SecureNote') {
        const SN = this.vault?.items.find(
          (item) => item.type === 'SecureNote' && item.id === id,
        ) as SecureNote | undefined;
        if (!SN) throw new Error('Secure note not found');

        const updatedNote: VaultItem = {
          type: 'SecureNote',
          ...SN,
          title: title.trim(),
          notes: notes?.trim() || '',
          tags: tags || [],
          updated_at: currentTime,
        };
        await this.saveItem(updatedNote);
      } else {
        // Find group and account
        let foundGroup: DomainGroup | null = null;
        let foundAccount: Account | null = null;
        if (this.vault) {
          for (const item of this.vault.items) {
            if (item.type === 'DomainGroup') {
              const acc = item.accounts.find((a) => a.id === id);
              if (acc) {
                foundGroup = item;
                foundAccount = acc;
                break;
              }
            }
          }
        }

        if (!foundGroup || !foundAccount) {
          throw new Error('Account not found');
        }

        const oldPassword = foundAccount.password;
        const newPassword = password || '';
        const passwordHistory = foundAccount.password_history
          ? [...foundAccount.password_history]
          : [];
        if (oldPassword && oldPassword !== newPassword) {
          passwordHistory.push({
            password: oldPassword,
            changed_at: currentTime,
          });
        }

        const updatedAccount: Account = {
          ...foundAccount,
          username: username?.trim() || '',
          password: newPassword,
          totp_secret: totpSecret?.trim() || null,
          notes: notes?.trim() || '',
          password_history: passwordHistory,
          updated_at: currentTime,
        };

        const targetUrl = url?.trim() || '';
        const baseDomain = getBaseDomain(targetUrl) || title.trim();

        // Update the account inside the group
        const updatedAccounts = foundGroup.accounts.map((a) =>
          a.id === id ? updatedAccount : a,
        );

        const groupCopy: VaultItem = {
          type: 'DomainGroup',
          ...foundGroup,
          accounts: updatedAccounts,
          title: baseDomain,
          urls:
            targetUrl && !foundGroup.urls.includes(targetUrl)
              ? [...foundGroup.urls, targetUrl]
              : foundGroup.urls,
          updated_at: currentTime,
        };

        if (tags) {
          groupCopy.tags = tags;
        }

        await this.saveItem(groupCopy);
      }
      return true;
    } catch (e) {
      this.error = getErrorMsg(e);
      return false;
    } finally {
      this.loading = false;
    }
  }

  async deleteAccount(groupId: string, accountId: string): Promise<boolean> {
    if (!this.vault) return false;
    try {
      this.error = '';
      this.loading = true;
      const now = new Date().toISOString();

      const group = this.vault.items.find(
        (i) => i.type === 'DomainGroup' && i.id === groupId,
      ) as DomainGroup | undefined;
      if (group) {
        if (group.accounts.length <= 1) {
          // Move the entire domain group to trash and keep the account inside it
          await this.deleteItem(groupId);
        } else {
          // Purge the specific account. The Rust backend will remove it from the group,
          // update the group's updated_at timestamp, generate a tombstone, and save the vault.
          if (invoke) {
            await invoke('purge_vault_item', { id: accountId, purgedAt: now });
          }
        }
      }
      await this.loadVault();
      return true;
    } catch (err) {
      this.error = getErrorMsg(err);
      return false;
    } finally {
      this.loading = false;
    }
  }
}

export const [getVaultContext, setVaultContext] = createContext<VaultState>();
export const vaultState = new VaultState();
