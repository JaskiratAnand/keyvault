<script lang="ts">
import {
  CircleHelp,
  Fingerprint,
  Globe,
  KeyRound,
  ShieldCheck,
} from 'lucide-svelte';
</script>

<div class="grow overflow-y-auto p-8 space-y-6">
  <div class="max-w-xl mx-auto space-y-6 animate-fade-in">
    <div>
      <h2 class="text-lg font-bold text-white mb-1 flex items-center gap-2">
        <CircleHelp class="h-5 w-5 text-[#06b6d4]" />
        Help & Guide
      </h2>
      <p class="text-xs text-zinc-400 font-sans">Learn how to get the most out of your zero-knowledge password manager.</p>
    </div>

    <div class="space-y-4">
      <!-- SECTION 1: AUTOFILL -->
      <div class="p-4 rounded-lg bg-[#18181b] border border-[#27272a] space-y-2">
        <div class="flex items-center gap-2 text-left">
          <div class="p-1.5 bg-[#06b6d4]/10 rounded-md text-[#06b6d4]">
            <Fingerprint class="h-4 w-4" />
          </div>
          <h3 class="text-sm text-white font-semibold">1. Extension Autofill</h3>
        </div>
        <p class="text-xs text-zinc-400 leading-relaxed font-sans text-left">
          KeyVault injects an <strong>Inline Autofill Overlay</strong> directly into detected input fields on login screens.
          Click the <strong>Autofill Badge</strong> (the key icon inside the text input) to open the <strong>Autofill Dropdown Panel</strong> and instantly select your account credentials.
        </p>
      </div>

      <!-- SECTION 2: SHORTCUTS -->
      <div class="p-4 rounded-lg bg-[#18181b] border border-[#27272a] space-y-2">
        <div class="flex items-center gap-2 text-left">
          <div class="p-1.5 bg-yellow-500/10 rounded-md text-yellow-400">
            <KeyRound class="h-4 w-4" />
          </div>
          <h3 class="text-sm text-white font-semibold">2. Keyboard Shortcuts</h3>
        </div>
        <p class="text-xs text-zinc-400 leading-relaxed font-sans text-left">
          Navigate quickly using your keyboard:
        </p>
        <div class="grid grid-cols-2 gap-2 text-[10px] text-zinc-300 font-mono pt-1 text-left">
          <div class="flex justify-between p-1.5 bg-zinc-900/40 rounded border border-zinc-800">
            <span>Open Popup:</span>
            <span class="text-[#06b6d4]">Cmd+Shift+K</span>
          </div>
          <div class="flex justify-between p-1.5 bg-zinc-900/40 rounded border border-zinc-800">
            <span>Search Vault:</span>
            <span class="text-[#06b6d4]">/</span>
          </div>
          <div class="flex justify-between p-1.5 bg-zinc-900/40 rounded border border-zinc-800">
            <span>Focus Next:</span>
            <span class="text-[#06b6d4]">Tab</span>
          </div>
          <div class="flex justify-between p-1.5 bg-zinc-900/40 rounded border border-zinc-800">
            <span>Lock Vault:</span>
            <span class="text-[#06b6d4]">Esc</span>
          </div>
        </div>
      </div>

      <!-- SECTION 3: SYNC -->
      <div class="p-4 rounded-lg bg-[#18181b] border border-[#27272a] space-y-2">
        <div class="flex items-center gap-2 text-left">
          <div class="p-1.5 bg-green-500/10 rounded-md text-green-400">
            <Globe class="h-4 w-4" />
          </div>
          <h3 class="text-sm text-white font-semibold">3. Cloud Synchronisation</h3>
        </div>
        <p class="text-xs text-zinc-400 leading-relaxed text-left font-sans">
          Sync keeps your database updated across all devices. We use client-side zero-knowledge encryption where credentials are encrypted locally with your derived Vault Key before uploading to Google Drive's secure <code>appDataFolder</code>.
        </p>
        <p class="text-[10px] text-zinc-500 leading-normal text-left">
          * A silent sync merges local/remote databases every time you add, update, or unlock credentials.
        </p>
      </div>

      <!-- SECTION 4: BEST PRACTICES -->
      <div class="p-4 rounded-lg bg-[#18181b] border border-[#27272a] space-y-2">
        <div class="flex items-center gap-2 text-left">
          <div class="p-1.5 bg-[#06b6d4]/10 rounded-md text-[#06b6d4]">
            <ShieldCheck class="h-4 w-4" />
          </div>
          <h3 class="text-sm text-white font-semibold">4. Security Best Practices</h3>
        </div>
        <ul class="text-xs text-zinc-400 leading-relaxed font-sans text-left list-disc pl-4 space-y-1">
          <li><strong>Generator</strong>: Always use the built-in Password Generator to create high-entropy, unique passwords.</li>
          <li><strong>Recovery Key</strong>: Keep your emergency recovery key written down offline in a physically secure location.</li>
          <li>
            <strong>Custom Client ID</strong>: For advanced control, you can configure your own OAuth Client ID under settings to sync directly to your personal API developer console.
            <details class="mt-2 border border-zinc-800 bg-zinc-900/40 rounded-md overflow-hidden">
              <summary class="flex justify-between items-center p-2 text-[10px] font-medium text-zinc-300 cursor-pointer hover:bg-zinc-800/40 select-none">
                <span>Guide: How to create a Custom Client ID</span>
              </summary>
              <div class="p-3 border-t border-zinc-800 text-[10px] text-zinc-400 font-sans space-y-2 text-left leading-relaxed">
                <p>To use your own Google Cloud configuration for desktop sync:</p>
                <ol class="list-decimal pl-4 space-y-1">
                  <li>Go to the <a href="https://console.cloud.google.com/" target="_blank" rel="noopener noreferrer" class="text-[#06b6d4] hover:underline">Google Cloud Console</a>.</li>
                  <li>Create a new project (e.g. <code>KeyVault-Desktop</code>).</li>
                  <li>Go to <strong>APIs & Services &gt; Library</strong>, search for <strong>Google Drive API</strong>, and click <strong>Enable</strong>.</li>
                  <li>Go to <strong>OAuth consent screen</strong>, select <strong>External</strong>, and click <strong>Create</strong>. Fill in app details, add scope <code>https://www.googleapis.com/auth/drive.appdata</code>, and add your email as a <strong>Test User</strong>.</li>
                  <li>Go to <strong>Credentials</strong>, click <strong>Create Credentials &gt; OAuth Client ID</strong>.</li>
                  <li>Choose an application type based on your needs:
                    <ul class="list-disc pl-4 mt-0.5 space-y-1">
                      <li><strong>Option A (Desktop Application - Recommended for Settings UI)</strong>: Select <strong>Desktop application</strong>. Google does not issue or require a Client Secret for desktop clients. Copy the generated <strong>Client ID</strong> and paste it directly into KeyVault's <strong>Settings &gt; Custom Google Client ID</strong>. <em>This is the only type compatible with the settings page input, as the UI does not collect a client secret.</em></li>
                      <li><strong>Option B (Web Application - Requires Rebuild)</strong>: Select <strong>Web application</strong>. Google strictly requires a Client Secret for web clients. Under <strong>Authorized redirect URIs</strong>, add <code>http://127.0.0.1:40305</code>. Copy the generated <strong>Client ID</strong> and <strong>Client Secret</strong>, save them in your local <code>.env</code> file (as <code>VITE_GDRIVE_DESKTOP_CLIENT_ID</code> and <code>VITE_GDRIVE_DESKTOP_CLIENT_SECRET</code>), and rebuild the application. <em>Note: Using a Web client ID in the settings UI will fail.</em></li>
                    </ul>
                  </li>
                </ol>
              </div>
            </details>
          </li>
        </ul>
      </div>
    </div>
  </div>
</div>
