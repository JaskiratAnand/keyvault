import type { RemoteVaultMetadata, StorageProvider } from './types.js';

export class GoogleDriveProvider implements StorageProvider {
  getProviderName(): string {
    return 'Google Drive';
  }

  private async getAccessToken(interactive: boolean): Promise<string | null> {
    try {
      if (typeof browser !== 'undefined' && browser.runtime?.getManifest) {
        const manifest = browser.runtime.getManifest();
        const clientId = manifest.oauth2?.client_id;
        if (!clientId || clientId.includes('dummy')) {
          return null;
        }
      }
      const auth = await browser.identity.getAuthToken({ interactive });
      return auth.token || null;
    } catch (err) {
      console.error('Failed to get OAuth token:', err);
      return null;
    }
  }

  async signIn(interactive: boolean): Promise<boolean> {
    const token = await this.getAccessToken(interactive);
    return !!token;
  }

  async signOut(): Promise<void> {
    try {
      if (typeof browser !== 'undefined' && browser.runtime?.getManifest) {
        const manifest = browser.runtime.getManifest();
        const clientId = manifest.oauth2?.client_id;
        if (!clientId || clientId.includes('dummy')) {
          return;
        }
      }
      const auth = await browser.identity.getAuthToken({ interactive: false });
      if (auth?.token) {
        await browser.identity.removeCachedAuthToken({ token: auth.token });
        // Revoke token via Google revoke URL
        await fetch(
          `https://oauth2.googleapis.com/revoke?token=${auth.token}`,
          { method: 'POST' },
        );
      }
    } catch (err) {
      console.error('Error signing out:', err);
    }
  }

  async isAuthenticated(): Promise<boolean> {
    const token = await this.getAccessToken(false);
    return !!token;
  }

  async getUserInfo(): Promise<{ email: string; name?: string } | null> {
    const token = await this.getAccessToken(false);
    if (!token) return null;
    try {
      const response = await fetch(
        'https://www.googleapis.com/oauth2/v2/userinfo',
        {
          headers: { Authorization: `Bearer ${token}` },
        },
      );
      if (!response.ok) return null;
      const data = await response.json();
      return { email: data.email, name: data.name };
    } catch {
      return null;
    }
  }

  async downloadVault(): Promise<{
    payload: Uint8Array;
    metadata: RemoteVaultMetadata;
  } | null> {
    const token = await this.getAccessToken(false);
    if (!token) return null;

    try {
      // 1. Search for vault.db in appDataFolder
      const searchUrl =
        "https://www.googleapis.com/drive/v3/files?spaces=appDataFolder&q=name='vault.db'&fields=files(id,mimeType,modifiedTime)&pageSize=1";
      const searchResponse = await fetch(searchUrl, {
        headers: { Authorization: `Bearer ${token}` },
      });
      if (!searchResponse.ok) return null;

      const searchData = await searchResponse.json();
      const file = searchData.files?.[0];
      if (!file) return null;

      // 2. Fetch the file content and its metadata
      const contentUrl = `https://www.googleapis.com/drive/v3/files/${file.id}?alt=media`;
      const metadataUrl = `https://www.googleapis.com/drive/v3/files/${file.id}?fields=id,name,mimeType,modifiedTime,headRevisionId`;

      const [contentRes, metadataRes] = await Promise.all([
        fetch(contentUrl, { headers: { Authorization: `Bearer ${token}` } }),
        fetch(metadataUrl, { headers: { Authorization: `Bearer ${token}` } }),
      ]);

      if (!contentRes.ok || !metadataRes.ok) return null;

      const arrayBuffer = await contentRes.arrayBuffer();
      const metadata = await metadataRes.json();
      const eTag =
        contentRes.headers.get('ETag') || metadata.headRevisionId || '';

      return {
        payload: new Uint8Array(arrayBuffer),
        metadata: {
          eTag,
          lastModified: metadata.modifiedTime || new Date().toISOString(),
        },
      };
    } catch (err) {
      console.error('Failed to download vault:', err);
      return null;
    }
  }

  async uploadVault(
    payload: Uint8Array,
    previousMetadata?: RemoteVaultMetadata,
  ): Promise<RemoteVaultMetadata> {
    const token = await this.getAccessToken(false);
    if (!token) throw new Error('Not authenticated');

    try {
      // 1. Search for existing file ID
      const searchUrl =
        "https://www.googleapis.com/drive/v3/files?spaces=appDataFolder&q=name='vault.db'&fields=files(id)&pageSize=1";
      const searchResponse = await fetch(searchUrl, {
        headers: { Authorization: `Bearer ${token}` },
      });
      if (!searchResponse.ok) throw new Error('Drive search failed');
      const searchData = await searchResponse.json();
      const fileId = searchData.files?.[0]?.id;

      let response: Response;
      if (fileId) {
        // 2. Perform a PATCH media upload
        const uploadUrl = `https://www.googleapis.com/upload/drive/v3/files/${fileId}?uploadType=media`;
        const headers: Record<string, string> = {
          Authorization: `Bearer ${token}`,
          'Content-Type': 'application/octet-stream',
        };
        // Optimistic concurrency check if we have a previous ETag
        if (previousMetadata?.eTag) {
          headers['If-Match'] = previousMetadata.eTag;
        }

        response = await fetch(uploadUrl, {
          method: 'PATCH',
          headers,
          body: new Blob([payload as BlobPart]),
        });
      } else {
        // 3. Perform a multipart creation upload (combining metadata and media)
        const uploadUrl =
          'https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart';
        const boundary = '-------314159265358979323846';

        const metadata = {
          name: 'vault.db',
          parents: ['appDataFolder'],
        };

        const delimiter = `\r\n--${boundary}\r\n`;
        const closeDelimiter = `\r\n--${boundary}--`;

        const header = `${delimiter}Content-Type: application/json; charset=UTF-8\r\n\r\n${JSON.stringify(metadata)}\r\n${delimiter}Content-Type: application/octet-stream\r\n\r\n`;

        const encoder = new TextEncoder();
        const headerBytes = encoder.encode(header);
        const footerBytes = encoder.encode(closeDelimiter);

        const body = new Uint8Array(
          headerBytes.length + payload.length + footerBytes.length,
        );
        body.set(headerBytes, 0);
        body.set(payload, headerBytes.length);
        body.set(footerBytes, headerBytes.length + payload.length);

        response = await fetch(uploadUrl, {
          method: 'POST',
          headers: {
            Authorization: `Bearer ${token}`,
            'Content-Type': `multipart/related; boundary=${boundary}`,
          },
          body: new Blob([body as BlobPart]),
        });
      }

      if (!response.ok) {
        throw new Error(`Upload failed with status: ${response.status}`);
      }

      const data = await response.json();
      const headRevisionId =
        data.headRevisionId || response.headers.get('ETag') || '';
      return {
        eTag: headRevisionId,
        lastModified: new Date().toISOString(),
      };
    } catch (err) {
      console.error('Failed to upload vault:', err);
      throw err;
    }
  }

  async deleteVault(): Promise<boolean> {
    const token = await this.getAccessToken(false);
    if (!token) return false;

    try {
      // 1. Search for existing file ID
      const searchUrl =
        "https://www.googleapis.com/drive/v3/files?spaces=appDataFolder&q=name='vault.db'&fields=files(id)&pageSize=1";
      const searchResponse = await fetch(searchUrl, {
        headers: { Authorization: `Bearer ${token}` },
      });
      if (!searchResponse.ok) return false;
      const searchData = await searchResponse.json();
      const fileId = searchData.files?.[0]?.id;

      if (fileId) {
        // 2. Send DELETE request
        const deleteUrl = `https://www.googleapis.com/drive/v3/files/${fileId}`;
        const deleteResponse = await fetch(deleteUrl, {
          method: 'DELETE',
          headers: { Authorization: `Bearer ${token}` },
        });
        return deleteResponse.ok;
      }
      return true; // No file to delete is considered a success
    } catch (err) {
      console.error('Failed to delete remote vault:', err);
      return false;
    }
  }
}
