/**
 * Unit tests for VaultState frontend state class.
 *
 * Strategy: stub window.__TAURI__.core.invoke before importing vault-state
 * so the constructor's async IPC calls resolve immediately (returning safe
 * defaults), then exercise pure computed logic and state-mutating methods
 * without requiring a real Tauri runtime.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { Vault, VaultItem } from './types.js';

// ---------------------------------------------------------------------------
// Tauri IPC mock helpers
// ---------------------------------------------------------------------------

// type InvokeHandler = (cmd: string, args?: Record<string, unknown>) => unknown;

function makeIpc(handlers: Partial<Record<string, (args?: any) => any>>) {
  return vi.fn((cmd: string, args?: Record<string, unknown>) => {
    const handler = handlers[cmd];
    if (handler) return Promise.resolve(handler(args));
    return Promise.resolve(undefined);
  });
}

function stubTauri(invokeFn: any) {
  (window as any).__TAURI__ = { core: { invoke: invokeFn } };
}

function clearTauri() {
  delete (window as any).__TAURI__;
}

// ---------------------------------------------------------------------------
// Test data helpers
// ---------------------------------------------------------------------------

function makeVault(overrides: Partial<Vault> = {}): Vault {
  return {
    items: [],
    trash: [],
    tombstones: [],
    ...overrides,
  };
}

function makeDomainGroup(
  id: string,
  title: string,
  tags: string[] = [],
  urls: string[] = [],
): VaultItem {
  return {
    type: 'DomainGroup',
    id,
    title,
    urls,
    accounts: [],
    tags,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
  };
}

function makeSecureNote(
  id: string,
  title: string,
  notes = '',
  tags: string[] = [],
): VaultItem {
  return {
    type: 'SecureNote',
    id,
    title,
    notes,
    tags,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
  };
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('VaultState', () => {
  let vaultState: any;
  let invokeMock: ReturnType<typeof vi.fn>;

  beforeEach(async () => {
    // Default IPC handlers — constructor makes these calls
    invokeMock = makeIpc({
      check_vault_exists: () => false,
      is_biometrics_supported: () => false,
    });
    stubTauri(invokeMock);

    // Dynamically import to ensure each test gets a fresh module context
    const mod = await import('./vault-state.svelte.js');
    // We instantiate directly because the exported singleton already ran.
    // Use the class constructor via the module re-import trick.
    const VaultStateForTest = (mod as any).VaultStateForTest;
    vaultState = VaultStateForTest ? new VaultStateForTest() : null;

    // If VaultStateForTest doesn't exist, fall back to directly accessing class
    if (!vaultState) {
      // Create via indirect class access from module (Svelte 5 exports)
      // We reuse the exported vaultState singleton instead and reset its state
      vaultState = mod.vaultState;
      // Reset key fields for test isolation
      vaultState.vault = null;
      vaultState.isUnlocked = false;
      vaultState.searchQuery = '';
      vaultState.activeTab = 'all';
      vaultState.selectedItem = null;
      vaultState.selectedTag = null;
      vaultState.error = '';
      vaultState.loading = false;
      vaultState.syncing = false;
      vaultState.syncError = '';
      vaultState.syncNeedsPassword = false;
      vaultState.pendingRemotePayload = null;
      vaultState.pendingRemoteMetadata = null;
      vaultState.pendingRemoteSalt = null;
      vaultState.gdriveAuthenticated = false;
      vaultState.gdriveEmail = '';
      vaultState.vaultExists = false;
    }

    invokeMock.mockClear();
  });

  afterEach(() => {
    clearTauri();
    vi.clearAllMocks();
  });

  // -------------------------------------------------------------------------
  describe('filteredItems — tab filtering', () => {
    it('returns all items when activeTab is "all"', () => {
      vaultState.vault = makeVault({
        items: [
          makeDomainGroup('g1', 'GitHub'),
          makeSecureNote('n1', 'My Note'),
        ],
      });
      vaultState.activeTab = 'all';

      expect(vaultState.filteredItems).toHaveLength(2);
    });

    it('returns only DomainGroups when activeTab is "logins"', () => {
      vaultState.vault = makeVault({
        items: [
          makeDomainGroup('g1', 'GitHub'),
          makeSecureNote('n1', 'My Note'),
        ],
      });
      vaultState.activeTab = 'logins';

      const result = vaultState.filteredItems;
      expect(result).toHaveLength(1);
      expect(result[0].type).toBe('DomainGroup');
    });

    it('returns only SecureNotes when activeTab is "notes"', () => {
      vaultState.vault = makeVault({
        items: [
          makeDomainGroup('g1', 'GitHub'),
          makeSecureNote('n1', 'My Note'),
        ],
      });
      vaultState.activeTab = 'notes';

      const result = vaultState.filteredItems;
      expect(result).toHaveLength(1);
      expect(result[0].type).toBe('SecureNote');
    });

    it('returns trash items when activeTab is "trash"', () => {
      vaultState.vault = makeVault({
        items: [makeDomainGroup('g1', 'GitHub')],
        trash: [makeSecureNote('tn1', 'Deleted Note')],
      });
      vaultState.activeTab = 'trash';

      const result = vaultState.filteredItems;
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe('tn1');
    });

    it('returns empty array when vault is null', () => {
      vaultState.vault = null;
      expect(vaultState.filteredItems).toEqual([]);
    });
  });

  // -------------------------------------------------------------------------
  describe('filteredItems — search query', () => {
    it('filters by title match (case insensitive)', () => {
      vaultState.vault = makeVault({
        items: [
          makeDomainGroup('g1', 'GitHub'),
          makeDomainGroup('g2', 'GitLab'),
          makeSecureNote('n1', 'Bank Credentials'),
        ],
      });
      vaultState.activeTab = 'all';
      vaultState.searchQuery = 'git';

      const result = vaultState.filteredItems;
      expect(result).toHaveLength(2);
      expect(result.map((i: VaultItem) => i.id).sort()).toEqual(['g1', 'g2']);
    });

    it('filters SecureNote by notes field', () => {
      vaultState.vault = makeVault({
        items: [
          makeSecureNote('n1', 'Note Alpha', 'secret PIN is 1234'),
          makeSecureNote('n2', 'Note Beta', 'no sensitive data'),
        ],
      });
      vaultState.activeTab = 'all';
      vaultState.searchQuery = 'PIN';

      const result = vaultState.filteredItems;
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe('n1');
    });

    it('returns all items when search query is blank', () => {
      vaultState.vault = makeVault({
        items: [makeDomainGroup('g1', 'GitHub'), makeSecureNote('n1', 'Note')],
      });
      vaultState.searchQuery = '   ';

      expect(vaultState.filteredItems).toHaveLength(2);
    });

    it('returns empty when nothing matches search', () => {
      vaultState.vault = makeVault({
        items: [makeDomainGroup('g1', 'GitHub')],
      });
      vaultState.searchQuery = 'zzznomatch';

      expect(vaultState.filteredItems).toHaveLength(0);
    });
  });

  // -------------------------------------------------------------------------
  describe('filteredItems — tag filter', () => {
    it('filters by selectedTag', () => {
      vaultState.vault = makeVault({
        items: [
          makeDomainGroup('g1', 'GitHub', ['work']),
          makeDomainGroup('g2', 'Netflix', ['personal']),
          makeSecureNote('n1', 'Note', '', ['work']),
        ],
      });
      vaultState.activeTab = 'all';
      vaultState.selectedTag = 'work';

      const result = vaultState.filteredItems;
      expect(result).toHaveLength(2);
      expect(result.map((i: VaultItem) => i.id).sort()).toEqual(['g1', 'n1']);
    });

    it('returns all when selectedTag is null', () => {
      vaultState.vault = makeVault({
        items: [
          makeDomainGroup('g1', 'GitHub', ['work']),
          makeSecureNote('n1', 'Note'),
        ],
      });
      vaultState.selectedTag = null;

      expect(vaultState.filteredItems).toHaveLength(2);
    });
  });

  // -------------------------------------------------------------------------
  describe('allTags', () => {
    it('collects unique tags from all items sorted alphabetically', () => {
      vaultState.vault = makeVault({
        items: [
          makeDomainGroup('g1', 'GitHub', ['work', 'coding']),
          makeSecureNote('n1', 'Note', '', ['personal', 'coding']),
        ],
      });
      vaultState.activeTab = 'all';

      expect(vaultState.allTags).toEqual(['coding', 'personal', 'work']);
    });

    it('returns empty array when vault is null', () => {
      vaultState.vault = null;
      expect(vaultState.allTags).toEqual([]);
    });

    it('returns empty array when items have no tags', () => {
      vaultState.vault = makeVault({
        items: [makeDomainGroup('g1', 'GitHub', [])],
      });
      expect(vaultState.allTags).toEqual([]);
    });
  });

  // -------------------------------------------------------------------------
  describe('unlock()', () => {
    it('sets isUnlocked = true and loads vault on success', async () => {
      const mockVault = makeVault({
        items: [makeSecureNote('n1', 'My Note')],
      });
      invokeMock = makeIpc({
        unlock_vault: () => undefined,
        get_vault: () => mockVault,
        check_vault_exists: () => true,
      });
      stubTauri(invokeMock);

      const result = await vaultState.unlock('correct-password');

      expect(result).toBe(true);
      expect(vaultState.isUnlocked).toBe(true);
      expect(vaultState.vault?.items).toHaveLength(1);
      expect(vaultState.error).toBe('');
    });

    it('sets error and returns false when unlock_vault throws', async () => {
      invokeMock = vi.fn((cmd: string) => {
        if (cmd === 'unlock_vault') {
          return Promise.reject({
            code: 'INVALID_PASSWORD',
            message: 'Wrong password',
          });
        }
        return Promise.resolve(undefined);
      });
      stubTauri(invokeMock);

      const result = await vaultState.unlock('wrong-password');

      expect(result).toBe(false);
      expect(vaultState.isUnlocked).toBe(false);
      expect(vaultState.error).toBe('Wrong password');
    });
  });

  // -------------------------------------------------------------------------
  describe('lock()', () => {
    it('clears isUnlocked, vault, and selectedItem', async () => {
      vaultState.isUnlocked = true;
      vaultState.vault = makeVault({
        items: [makeSecureNote('n1', 'Note')],
      });
      vaultState.selectedItem = makeSecureNote('n1', 'Note');

      invokeMock = makeIpc({
        lock_vault: () => undefined,
        check_vault_exists: () => false,
      });
      stubTauri(invokeMock);

      await vaultState.lock();

      expect(vaultState.isUnlocked).toBe(false);
      expect(vaultState.vault).toBeNull();
      expect(vaultState.selectedItem).toBeNull();
    });
  });

  // -------------------------------------------------------------------------
  describe('deleteItem()', () => {
    it('clears selectedItem if it matches deleted id', async () => {
      vaultState.isUnlocked = true;
      vaultState.vault = makeVault({
        items: [makeSecureNote('n1', 'Note')],
      });
      vaultState.selectedItem = makeSecureNote('n1', 'Note');

      const mockVault = makeVault({
        items: [],
        trash: [makeSecureNote('n1', 'Note')],
      });
      invokeMock = makeIpc({
        delete_vault_item: () => undefined,
        get_vault: () => mockVault,
      });
      stubTauri(invokeMock);

      await vaultState.deleteItem('n1');

      expect(vaultState.selectedItem).toBeNull();
    });

    it('does NOT clear selectedItem if a different item is deleted', async () => {
      vaultState.isUnlocked = true;
      const note1 = makeSecureNote('n1', 'Note 1');
      const note2 = makeSecureNote('n2', 'Note 2');
      vaultState.vault = makeVault({ items: [note1, note2] });
      vaultState.selectedItem = note1;

      const mockVault = makeVault({
        items: [note1],
        trash: [note2],
      });
      invokeMock = makeIpc({
        delete_vault_item: () => undefined,
        get_vault: () => mockVault,
      });
      stubTauri(invokeMock);

      await vaultState.deleteItem('n2');

      expect(vaultState.selectedItem?.id).toBe('n1');
    });
  });

  // -------------------------------------------------------------------------
  describe('loading flag management', () => {
    it('sets loading=true during unlock and false after', async () => {
      const loadingStates: boolean[] = [];
      let resolveUnlock!: () => void;
      const slowUnlock = new Promise<void>((r) => (resolveUnlock = r));

      invokeMock = vi.fn((cmd: string) => {
        if (cmd === 'unlock_vault') return slowUnlock.then(() => undefined);
        return Promise.resolve(undefined);
      });
      stubTauri(invokeMock);

      const promise = vaultState.unlock('pw');
      // Immediately after starting — should be loading
      loadingStates.push(vaultState.loading);
      resolveUnlock();
      await promise;
      loadingStates.push(vaultState.loading);

      expect(loadingStates[0]).toBe(true);
      expect(loadingStates[1]).toBe(false);
    });
  });
});
