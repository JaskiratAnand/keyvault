import { GoogleDriveProvider } from './google-drive.js';
import type { StorageProvider } from './types.js';

export function getStorageProvider(type: string): StorageProvider | null {
  switch (type.toLowerCase()) {
    case 'google':
      return new GoogleDriveProvider();
    default:
      return null;
  }
}
