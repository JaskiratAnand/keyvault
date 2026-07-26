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

export interface Tombstone {
  id: string;
  deleted_at: string;
}

export interface Vault {
  items: VaultItem[];
  tombstones: Tombstone[];
  trash: VaultItem[];
}

export interface RemoteVaultMetadata {
  eTag: string;
  lastModified: string;
}
