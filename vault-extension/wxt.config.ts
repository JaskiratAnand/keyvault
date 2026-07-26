import fs from 'node:fs';
import path from 'node:path';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'wxt';

declare const process: {
  cwd: () => string;
  env: Record<string, string | undefined>;
};

function getEnvVariable(key: string): string {
  const val = process.env[key];
  if (val) return val;
  try {
    const envPath = path.resolve(process.cwd(), '.env');
    if (fs.existsSync(envPath)) {
      const content = fs.readFileSync(envPath, 'utf-8');
      for (const line of content.split('\n')) {
        const trimmed = line.trim();
        if (trimmed && !trimmed.startsWith('#') && trimmed.includes('=')) {
          const [k, ...v] = trimmed.split('=');
          if (k.trim() === key) {
            return v.join('=').trim();
          }
        }
      }
    }
  } catch {
    // Ignore read errors
  }
  return '';
}

const clientId = getEnvVariable('VITE_GDRIVE_CLIENT_ID');

// See https://wxt.dev/api/config.html
export default defineConfig({
  srcDir: 'src',
  modules: ['@wxt-dev/module-svelte'],
  vite: () => ({
    plugins: [tailwindcss()],
  }),
  manifest: {
    name: 'KeyVault',
    description: 'Secure password manager built with Rust & WebAssembly',
    permissions: ['storage', 'activeTab', 'scripting', 'identity'],
    content_security_policy: {
      extension_pages:
        "script-src 'self' 'wasm-unsafe-eval'; object-src 'self';",
    },
    ...(clientId
      ? {
          oauth2: {
            client_id: clientId,
            scopes: [
              'https://www.googleapis.com/auth/drive.appdata',
              'https://www.googleapis.com/auth/userinfo.email',
              'https://www.googleapis.com/auth/userinfo.profile',
            ],
          },
        }
      : {}),
    options_ui: {
      page: 'options.html',
      open_in_tab: true,
    },
    web_accessible_resources: [
      {
        resources: [
          'vault_core_bg.wasm',
          'inline-badge.html',
          'inline-dropdown.html',
        ],
        matches: ['<all_urls>'],
      },
    ],
    key: 'MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAxKsmEcF0IUx8nyEIaC5k/PNNi+SDC54+dO6cVR1SwHz3K6R8baElWUdhr2kN6/XYabDgCILvr61OF3nzbdQgj2gUSPL2tdaJgU+JLcouOQpItEG/2/tn0xUR+NnbWoKJMLhrjAAFSskVkNyBQtZToq5MSAsME52D9ydGyuQFz9I035D1HCBF6yyOZNifxAvctvJAZo4QY6DwZJAtEwGlOLot7iX8E+/GZ8gRP4Q5LjXMh347Kc9G0L5qT4OlNWXFN7WpeObuRbIWkft+4Q0jl4Ijx69OK6cwM/+XXLzIv+hAV/30yKvkoeAMpOuWAi3SW7DirFo+ISbfak7YhhsVKwIDAQAB',
    action: {
      default_icon: {
        16: 'icon/16.png',
        32: 'icon/32.png',
        48: 'icon/48.png',
        96: 'icon/96.png',
        128: 'icon/128.png',
        256: 'icon/256.png',
        512: 'icon/512.png',
      },
    },
  },
});
