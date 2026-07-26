import init, {
  wasm_decrypt_vault,
  wasm_decrypt_vault_packaged,
  wasm_derive_key,
  wasm_encrypt_vault,
  wasm_encrypt_vault_packaged,
  wasm_generate,
  wasm_generate_totp,
  wasm_vault_add_entry,
  wasm_vault_delete_entry,
  wasm_vault_export_csv,
  wasm_vault_extract_salt,
  wasm_vault_import_csv,
  wasm_vault_merge,
  wasm_vault_new,
  wasm_vault_purge_entry,
  wasm_vault_restore_entry,
  wasm_vault_update_entry,
} from 'vault-core';
import { getStorageProvider } from './sync/factory.js';

export interface CustomField {
  key: string;
  value: string;
}

export interface PasswordSnapshot {
  password: string;
  changed_at: string;
}

export interface Account {
  id: string;
  username?: string;
  password?: string;
  totp_secret?: string | null;
  notes?: string;
  custom_fields?: CustomField[];
  password_history?: PasswordSnapshot[];
  created_at?: string;
  updated_at: string;
}

export interface DomainGroup {
  id: string;
  title: string;
  urls: string[];
  accounts: Account[];
  tags: string[];
  created_at?: string;
  updated_at: string;
}

export interface SecureNote {
  id: string;
  title: string;
  notes: string;
  tags: string[];
  created_at?: string;
  updated_at: string;
}

export type VaultItem =
  | ({ type: 'DomainGroup' } & DomainGroup)
  | ({ type: 'SecureNote' } & SecureNote);

import type { RemoteVaultMetadata } from './sync/types.js';

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

class VaultState {
  wasmReady = $state(false);
  isRegistered = $state(false);
  isUnlocked = $state(false);
  masterKey = $state<Uint8Array | null>(null);
  vaultJson = $state('');
  error = $state('');

  syncNeedsPassword = $state(false);
  pendingRemoteSalt = $state<Uint8Array | null>(null);
  pendingRemotePayload = $state<Uint8Array | null>(null);
  pendingRemoteMetadata = $state<RemoteVaultMetadata | null>(null);

  // Derived rune to parse JSON
  vault = $derived.by(() => {
    try {
      if (!this.vaultJson) return { items: [], trash: [] };
      const parsed = JSON.parse(this.vaultJson);
      if (parsed) {
        if (!Array.isArray(parsed.items)) {
          parsed.items = [];
        }
        if (!Array.isArray(parsed.trash)) {
          parsed.trash = [];
        }
      }
      return parsed as {
        items: VaultItem[];
        trash: VaultItem[];
        tombstones: { id: string; deleted_at: string }[];
      };
    } catch {
      return { items: [], trash: [] };
    }
  });

  async setSessionState(key: Uint8Array, vaultJson: string) {
    if (typeof browser !== 'undefined' && browser.storage?.session) {
      await browser.storage.session.set({
        session_key: Array.from(key),
        session_vault_json: vaultJson,
      });
    }
  }

  async clearSessionState() {
    if (typeof browser !== 'undefined' && browser.storage?.session) {
      await browser.storage.session.remove([
        'session_key',
        'session_vault_json',
      ]);
    }
  }

  async checkSessionState() {
    if (typeof browser !== 'undefined' && browser.storage?.session) {
      try {
        const sessionData = await browser.storage.session.get([
          'session_key',
          'session_vault_json',
        ]);
        if (sessionData.session_key && sessionData.session_vault_json) {
          this.masterKey = new Uint8Array(sessionData.session_key as number[]);
          this.vaultJson = sessionData.session_vault_json as string;
          this.isUnlocked = true;
        }
      } catch (e) {
        console.error('Failed to read session state:', e);
      }
    }
  }

  async initWasm() {
    if (this.wasmReady) return;
    try {
      const wasmUrl =
        typeof browser !== 'undefined' && browser.runtime?.getURL
          ? browser.runtime.getURL('vault_core_bg.wasm' as never)
          : '/vault_core_bg.wasm';
      await init({ module_or_path: wasmUrl });

      await this.checkRegistration();
      await this.checkSessionState();
      this.wasmReady = true;
    } catch (e) {
      const errMsg = e instanceof Error ? e.message : String(e);
      this.error = `Failed to load cryptography engine: ${errMsg}`;
    }
  }

  exportCsv(): string {
    if (!this.vaultJson) return '';
    return wasm_vault_export_csv(this.vaultJson);
  }

  async importCsv(csvText: string): Promise<boolean> {
    try {
      this.error = '';
      if (!this.masterKey || !this.vaultJson) {
        throw new Error('Vault must be unlocked to import.');
      }

      const currentTime = new Date().toISOString();
      const updatedVault = wasm_vault_import_csv(
        this.vaultJson,
        csvText,
        currentTime,
      );

      await this.saveVault(updatedVault);
      return true;
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
      return false;
    }
  }

  async verifyPassword(password: string): Promise<boolean> {
    try {
      if (typeof browser !== 'undefined' && browser.storage?.local) {
        const data = await browser.storage.local.get([
          'vault_salt',
          'vault_payload',
        ]);
        if (!data.vault_salt || !data.vault_payload) {
          return false;
        }

        const salt = new Uint8Array(data.vault_salt as number[]);
        const payload = new Uint8Array(data.vault_payload as number[]);

        const key = wasm_derive_key(password, salt);
        wasm_decrypt_vault(key, payload);
        return true;
      }
      return false;
    } catch {
      return false;
    }
  }

  async checkRegistration() {
    if (typeof browser !== 'undefined' && browser.storage?.local) {
      const data = await browser.storage.local.get(['vault_salt']);
      this.isRegistered = !!data.vault_salt;
    }
  }

  async register(password: string) {
    try {
      this.error = '';
      if (password.length < 8) {
        this.error = 'Password must be at least 8 characters.';
        return false;
      }

      const salt = new Uint8Array(16);
      crypto.getRandomValues(salt);

      const key = wasm_derive_key(password, salt);
      const newVault = wasm_vault_new();

      const encoder = new TextEncoder();
      const encrypted = wasm_encrypt_vault(key, encoder.encode(newVault));

      if (typeof browser !== 'undefined' && browser.storage?.local) {
        await browser.storage.local.set({
          vault_salt: Array.from(salt),
          vault_payload: Array.from(encrypted),
        });
      }

      this.masterKey = key;
      this.vaultJson = newVault;
      this.isRegistered = true;
      this.isUnlocked = true;
      await this.setSessionState(key, newVault);
      return true;
    } catch (e) {
      const errMsg = e instanceof Error ? e.message : String(e);
      this.error = `Registration failed: ${errMsg}`;
      return false;
    }
  }

  async unlock(password: string) {
    try {
      this.error = '';
      if (!password) {
        this.error = 'Master password required.';
        return false;
      }

      if (typeof browser !== 'undefined' && browser.storage?.local) {
        const data = await browser.storage.local.get([
          'vault_salt',
          'vault_payload',
        ]);
        if (!data.vault_salt || !data.vault_payload) {
          this.error = 'Vault not found. Please register.';
          return false;
        }

        const salt = new Uint8Array(data.vault_salt as number[]);
        const payload = new Uint8Array(data.vault_payload as number[]);

        const key = wasm_derive_key(password, salt);
        const plaintextBytes = wasm_decrypt_vault(key, payload);
        const decoder = new TextDecoder();
        const decryptedJson = decoder.decode(plaintextBytes);

        this.masterKey = key;
        this.vaultJson = decryptedJson;
        this.isUnlocked = true;
        await this.setSessionState(key, decryptedJson);
        this.sync(); // Silent background sync!
        return true;
      }
      this.error = 'Browser storage is not available.';
      return false;
    } catch {
      this.error = 'Incorrect master password.';
      return false;
    }
  }

  lock() {
    this.masterKey = null;
    this.vaultJson = '';
    this.isUnlocked = false;
    this.clearSessionState();
  }

  async resetVault() {
    try {
      this.error = '';
      if (typeof browser !== 'undefined' && browser.storage?.local) {
        await browser.storage.local.remove(['vault_salt', 'vault_payload']);
      }
      this.masterKey = null;
      this.vaultJson = '';
      this.isRegistered = false;
      this.isUnlocked = false;

      // Clear Google Drive sync & login state
      const provider = getStorageProvider('google');
      if (provider) {
        try {
          await provider.signOut();
        } catch {
          // Ignore signout errors on reset
        }
      }
      this.syncNeedsPassword = false;
      this.pendingRemoteSalt = null;
      this.pendingRemotePayload = null;
      this.pendingRemoteMetadata = null;

      await this.clearSessionState();
      return true;
    } catch (e) {
      const errMsg = e instanceof Error ? e.message : String(e);
      this.error = `Reset failed: ${errMsg}`;
      return false;
    }
  }

  async saveVault(updatedVault: string) {
    if (!this.masterKey) return;
    const encoder = new TextEncoder();
    const encrypted = wasm_encrypt_vault(
      this.masterKey,
      encoder.encode(updatedVault),
    );

    if (typeof browser !== 'undefined' && browser.storage?.local) {
      await browser.storage.local.set({
        vault_payload: Array.from(encrypted),
      });
    }
    if (typeof browser !== 'undefined' && browser.storage?.session) {
      await browser.storage.session.set({
        session_vault_json: updatedVault,
      });
    }
    this.vaultJson = updatedVault;
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
  ) {
    try {
      this.error = '';
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

        const updatedVault = wasm_vault_add_entry(
          this.vaultJson,
          JSON.stringify(note),
        );
        await this.saveVault(updatedVault);
      } else {
        const targetUrl = url?.trim() || '';
        const baseDomain = getBaseDomain(targetUrl) || title.trim();

        const currentVault = JSON.parse(this.vaultJson);
        const existingGroupIndex = (currentVault.items || []).findIndex(
          (i: VaultItem) =>
            i.type === 'DomainGroup' &&
            i.title.toLowerCase() === baseDomain.toLowerCase(),
        );

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

        let updatedVault: string;
        if (existingGroupIndex >= 0) {
          const group = currentVault.items[existingGroupIndex];
          group.accounts.push(newAccount);
          if (tags && tags.length > 0) {
            group.tags = Array.from(new Set([...(group.tags || []), ...tags]));
          }
          if (targetUrl && !group.urls.includes(targetUrl)) {
            group.urls.push(targetUrl);
          }
          group.updated_at = currentTime;
          updatedVault = wasm_vault_update_entry(
            this.vaultJson,
            JSON.stringify(group),
          );
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
          updatedVault = wasm_vault_add_entry(
            this.vaultJson,
            JSON.stringify(newGroup),
          );
        }

        await this.saveVault(updatedVault);
      }

      this.sync(); // Silent background sync!
      return true;
    } catch (e) {
      const errMsg = e instanceof Error ? e.message : String(e);
      this.error = `Failed to add entry: ${errMsg}`;
      return false;
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
  ) {
    try {
      this.error = '';
      const currentVault = JSON.parse(this.vaultJson);
      const currentTime = new Date().toISOString();

      if (entryType === 'SecureNote') {
        const snIdx = (currentVault.items || []).findIndex(
          (item: VaultItem) => item.type === 'SecureNote' && item.id === id,
        );
        if (snIdx === -1) {
          throw new Error('Secure note not found');
        }
        const existing = currentVault.items[snIdx];
        const updatedNote = {
          ...existing,
          title: title.trim(),
          notes: notes?.trim() || '',
          tags: tags || [],
          updated_at: currentTime,
        };
        const updatedVault = wasm_vault_update_entry(
          this.vaultJson,
          JSON.stringify(updatedNote),
        );
        await this.saveVault(updatedVault);
      } else {
        // It's an Account ID. Find the DomainGroup containing this account.
        let foundGroupIndex = -1;
        let foundAccountIndex = -1;
        for (let i = 0; i < (currentVault.items || []).length; i++) {
          const item = currentVault.items[i];
          if (item.type === 'DomainGroup') {
            const accIdx = item.accounts.findIndex((a: Account) => a.id === id);
            if (accIdx !== -1) {
              foundGroupIndex = i;
              foundAccountIndex = accIdx;
              break;
            }
          }
        }

        if (foundGroupIndex === -1) {
          throw new Error('Account not found');
        }

        const group = currentVault.items[foundGroupIndex];
        const existingAccount = group.accounts[foundAccountIndex];

        const oldPassword = existingAccount.password;
        const newPassword = password || '';
        const passwordHistory = existingAccount.password_history
          ? [...existingAccount.password_history]
          : [];
        if (oldPassword && oldPassword !== newPassword) {
          passwordHistory.push({
            password: oldPassword,
            changed_at: currentTime,
          });
        }

        group.accounts[foundAccountIndex] = {
          ...existingAccount,
          username: username?.trim() || '',
          password: newPassword,
          totp_secret: totpSecret?.trim() || null,
          notes: notes?.trim() || '',
          password_history: passwordHistory,
          updated_at: currentTime,
        };

        const targetUrl = url?.trim() || '';
        const baseDomain = getBaseDomain(targetUrl) || title.trim();

        if (tags) {
          group.tags = tags;
        }
        if (targetUrl && !group.urls.includes(targetUrl)) {
          group.urls.push(targetUrl);
        }
        group.title = baseDomain;
        group.updated_at = currentTime;

        const updatedVault = wasm_vault_update_entry(
          this.vaultJson,
          JSON.stringify(group),
        );
        await this.saveVault(updatedVault);
      }
      this.sync(); // Silent background sync!
      return true;
    } catch (e) {
      const errMsg = e instanceof Error ? e.message : String(e);
      this.error = `Failed to update entry: ${errMsg}`;
      return false;
    }
  }

  async deleteEntry(id: string) {
    try {
      this.error = '';
      const updatedVault = wasm_vault_delete_entry(
        this.vaultJson,
        id,
        new Date().toISOString(),
      );
      await this.saveVault(updatedVault);
      this.sync(); // Silent background sync!
      return true;
    } catch (e) {
      const errMsg = e instanceof Error ? e.message : String(e);
      this.error = `Failed to delete entry: ${errMsg}`;
      return false;
    }
  }

  async deleteAccount(groupId: string, accountId: string) {
    try {
      this.error = '';
      const currentVault = JSON.parse(this.vaultJson);
      const currentTime = new Date().toISOString();

      const groupIdx = (currentVault.items || []).findIndex(
        (i: VaultItem) => i.type === 'DomainGroup' && i.id === groupId,
      );
      if (groupIdx === -1) {
        throw new Error('Group not found');
      }

      const group = currentVault.items[groupIdx];
      group.accounts = (group.accounts || []).filter(
        (a: Account) => a.id !== accountId,
      );

      let updatedVault: string;
      if (group.accounts.length === 0) {
        updatedVault = wasm_vault_delete_entry(
          this.vaultJson,
          groupId,
          currentTime,
        );
      } else {
        group.updated_at = currentTime;
        updatedVault = wasm_vault_update_entry(
          this.vaultJson,
          JSON.stringify(group),
        );
      }

      // Record a tombstone for the account ID to prevent resurrection
      updatedVault = wasm_vault_purge_entry(
        updatedVault,
        accountId,
        currentTime,
      );

      await this.saveVault(updatedVault);
      this.sync();
      return true;
    } catch (e) {
      const errMsg = e instanceof Error ? e.message : String(e);
      this.error = `Failed to delete account: ${errMsg}`;
      return false;
    }
  }

  async restoreEntry(id: string) {
    try {
      this.error = '';
      const updatedVault = wasm_vault_restore_entry(
        this.vaultJson,
        id,
        new Date().toISOString(),
      );
      await this.saveVault(updatedVault);
      this.sync(); // Silent background sync!
      return true;
    } catch (e) {
      const errMsg = e instanceof Error ? e.message : String(e);
      this.error = `Failed to restore entry: ${errMsg}`;
      return false;
    }
  }

  async purgeEntry(id: string) {
    try {
      this.error = '';
      const updatedVault = wasm_vault_purge_entry(
        this.vaultJson,
        id,
        new Date().toISOString(),
      );
      await this.saveVault(updatedVault);
      this.sync(); // Silent background sync!
      return true;
    } catch (e) {
      const errMsg = e instanceof Error ? e.message : String(e);
      this.error = `Failed to purge entry: ${errMsg}`;
      return false;
    }
  }

  generateCredential(config: Record<string, unknown>) {
    try {
      const resultJson = wasm_generate(JSON.stringify(config));
      return JSON.parse(resultJson);
    } catch (e) {
      console.error('Failed to generate credential:', e);
      return null;
    }
  }

  generateTotp(secret: string, timestamp: number) {
    try {
      return wasm_generate_totp(secret, BigInt(timestamp));
    } catch (e) {
      console.error('Failed to generate TOTP:', e);
      return '';
    }
  }

  async syncAndMerge(remoteVaultJson: string) {
    try {
      const mergedVaultJson = wasm_vault_merge(this.vaultJson, remoteVaultJson);
      await this.saveVault(mergedVaultJson);
      return true;
    } catch (e) {
      console.error('Failed to merge vault:', e);
      return false;
    }
  }

  async resolveSyncSaltMismatch(password: string): Promise<boolean> {
    try {
      this.error = '';
      if (!this.pendingRemoteSalt || !this.pendingRemotePayload) {
        this.error = 'No pending sync data.';
        return false;
      }

      // Derive the remote key using the remote salt
      const remoteKey = wasm_derive_key(
        password,
        this.pendingRemoteSalt,
      ) as Uint8Array;

      // Attempt decryption with remote key
      const decryptedBytes = wasm_decrypt_vault_packaged(
        remoteKey,
        this.pendingRemotePayload,
      );
      const decoder = new TextDecoder();
      const remoteVaultJson = decoder.decode(decryptedBytes);

      // Merge remote and local JSON
      const mergedVaultJson = wasm_vault_merge(this.vaultJson, remoteVaultJson);

      // Encrypt merged JSON using remote key and remote salt for GDrive packaging
      const encoder = new TextEncoder();
      const encrypted = wasm_encrypt_vault_packaged(
        remoteKey,
        this.pendingRemoteSalt,
        encoder.encode(mergedVaultJson),
      );

      // Overwrite local credentials and salt to match remote
      if (typeof browser !== 'undefined' && browser.storage?.local) {
        const localEncrypted = wasm_encrypt_vault(
          remoteKey,
          encoder.encode(mergedVaultJson),
        );
        await browser.storage.local.set({
          vault_salt: Array.from(this.pendingRemoteSalt),
          vault_payload: Array.from(localEncrypted),
        });
      }
      if (typeof browser !== 'undefined' && browser.storage?.session) {
        await browser.storage.session.set({
          session_key: Array.from(remoteKey),
          session_vault_json: mergedVaultJson,
        });
      }
      this.masterKey = remoteKey;
      this.vaultJson = mergedVaultJson;

      // Upload to Google Drive
      const provider = getStorageProvider('google');
      if (provider) {
        await provider.uploadVault(
          encrypted,
          this.pendingRemoteMetadata || undefined,
        );
      }

      // Reset mismatch state
      this.syncNeedsPassword = false;
      this.pendingRemoteSalt = null;
      this.pendingRemotePayload = null;
      this.pendingRemoteMetadata = null;
      return true;
    } catch (e) {
      const errMsg = e instanceof Error ? e.message : String(e);
      this.error = `Sync validation failed: ${errMsg}`;
      return false;
    }
  }

  cancelSyncMismatch() {
    this.syncNeedsPassword = false;
    this.pendingRemoteSalt = null;
    this.pendingRemotePayload = null;
    this.pendingRemoteMetadata = null;
    this.error = 'Sync cancelled.';
  }

  async backupLocal(): Promise<boolean> {
    try {
      this.error = '';
      if (!this.masterKey || !this.vaultJson) {
        this.error = 'Vault must be unlocked to backup.';
        return false;
      }

      const provider = getStorageProvider('google');
      if (!provider) return false;

      const authenticated = await provider.signIn(false);
      if (!authenticated) {
        this.error = 'Not signed in to Google.';
        return false;
      }

      // Read local salt to package the file
      const localData = await browser.storage.local.get(['vault_salt']);
      if (!localData.vault_salt) {
        throw new Error('Local salt not found.');
      }
      const localSalt = new Uint8Array(localData.vault_salt as number[]);

      const encoder = new TextEncoder();
      const driveFile = wasm_encrypt_vault_packaged(
        this.masterKey,
        localSalt,
        encoder.encode(this.vaultJson),
      );

      // Fetch existing remote file ID (metadata) if any, to update instead of duplicate
      const result = await provider.downloadVault();
      const metadata = result?.metadata;

      await provider.uploadVault(driveFile, metadata);
      return true;
    } catch (e) {
      const errMsg = e instanceof Error ? e.message : String(e);
      this.error = `Backup failed: ${errMsg}`;
      return false;
    }
  }

  async signInAndFetchRemoteVault(): Promise<{
    exists: boolean;
    error?: string;
  }> {
    try {
      this.error = '';
      const provider = getStorageProvider('google');
      if (!provider) {
        return { exists: false, error: 'Storage provider not available.' };
      }

      // 1. Initialize Google Drive sign-in (interactive = true)
      const authenticated = await provider.signIn(true);
      if (!authenticated) {
        return { exists: false, error: 'Google Drive authentication failed.' };
      }

      // 2. Search Google Drive's appDataFolder and download payload
      const result = await provider.downloadVault();
      if (!result) {
        // No remote vault exists
        return { exists: false };
      }

      const { payload, metadata } = result;

      // 3. Extract remote salt (KV01 header: bytes 4 to 20)
      let remoteSalt: Uint8Array;
      try {
        remoteSalt = wasm_vault_extract_salt(payload) as Uint8Array;
      } catch {
        if (payload.length >= 20) {
          remoteSalt = payload.slice(4, 20);
        } else {
          return { exists: false, error: 'Invalid remote vault file format.' };
        }
      }

      // 4. Set pending state values
      this.pendingRemoteSalt = remoteSalt;
      this.pendingRemotePayload = payload;
      this.pendingRemoteMetadata = metadata;
      this.syncNeedsPassword = true;

      return { exists: true };
    } catch (e) {
      const errMsg = e instanceof Error ? e.message : String(e);
      this.error = `Failed to fetch remote vault: ${errMsg}`;
      return { exists: false, error: errMsg };
    }
  }

  async restoreRemote(password?: string): Promise<boolean> {
    try {
      this.error = '';
      const provider = getStorageProvider('google');
      if (!provider) return false;

      const authenticated = await provider.signIn(false);
      if (!authenticated) {
        this.error = 'Not signed in to Google.';
        return false;
      }

      const result = await provider.downloadVault();
      if (!result) {
        this.error = 'No remote backup found on Google Drive.';
        return false;
      }

      const { payload, metadata } = result;

      // Extract remote salt. Since we only maintain the new system, we expect it to exist.
      let remoteSalt: Uint8Array;
      try {
        remoteSalt = wasm_vault_extract_salt(payload) as Uint8Array;
      } catch {
        this.error = 'Invalid Google Drive backup file format.';
        return false;
      }

      // Check salt mismatch against local
      const localData = await browser.storage.local.get(['vault_salt']);
      const localSalt = localData.vault_salt
        ? new Uint8Array(localData.vault_salt as number[])
        : null;

      let saltMismatch = false;
      if (localSalt) {
        for (let i = 0; i < 16; i++) {
          if (remoteSalt[i] !== localSalt[i]) {
            saltMismatch = true;
            break;
          }
        }
      } else {
        // No local salt means we're a new device or have been reset
        saltMismatch = true;
      }

      if (saltMismatch && !password) {
        // Flag that we need password entry to proceed with restoration
        this.pendingRemoteSalt = remoteSalt;
        this.pendingRemotePayload = payload;
        this.pendingRemoteMetadata = metadata;
        this.syncNeedsPassword = true;
        this.error =
          'Key mismatch. Master password required to decrypt backup.';
        return false;
      }

      // Determine the key to use for decryption
      let keyToUse: Uint8Array | null = this.masterKey;
      const activeSalt: Uint8Array = remoteSalt; // Always use the remote salt for restoring database

      if (saltMismatch && password) {
        keyToUse = wasm_derive_key(password, remoteSalt) as Uint8Array;
      }

      if (!keyToUse) {
        this.error = 'Vault is locked. Cannot decrypt.';
        return false;
      }

      // Decrypt using our Rust helper
      let remoteVaultJson: string;
      try {
        const decryptedBytes = wasm_decrypt_vault_packaged(keyToUse, payload);
        const decoder = new TextDecoder();
        remoteVaultJson = decoder.decode(decryptedBytes);
      } catch {
        // If decryption fails, it's either an incorrect password or mismatch
        if (saltMismatch && password) {
          this.error = 'Incorrect remote master password.';
        } else {
          // If we tried using local masterKey on a mismatch (like localSalt was missing)
          this.pendingRemoteSalt = remoteSalt;
          this.pendingRemotePayload = payload;
          this.pendingRemoteMetadata = metadata;
          this.syncNeedsPassword = true;
          this.error =
            'Key mismatch. Master password required to decrypt backup.';
        }
        return false;
      }

      // Overwrite local state completely
      if (typeof browser !== 'undefined' && browser.storage?.local) {
        const encoder = new TextEncoder();
        const encryptedLocal = wasm_encrypt_vault(
          keyToUse,
          encoder.encode(remoteVaultJson),
        );

        await browser.storage.local.set({
          vault_salt: Array.from(activeSalt),
          vault_payload: Array.from(encryptedLocal),
        });
      }

      if (typeof browser !== 'undefined' && browser.storage?.session) {
        await browser.storage.session.set({
          session_key: Array.from(keyToUse),
          session_vault_json: remoteVaultJson,
        });
      }

      this.masterKey = keyToUse;
      this.vaultJson = remoteVaultJson;
      this.isUnlocked = true;
      this.isRegistered = true;

      // Clear pending state
      this.syncNeedsPassword = false;
      this.pendingRemoteSalt = null;
      this.pendingRemotePayload = null;
      this.pendingRemoteMetadata = null;

      return true;
    } catch (e) {
      const errMsg = e instanceof Error ? e.message : String(e);
      this.error = `Restore failed: ${errMsg}`;
      return false;
    }
  }

  async sync() {
    try {
      this.error = '';
      if (!this.masterKey || !this.vaultJson) {
        this.error = 'Vault must be unlocked to sync.';
        return false;
      }

      const provider = getStorageProvider('google');
      if (!provider) return false;

      const authenticated = await provider.signIn(false);
      if (!authenticated) {
        this.error = 'Not signed in to Google.';
        return false;
      }

      const result = await provider.downloadVault();
      if (!result) {
        // No remote vault found. Upload local vault as initial copy!
        const encoder = new TextEncoder();
        const localData = await browser.storage.local.get(['vault_salt']);
        if (!localData.vault_salt) {
          throw new Error('Local salt not found.');
        }
        const localSalt = new Uint8Array(localData.vault_salt as number[]);
        const driveFile = wasm_encrypt_vault_packaged(
          this.masterKey,
          localSalt,
          encoder.encode(this.vaultJson),
        );

        await provider.uploadVault(driveFile);
        return true;
      }

      const { payload, metadata } = result;

      // Extract remote salt. Strict check.
      let remoteSalt: Uint8Array;
      try {
        remoteSalt = wasm_vault_extract_salt(payload) as Uint8Array;
      } catch {
        this.error = 'Invalid Google Drive backup file format.';
        return false;
      }

      // Check salt mismatch
      const localData = await browser.storage.local.get(['vault_salt']);
      const localSalt = localData.vault_salt
        ? new Uint8Array(localData.vault_salt as number[])
        : null;

      let saltMismatch = false;
      if (localSalt) {
        for (let i = 0; i < 16; i++) {
          if (remoteSalt[i] !== localSalt[i]) {
            saltMismatch = true;
            break;
          }
        }
      } else {
        saltMismatch = true;
      }

      if (saltMismatch) {
        this.pendingRemoteSalt = remoteSalt;
        this.pendingRemotePayload = payload;
        this.pendingRemoteMetadata = metadata;
        this.syncNeedsPassword = true;
        this.error = 'Key mismatch. Authentication required.';
        return false;
      }

      // Decrypt remote payload using local derived key
      let remoteVaultJson: string;
      try {
        const decryptedBytes = wasm_decrypt_vault_packaged(
          this.masterKey,
          payload,
        );
        const decoder = new TextDecoder();
        remoteVaultJson = decoder.decode(decryptedBytes);
      } catch {
        // Any decryption error on auto-sync is handled as a key mismatch
        this.pendingRemoteSalt = remoteSalt;
        this.pendingRemotePayload = payload;
        this.pendingRemoteMetadata = metadata;
        this.syncNeedsPassword = true;
        this.error = 'Key mismatch. Authentication required.';
        return false;
      }

      // Merge local and remote JSON via wasm_vault_merge()
      const mergedVaultJson = wasm_vault_merge(this.vaultJson, remoteVaultJson);

      // Encrypt reconciled database and save local
      const encoder = new TextEncoder();
      const encryptedLocal = wasm_encrypt_vault(
        this.masterKey,
        encoder.encode(mergedVaultJson),
      );

      // Prepare drive file with magic and salt
      const driveFile = wasm_encrypt_vault_packaged(
        this.masterKey,
        remoteSalt, // Use remote salt
        encoder.encode(mergedVaultJson),
      );

      // Save local
      if (typeof browser !== 'undefined' && browser.storage?.local) {
        await browser.storage.local.set({
          vault_payload: Array.from(encryptedLocal),
        });
      }
      if (typeof browser !== 'undefined' && browser.storage?.session) {
        await browser.storage.session.set({
          session_vault_json: mergedVaultJson,
        });
      }
      this.vaultJson = mergedVaultJson;

      // Upload updated payload back to Google Drive
      await provider.uploadVault(driveFile, metadata);
      return true;
    } catch (e) {
      const errMsg = e instanceof Error ? e.message : String(e);
      this.error = `Sync failed: ${errMsg}`;
      return false;
    }
  }
}

export const vaultState = new VaultState();
