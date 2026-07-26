import type { RemoteVaultMetadata } from '../types.js';

export interface StorageProvider {
  getProviderName(): string;
  signIn(interactive: boolean): Promise<boolean>;
  signOut(): Promise<void>;
  isAuthenticated(): Promise<boolean>;
  getUserInfo(): Promise<{ email: string; name?: string } | null>;
  downloadVault(): Promise<{
    payload: Uint8Array;
    metadata: RemoteVaultMetadata;
  } | null>;
  uploadVault(
    payload: Uint8Array,
    previousMetadata?: RemoteVaultMetadata,
  ): Promise<RemoteVaultMetadata>;
  deleteVault?(): Promise<boolean>;
}
