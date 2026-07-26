<script lang="ts">
import {
  Download,
  Eye,
  EyeOff,
  FileSpreadsheet,
  ShieldCheck,
  Sparkles,
  Trash2,
  Upload,
} from 'lucide-svelte';
import { getErrorMsg } from '../lib/tauri-ipc.js';
import { getVaultContext } from '../lib/vault-state.svelte.js';

interface Props {
  currentTab: 'vault' | 'generator' | 'trash' | 'sync' | 'settings' | 'help';
}

let { currentTab = $bindable() }: Props = $props();

const vaultState = getVaultContext();

const isMac =
  typeof window !== 'undefined' && navigator.userAgent.includes('Mac');
const { invoke } = (window as any).__TAURI__?.core || {};

// Local Settings States
let customClientId = $state(
  localStorage.getItem('gdrive_custom_client_id') || '',
);

let showExportConfirm = $state(false);
let exportType = $state<'json' | 'csv' | null>(null);
let exportPassword = $state('');
let exportError = $state('');
let showExportPassword = $state(false);

let importSuccessMsg = $state('');
let importErrorMsg = $state('');

let bioError = $state('');

let showSettingsResetConfirm = $state(false);
let resetConfirmText = $state('');
let resetError = $state('');

function handleSaveCustomClientId() {
  localStorage.setItem('gdrive_custom_client_id', customClientId.trim());
  vaultState.checkGDriveAuth();
  alert('Custom Client ID saved. Refresh connection if signed in.');
}

function handleResetCustomClientId() {
  customClientId = '';
  localStorage.removeItem('gdrive_custom_client_id');
  vaultState.checkGDriveAuth();
  alert('Reset to default developer Client ID.');
}

function requestExport(type: 'json' | 'csv') {
  exportType = type;
  exportPassword = '';
  exportError = '';
  showExportPassword = false;
  showExportConfirm = true;
}

async function handleConfirmExport() {
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
    await executeExportCsv();
  }
  showExportConfirm = false;
  exportPassword = '';
  exportType = null;
}

function executeExportJson() {
  const dataStr = `data:text/json;charset=utf-8,${encodeURIComponent(
    JSON.stringify(vaultState.vault),
  )}`;
  const downloadAnchor = document.createElement('a');
  downloadAnchor.setAttribute('href', dataStr);
  downloadAnchor.setAttribute(
    'download',
    `keyvault-export-${new Date().toISOString().split('T')[0]}.json`,
  );
  document.body.appendChild(downloadAnchor);
  downloadAnchor.click();
  downloadAnchor.remove();
}

async function executeExportCsv() {
  try {
    const csvContent = await vaultState.exportCsv();
    const dataStr = `data:text/csv;charset=utf-8,${encodeURIComponent(csvContent)}`;
    const downloadAnchor = document.createElement('a');
    downloadAnchor.setAttribute('href', dataStr);
    downloadAnchor.setAttribute(
      'download',
      `keyvault-export-${new Date().toISOString().split('T')[0]}.csv`,
    );
    document.body.appendChild(downloadAnchor);
    downloadAnchor.click();
    downloadAnchor.remove();
  } catch (err) {
    console.error('Failed to export CSV:', err);
  }
}

async function handleNativeImportCsv() {
  importSuccessMsg = '';
  importErrorMsg = '';
  try {
    const currentTime = new Date().toISOString();
    const count = await vaultState.selectAndImportCsv(currentTime);
    importSuccessMsg = `Successfully imported ${count} credentials from CSV!`;
  } catch (err) {
    importErrorMsg = getErrorMsg(err);
  }
}

async function handleToggleBiometrics() {
  bioError = '';
  try {
    if (vaultState.bioEnabled) {
      localStorage.removeItem('biometrics_enabled');
      vaultState.bioEnabled = false;
    } else if (invoke) {
      const success = await invoke('authenticate_biometrics');
      if (success) {
        localStorage.setItem('biometrics_enabled', 'true');
        vaultState.bioEnabled = true;
      } else {
        bioError = 'Biometric verification failed.';
      }
    }
  } catch (err) {
    bioError = getErrorMsg(err);
  }
}

async function handleSettingsResetVault() {
  resetError = '';
  if (resetConfirmText !== 'RESET') {
    resetError = 'Confirmation text must match "RESET".';
    return;
  }
  const success = await vaultState.resetVault();
  if (success) {
    showSettingsResetConfirm = false;
    resetConfirmText = '';
    currentTab = 'vault';
  } else {
    resetError = vaultState.error || 'Reset failed.';
  }
}
</script>

<div class="grow overflow-y-auto p-8 space-y-6">
  <div class="max-w-xl mx-auto space-y-6 animate-fade-in">
    <div>
      <h2 class="text-lg font-bold text-[#fafafa] mb-1">Application Settings</h2>
      <p class="text-xs text-[#a1a1aa]">Configure OAuth credentials, backup exports, CSV imports, and security options.</p>
    </div>

    <div class="space-y-4">
      <!-- OAUTH CLIENT CREDENTIALS CARD -->
      <div class="p-4 rounded-lg bg-[#18181b] border border-[#27272a] space-y-3">
        <h3 class="text-xs font-semibold text-[#fafafa] uppercase tracking-wider flex items-center gap-2">
          <Sparkles class="w-3.5 h-3.5 text-[#06b6d4]" /> OAuth client credentials
        </h3>
        <p class="text-xs text-[#a1a1aa] leading-relaxed">
          KeyVault includes a developer OAuth client. Specify a custom Google OAuth Client ID if deploying a custom build.
        </p>
        <p class="text-[10px] text-amber-500/90 leading-relaxed font-sans mt-1">
          ⚠️ <strong>Requirement</strong>: The Client ID entered below must be registered as a <strong>Desktop application</strong> in Google Cloud Console. Web application Client IDs will fail because they require a Client Secret, which is not supported by the settings input (use Option B with a <code>.env</code> file rebuild instead).
        </p>
        <div class="flex gap-2">
          <input
            type="text"
            placeholder="Custom Client ID"
            bind:value={customClientId}
            class="grow bg-[#09090b] border border-[#27272a] rounded-lg px-3 py-2 text-xs text-[#fafafa] placeholder-[#a1a1aa] outline-none focus:border-[#d4d4d8] focus:ring-1 focus:ring-[#06b6d4]/40 transition-all"
            autocapitalize="none"
            autocorrect="off"
            spellcheck="false"
          />
          <button
            onclick={handleSaveCustomClientId}
            class="bg-[#fafafa] hover:bg-[#fafafa]/90 text-[#18181b] text-xs font-semibold px-3 py-2 rounded-lg transition-colors cursor-pointer border-0"
          >
            Save
          </button>
          {#if localStorage.getItem('gdrive_custom_client_id')}
            <button
              onclick={handleResetCustomClientId}
              class="bg-transparent border border-[#7f1d1d]/30 hover:border-[#ef4444] hover:bg-[#7f1d1d]/10 text-[#ef4444] text-xs px-3 py-2 rounded-lg transition-all cursor-pointer"
            >
              Reset
            </button>
          {/if}
        </div>
      </div>

      <!-- BACKUP & EXPORT CARD -->
      <div class="p-4 rounded-lg bg-[#18181b] border border-[#27272a] space-y-3">
        <div class="flex items-center gap-2">
          <Download class="h-4 w-4 text-[#fafafa]" />
          <h3 class="text-sm font-semibold text-[#fafafa]">Backup & Export Data</h3>
        </div>
        <span class="text-xs text-[#a1a1aa] leading-relaxed block">
          Download decrypted credentials in CSV or JSON formats.
          <p class="text-xs text-[#ef4444] leading-relaxed font-medium mt-1">WARNING: These will contain raw passwords. Handle with extreme caution.</p>
        </span>

        <div class="pt-2">
          {#if showExportConfirm}
            <div class="p-4 bg-[#09090b]/40 border border-[#27272a] rounded-lg space-y-3">
              <p class="text-xs text-[#fafafa]">
                Enter master password to authorize decrypted export ({exportType?.toUpperCase()}):
              </p>
              <div class="relative">
                <input
                  type={showExportPassword ? 'text' : 'password'}
                  bind:value={exportPassword}
                  placeholder="Master password"
                  class="w-full bg-[#09090b] border border-[#27272a] text-[#fafafa] placeholder-[#a1a1aa] text-xs px-3 py-2 pr-10 rounded-lg outline-none focus:border-[#d4d4d8] focus:ring-1 focus:ring-[#06b6d4]/40 transition-all"
                  onkeydown={(e) => e.key === 'Enter' && handleConfirmExport()}
                />
                <button
                  type="button"
                  class="absolute right-3 top-1/2 -translate-y-1/2 text-[#a1a1aa] hover:text-[#fafafa] transition-colors cursor-pointer no-scale border-0 bg-transparent"
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
                <p class="text-[11px] text-[#ef4444] flex items-center gap-1">
                  <span>⚠</span> {exportError}
                </p>
              {/if}
              <div class="flex gap-2 pt-1">
                <button
                  class="px-4 py-2 bg-[#fafafa] hover:bg-[#fafafa]/90 text-[#18181b] rounded-lg text-xs font-semibold transition-colors cursor-pointer border-0"
                  onclick={handleConfirmExport}
                >
                  Confirm Export
                </button>
                <button
                  class="px-4 py-2 bg-[#27272a] hover:bg-[#3f3f46] text-[#fafafa] rounded-lg text-xs font-semibold transition-colors border border-[#27272a] cursor-pointer"
                  onclick={() => { showExportConfirm = false; exportPassword = ''; exportError = ''; }}
                >
                  Cancel
                </button>
              </div>
            </div>
          {:else}
            <div class="flex flex-wrap gap-2">
              <button
                class="px-4 py-2 bg-[#fafafa] text-[#18181b] hover:bg-[#fafafa]/90 rounded-lg text-xs font-semibold flex items-center gap-2 transition-colors cursor-pointer border-0"
                onclick={() => requestExport('json')}
              >
                <Download class="h-3.5 w-3.5" /> Export JSON (All Data)
              </button>
              <button
                class="px-4 py-2 bg-[#27272a] hover:bg-[#3f3f46] border border-[#27272a] text-[#fafafa] rounded-lg text-xs font-semibold flex items-center gap-2 transition-colors cursor-pointer"
                onclick={() => requestExport('csv')}
              >
                <FileSpreadsheet class="h-3.5 w-3.5" /> Export Passwords (CSV)
              </button>
            </div>
          {/if}
        </div>
      </div>

      <!-- IMPORT DATA CARD -->
      <div class="p-4 rounded-lg bg-[#18181b] border border-[#27272a] space-y-3">
        <div class="flex items-center gap-2">
          <Upload class="h-4 w-4 text-[#fafafa]" />
          <h3 class="text-sm font-semibold text-[#fafafa]">Import Data</h3>
        </div>
        <p class="text-xs text-[#a1a1aa] leading-relaxed">
          Import credentials from an RFC 4180 CSV file. Columns must contain a header for <code class="text-[#06b6d4] bg-[#09090b] px-1.5 py-0.5 rounded font-mono text-[11px] border border-[#27272a]">name</code> or <code class="text-[#06b6d4] bg-[#09090b] px-1.5 py-0.5 rounded font-mono text-[11px] border border-[#27272a]">title</code>.
        </p>
        <div class="pt-2">
          <button
            onclick={handleNativeImportCsv}
            class="inline-flex px-4 py-2 bg-[#27272a] hover:bg-[#3f3f46] border border-[#27272a] text-[#fafafa] rounded-lg text-xs font-semibold items-center gap-2 transition-colors cursor-pointer"
          >
            <Upload class="h-3.5 w-3.5" /> Select & Import from CSV
          </button>
          {#if importSuccessMsg}
            <p class="text-[11px] text-emerald-400 mt-2 flex items-center gap-1">
              <span>✓</span> {importSuccessMsg}
            </p>
          {/if}
          {#if importErrorMsg}
            <p class="text-[11px] text-[#ef4444] mt-2 flex items-center gap-1">
              <span>⚠</span> {importErrorMsg}
            </p>
          {/if}
        </div>
      </div>

      <!-- SECURITY SETTINGS CARD -->
      <div class="p-4 rounded-lg bg-[#18181b] border border-[#27272a] space-y-3">
        <div class="flex items-center gap-2">
          <ShieldCheck class="h-4 w-4 text-[#fafafa]" />
          <h3 class="text-sm font-semibold text-[#fafafa]">Security Settings</h3>
        </div>
        <p class="text-xs text-[#a1a1aa] leading-relaxed">
          Configure device-level local re-authentication requirements before revealing or copying passwords.
        </p>
        <div class="pt-2 space-y-2">
          {#if vaultState.bioSupported}
            <div class="flex items-center justify-between p-3 bg-[#09090b]/40 border border-[#27272a] rounded-lg">
              <div class="space-y-0.5 text-left">
                <span class="text-xs font-semibold text-[#fafafa]">Biometric Re-authentication</span>
                <p class="text-[11px] text-[#a1a1aa] leading-relaxed max-w-[320px]">
                  Use {isMac ? 'Touch ID on this Mac' : 'Windows Hello'} to authorize copying or revealing passwords.
                </p>
              </div>
              <button
                onclick={handleToggleBiometrics}
                class="px-3 py-1 bg-[#27272a] border border-[#27272a] hover:bg-[#3f3f46] text-[#fafafa] text-xs font-semibold rounded-lg transition-colors cursor-pointer"
              >
                {vaultState.bioEnabled ? 'Disable' : 'Enable'}
              </button>
            </div>
          {:else}
            <div class="p-3 bg-[#09090b]/20 border border-[#27272a] rounded-lg">
              <p class="text-[11px] text-[#a1a1aa] leading-relaxed">
                Biometric authentication ({isMac ? 'Touch ID' : 'Windows Hello'}) is not supported or enrolled on this system. Master Password verification will be used as a fallback prompt.
              </p>
            </div>
          {/if}
          {#if bioError}
            <p class="text-[11px] text-[#ef4444] flex items-center gap-1">
              <span>⚠</span> {bioError}
            </p>
          {/if}
        </div>
      </div>

      <!-- RESET VAULT CARD -->
      <div class="p-4 rounded-lg bg-[#7f1d1d]/10 border border-[#7f1d1d]/20 space-y-3">
        <div class="flex items-center gap-2">
          <Trash2 class="h-4 w-4 text-[#ef4444]" />
          <h3 class="text-sm font-semibold text-[#ef4444]">Reset Vault Database</h3>
        </div>
        <p class="text-xs text-[#a1a1aa] leading-relaxed">
          Permanently delete the local encryption payload and keys. This action will completely erase all stored credentials on this machine. <span class="text-[#ef4444] font-medium">This cannot be undone.</span>
        </p>
        <div class="pt-2">
          {#if showSettingsResetConfirm}
            <div class="p-4 bg-[#7f1d1d]/20 border border-[#7f1d1d]/30 rounded-lg space-y-3">
              <p class="text-xs text-[#ef4444] font-medium">
                Warning: This wipes all local data. Type <strong class="text-white font-mono">RESET</strong> to confirm:
              </p>
              <input
                type="text"
                bind:value={resetConfirmText}
                placeholder="Type RESET"
                class="w-full bg-[#09090b] border border-[#7f1d1d]/40 text-[#ef4444] placeholder-[#7f1d1d]/50 text-xs px-3 py-2 rounded-lg outline-none focus:border-[#ef4444] focus:ring-1 focus:ring-[#ef4444]/30 transition-all font-mono"
              />
              {#if resetError}
                <p class="text-[11px] text-[#ef4444] flex items-center gap-1">
                  <span>⚠</span> {resetError}
                </p>
              {/if}
              <div class="flex gap-2 pt-1">
                <button
                  class="px-4 py-2 bg-[#7f1d1d] hover:bg-[#991b1b] text-[#fafafa] rounded-lg text-xs font-semibold cursor-pointer transition-colors border-0"
                  onclick={handleSettingsResetVault}
                >
                  Confirm Complete Reset
                </button>
                <button
                  class="px-4 py-2 bg-[#27272a] hover:bg-[#3f3f46] text-[#fafafa] rounded-lg text-xs font-semibold border border-[#27272a] cursor-pointer transition-colors"
                  onclick={() => { showSettingsResetConfirm = false; resetConfirmText = ''; resetError = ''; }}
                >
                  Cancel
                </button>
              </div>
            </div>
          {:else}
            <button
              class="px-4 py-2 bg-[#7f1d1d]/20 hover:bg-[#7f1d1d]/30 border border-[#7f1d1d]/30 text-[#ef4444] rounded-lg text-xs font-semibold flex items-center gap-2 transition-colors cursor-pointer"
              onclick={() => showSettingsResetConfirm = true}
            >
              <Trash2 class="h-3.5 w-3.5" /> Reset Local Vault
            </button>
          {/if}
        </div>
      </div>
    </div>
  </div>
</div>
