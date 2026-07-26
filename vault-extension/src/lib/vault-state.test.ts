import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import init, {
  wasm_vault_add_entry,
  wasm_vault_delete_entry,
  wasm_vault_new,
} from 'vault-core';
import { beforeAll, describe, expect, it, vi } from 'vitest';
import { GoogleDriveProvider } from './sync/google-drive.js';
import { type VaultItem, vaultState } from './vault-state.svelte.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Mock browser global object for Node.js test environment
// biome-ignore lint/suspicious/noExplicitAny: Mocking browser global object for tests
(globalThis as any).browser = {
  storage: {
    local: {
      get: vi.fn(),
      set: vi.fn(),
      remove: vi.fn(),
    },
  },
  runtime: {
    getURL: vi.fn((path) => path),
    getManifest: vi.fn(() => ({
      oauth2: {
        client_id:
          '483984950348-dummyclientid1234567.apps.googleusercontent.com',
      },
    })),
  },
  identity: {
    getAuthToken: vi.fn(),
    removeCachedAuthToken: vi.fn(),
  },
  // biome-ignore lint/suspicious/noExplicitAny: Mocking browser global object for tests
} as any;

describe('VaultState & Sync Integration Tests', () => {
  beforeAll(async () => {
    // Read the WASM binary directly from local public assets folder in Node
    const wasmPath = path.resolve(__dirname, '../../public/vault_core_bg.wasm');
    const wasmBuffer = fs.readFileSync(wasmPath);
    await init({ module_or_path: wasmBuffer });
    vaultState.wasmReady = true;
    vaultState.masterKey = new Uint8Array(32);
  });

  it('should merge remote vault updates using Last-Write-Wins (LWW)', async () => {
    const groupId = 'group-uuid-123';
    const accountId = 'acc-uuid-123';
    const timestampLocal = '2026-07-03T10:00:00.000Z';
    const timestampRemote = '2026-07-03T11:00:00.000Z'; // Newer remote update

    const localItem: VaultItem = {
      type: 'DomainGroup',
      id: groupId,
      title: 'gmail.com',
      urls: ['https://gmail.com'],
      accounts: [
        {
          id: accountId,
          username: 'local_user',
          password: 'local_password',
          totp_secret: null,
          notes: '',
          custom_fields: [],
          password_history: [],
          created_at: timestampLocal,
          updated_at: timestampLocal,
        },
      ],
      tags: [],
      created_at: timestampLocal,
      updated_at: timestampLocal,
    };

    const remoteItem: VaultItem = {
      type: 'DomainGroup',
      id: groupId,
      title: 'gmail.com',
      urls: ['https://gmail.com'],
      accounts: [
        {
          id: accountId,
          username: 'local_user',
          password: 'remote_password',
          totp_secret: null,
          notes: '',
          custom_fields: [],
          password_history: [],
          created_at: timestampLocal,
          updated_at: timestampRemote,
        },
      ],
      tags: [],
      created_at: timestampLocal,
      updated_at: timestampRemote,
    };

    // Initialize local vault State
    const localVault = wasm_vault_add_entry(
      wasm_vault_new(),
      JSON.stringify(localItem),
    );
    vaultState.vaultJson = localVault;

    // Generate remote vault JSON
    const remoteVault = wasm_vault_add_entry(
      wasm_vault_new(),
      JSON.stringify(remoteItem),
    );

    // Execute sync merge
    const success = await vaultState.syncAndMerge(remoteVault);
    expect(success).toBe(true);

    // Verify remote (newer) changes were applied locally
    const parsed = JSON.parse(vaultState.vaultJson);
    const mergedItem = parsed.items.find((e: VaultItem) => e.id === groupId);
    expect(mergedItem.title).toBe('gmail.com');
    expect(mergedItem.accounts[0].password).toBe('remote_password');
    expect(mergedItem.updated_at).toBe(timestampRemote);
  });

  it('should propagate tombstones to delete credentials during sync merge', async () => {
    const groupId = 'group-uuid-123';
    const timestamp = '2026-07-03T10:00:00.000Z';
    const deletionTimestamp = '2026-07-03T11:00:00.000Z'; // Deleted later

    const item: VaultItem = {
      type: 'DomainGroup',
      id: groupId,
      title: 'gmail.com',
      urls: [],
      accounts: [],
      tags: [],
      created_at: timestamp,
      updated_at: timestamp,
    };

    // Initialize local vault containing the item
    const localVault = wasm_vault_add_entry(
      wasm_vault_new(),
      JSON.stringify(item),
    );
    vaultState.vaultJson = localVault;

    // Generate remote vault containing a deletion tombstone for that item
    let remoteVault = wasm_vault_add_entry(
      wasm_vault_new(),
      JSON.stringify(item),
    );
    remoteVault = wasm_vault_delete_entry(
      remoteVault,
      groupId,
      deletionTimestamp,
    );

    // Verify item is moved to trash on remote before merge
    const remoteParsed = JSON.parse(remoteVault);
    expect(remoteParsed.items.length).toBe(0);
    expect(remoteParsed.trash.length).toBe(1);
    expect(remoteParsed.tombstones.length).toBe(0);

    // Execute sync merge
    const success = await vaultState.syncAndMerge(remoteVault);
    expect(success).toBe(true);

    // Verify item was removed from active and added to trash locally
    const localParsed = JSON.parse(vaultState.vaultJson);
    expect(localParsed.items.length).toBe(0);
    expect(localParsed.trash.length).toBe(1);
    expect(localParsed.trash[0].id).toBe(groupId);
  });

  it('should resurrect a deleted entry if it was edited locally after the remote deletion', async () => {
    const groupId = 'group-uuid-123';
    const timestamp = '2026-07-03T10:00:00.000Z';
    const deletionTimestamp = '2026-07-03T11:00:00.000Z'; // Remote deletion
    const localEditTimestamp = '2026-07-03T12:00:00.000Z'; // Newer local edit

    const originalItem: VaultItem = {
      type: 'DomainGroup',
      id: groupId,
      title: 'gmail.com',
      urls: [],
      accounts: [],
      tags: [],
      created_at: timestamp,
      updated_at: timestamp,
    };

    const localEditedItem: VaultItem = {
      ...originalItem,
      title: 'resurrected.com',
      updated_at: localEditTimestamp,
    };

    // Initialize local vault containing the newer edit
    const localVault = wasm_vault_add_entry(
      wasm_vault_new(),
      JSON.stringify(localEditedItem),
    );
    vaultState.vaultJson = localVault;

    // Generate remote vault with deletion tombstone
    let remoteVault = wasm_vault_add_entry(
      wasm_vault_new(),
      JSON.stringify(originalItem),
    );
    remoteVault = wasm_vault_delete_entry(
      remoteVault,
      groupId,
      deletionTimestamp,
    );

    // Execute sync merge
    const success = await vaultState.syncAndMerge(remoteVault);
    expect(success).toBe(true);

    // Verify item is preserved/resurrected locally because local edit is newer
    const localParsed = JSON.parse(vaultState.vaultJson);
    const resurrected = localParsed.items.find(
      (e: VaultItem) => e.id === groupId,
    );
    expect(resurrected).toBeDefined();
    expect(resurrected.title).toBe('resurrected.com');
    expect(resurrected.updated_at).toBe(localEditTimestamp);
  });

  it('should bypass identity API calls if a dummy client_id is present', async () => {
    const provider = new GoogleDriveProvider();

    // Call getAccessToken which uses getManifest mockup returning dummy client_id
    // biome-ignore lint/suspicious/noExplicitAny: access private method for testing
    const token = await (provider as any).getAccessToken(false);

    // Verify it returns null and does not call identity API
    expect(token).toBeNull();
    expect(browser.identity.getAuthToken).not.toHaveBeenCalled();
  });

  it('should support updating an entry and recording password history', async () => {
    // Reset state
    vaultState.vaultJson = wasm_vault_new();

    // Add entry (will create group automatically)
    const addSuccess = await vaultState.addEntry(
      'google.com',
      'original_user',
      'password123',
      'https://google.com',
      'Initial notes',
      'Login',
      undefined,
      ['tag1'],
    );
    expect(addSuccess).toBe(true);

    const parsedGroup = JSON.parse(vaultState.vaultJson);
    expect(parsedGroup.items).toHaveLength(1);
    const group = parsedGroup.items[0];
    expect(group.accounts).toHaveLength(1);
    const accountId = group.accounts[0].id;

    // Update account with new password
    const editSuccess = await vaultState.updateEntry(
      accountId,
      'google.com',
      'original_user',
      'newpassword456',
      'https://google.com',
      'Updated notes',
      'Login',
      undefined,
      ['tag1', 'tag2'],
    );

    expect(editSuccess).toBe(true);

    // Verify updated details and password history
    const parsed = JSON.parse(vaultState.vaultJson);
    const updatedGroup = parsed.items[0];
    const updatedAcc = updatedGroup.accounts[0];
    expect(updatedAcc).toBeDefined();
    expect(updatedGroup.title).toBe('google.com');
    expect(updatedAcc.password).toBe('newpassword456');
    expect(updatedAcc.notes).toBe('Updated notes');
    expect(updatedGroup.tags).toEqual(['tag1', 'tag2']);
    expect(updatedAcc.password_history).toHaveLength(1);
    expect(updatedAcc.password_history[0].password).toBe('password123');
  });

  it('should sign in interactively and fetch the remote vault payload during setup', async () => {
    // Mock provider methods on prototype
    const originalSignIn = GoogleDriveProvider.prototype.signIn;
    const originalDownloadVault = GoogleDriveProvider.prototype.downloadVault;

    const dummyPayload = new Uint8Array([
      // KV01 magic header
      0x4b, 0x56, 0x30, 0x31,
      // 16-byte salt
      1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
      // cipher bytes...
      17, 18, 19, 20,
    ]);
    const dummyMetadata = {
      eTag: 'etag-123',
      lastModified: '2026-07-03T10:00:00.000Z',
    };

    GoogleDriveProvider.prototype.signIn = vi.fn().mockResolvedValue(true);
    GoogleDriveProvider.prototype.downloadVault = vi.fn().mockResolvedValue({
      payload: dummyPayload,
      metadata: dummyMetadata,
    });

    // Run the method
    const result = await vaultState.signInAndFetchRemoteVault();

    expect(result.exists).toBe(true);
    expect(GoogleDriveProvider.prototype.signIn).toHaveBeenCalledWith(true);
    expect(GoogleDriveProvider.prototype.downloadVault).toHaveBeenCalled();
    expect(vaultState.pendingRemoteSalt).toEqual(
      new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]),
    );
    expect(vaultState.pendingRemotePayload).toBe(dummyPayload);
    expect(vaultState.pendingRemoteMetadata).toBe(dummyMetadata);
    expect(vaultState.syncNeedsPassword).toBe(true);

    // Clean up mocks
    GoogleDriveProvider.prototype.signIn = originalSignIn;
    GoogleDriveProvider.prototype.downloadVault = originalDownloadVault;

    // Reset state mismatch values
    vaultState.cancelSyncMismatch();
  });

  it('should generate TOTP code successfully from Base32 secret', () => {
    // RFC 6238 vector: secret "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ" at T=1234567890 should produce "005924"
    const secret = 'GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ';
    const code = vaultState.generateTotp(secret, 1234567890);
    expect(code).toBe('005924');
  });

  it('should clear local storage and Google Drive sync state on resetVault', async () => {
    const originalSignOut = GoogleDriveProvider.prototype.signOut;
    GoogleDriveProvider.prototype.signOut = vi.fn().mockResolvedValue(undefined);

    vaultState.syncNeedsPassword = true;
    vaultState.pendingRemotePayload = new Uint8Array([1, 2, 3]);
    vaultState.pendingRemoteSalt = new Uint8Array([4, 5, 6]);

    const res = await vaultState.resetVault();

    expect(res).toBe(true);
    expect(GoogleDriveProvider.prototype.signOut).toHaveBeenCalled();
    expect(vaultState.syncNeedsPassword).toBe(false);
    expect(vaultState.pendingRemotePayload).toBeNull();
    expect(vaultState.pendingRemoteSalt).toBeNull();
    expect(browser.storage.local.remove).toHaveBeenCalledWith([
      'vault_salt',
      'vault_payload',
    ]);

    GoogleDriveProvider.prototype.signOut = originalSignOut;
  });
});
