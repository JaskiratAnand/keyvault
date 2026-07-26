<script lang="ts">
import {
  AlertTriangle,
  ArrowLeft,
  ArrowRight,
  Check,
  CheckCircle2,
  Copy,
  Eye,
  EyeOff,
  Fingerprint,
  Info,
  KeyRound,
  Lock,
  RefreshCw,
  ShieldCheck,
} from 'lucide-svelte';
import { onMount } from 'svelte';
import { vaultState } from '../lib/vault-state.svelte.js';

let { onComplete } = $props<{ onComplete?: () => void }>();

const { invoke } = (window as any).__TAURI__?.core || {};

// Wizard Steps:
// 1: Choose Path (Create New vs Google Drive Restore)
// 2: Security Config (Set Master Password or Google Drive SignIn & Decrypt)
// 3: Hardening (Emergency Recovery Key, PIN, and Biometrics)
// 4: Setup Complete
let step = $state(1);
let path = $state<'new' | 'restore' | null>(null);

// Form Inputs
let masterPassword = $state('');
let confirmPassword = $state('');
let showPassword = $state(false);

let recoveryPassword = $state('');
let showRecoveryPassword = $state(false);

// Strength Validation rules
let strengthScore = $derived.by(() => {
  let score = 0;
  if (masterPassword.length >= 8) score++;
  if (/[A-Z]/.test(masterPassword)) score++;
  if (/[a-z]/.test(masterPassword)) score++;
  if (/[0-9]/.test(masterPassword)) score++;
  if (/[^A-Za-z0-9]/.test(masterPassword)) score++;
  return score;
});

let passwordValidations = $derived({
  length: masterPassword.length >= 8,
  uppercase: /[A-Z]/.test(masterPassword),
  lowercase: /[a-z]/.test(masterPassword),
  number: /[0-9]/.test(masterPassword),
  special: /[^A-Za-z0-9]/.test(masterPassword),
});

let isPasswordValid = $derived(
  passwordValidations.length &&
    passwordValidations.uppercase &&
    passwordValidations.lowercase &&
    passwordValidations.number &&
    passwordValidations.special &&
    masterPassword === confirmPassword,
);

// Cloud Search/Restore state
let searchStatus = $state<
  'idle' | 'searching' | 'found' | 'not_found' | 'error'
>('idle');
let searchError = $state('');
let decryptError = $state('');
let isRestoring = $state(false);

// Hardening States
let recoveryKey = $state('');
let verifiedRecoveryKey = $state('');
let showRecoveryCopied = $state(false);
let isRecoveryVerified = $derived(
  recoveryKey.replace(/[- ]/g, '').toLowerCase() ===
    verifiedRecoveryKey.replace(/[- ]/g, '').toLowerCase() &&
    recoveryKey.length > 0,
);

let bioSupported = $state(false);
let bioEnabled = $state(false);
let pinEnabled = $state(false);
let pin = $state('');
let confirmPin = $state('');
let pinError = $state('');

onMount(async () => {
  if (invoke) {
    try {
      bioSupported = await invoke('is_biometrics_supported');
    } catch {
      bioSupported = false;
    }
  }
});

// Logic Functions
const generateRecoveryKey = () => {
  const arr = new Uint8Array(16);
  crypto.getRandomValues(arr);
  const hex = Array.from(arr)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
  const matches = hex.match(/.{1,4}/g);
  recoveryKey = matches ? matches.join('-').toUpperCase() : hex.toUpperCase();
};

const handleCopyRecoveryKey = async () => {
  if (!recoveryKey) return;
  await navigator.clipboard.writeText(recoveryKey);
  showRecoveryCopied = true;
  setTimeout(() => {
    showRecoveryCopied = false;
  }, 2000);
};

const handleChoosePath = (selectedPath: 'new' | 'restore') => {
  path = selectedPath;
  if (path === 'new') {
    generateRecoveryKey();
    step = 2;
  } else {
    step = 2;
    handleGoogleSignIn();
  }
};

const handleGoogleSignIn = async () => {
  searchStatus = 'searching';
  searchError = '';
  try {
    const result = await vaultState.signInAndFetchRemoteVault();
    if (result.exists) {
      searchStatus = 'found';
    } else if (result.error) {
      searchStatus = 'error';
      searchError = result.error;
    } else {
      searchStatus = 'not_found';
    }
  } catch (e) {
    searchStatus = 'error';
    searchError = e instanceof Error ? e.message : String(e);
  }
};

const handleDecryptAndRestore = async () => {
  decryptError = '';
  isRestoring = true;
  try {
    const success = await vaultState.restoreRemoteFirstTime(recoveryPassword);
    if (success) {
      generateRecoveryKey(); // Prepare recovery key for hardening page
      step = 3;
    } else {
      decryptError = vaultState.syncError || 'Incorrect password.';
    }
  } catch (e) {
    decryptError = e instanceof Error ? e.message : String(e);
  } finally {
    isRestoring = false;
  }
};

const handleInitializeNew = async () => {
  if (!isPasswordValid) return;
  vaultState.error = '';
  const success = await vaultState.unlock(masterPassword);
  if (success) {
    step = 3;
  }
};

const handleEnableBiometrics = async () => {
  if (!invoke) return;
  try {
    const success = await invoke('authenticate_biometrics');
    if (success) {
      bioEnabled = true;
      localStorage.setItem('biometrics_enabled', 'true');
      vaultState.bioEnabled = true;
    }
  } catch (e) {
    console.error('Biometrics failed:', e);
  }
};

const handleSaveHardening = async () => {
  pinError = '';

  // 1. Process PIN if filled
  if (pinEnabled) {
    if (!pin || pin.length < 4 || pin.length > 6 || !/^\d+$/.test(pin)) {
      pinError = 'PIN must be 4 to 6 digits.';
      return;
    }
    if (pin !== confirmPin) {
      pinError = 'PINs do not match.';
      return;
    }

    try {
      const encoder = new TextEncoder();
      const cleanPin = pin;
      const data = encoder.encode(cleanPin);
      const hashBuffer = await crypto.subtle.digest('SHA-256', data);
      const hashArray = Array.from(new Uint8Array(hashBuffer));
      const pinHash = hashArray
        .map((b) => b.toString(16).padStart(2, '0'))
        .join('');

      if (invoke) {
        const payload = await invoke('generate_recovery_key_payload', {
          recoveryKey: cleanPin,
        });
        localStorage.setItem('vault_pin_hash', pinHash);
        localStorage.setItem(
          'vault_pin_salt',
          JSON.stringify(Array.from(payload.recovery_salt as number[])),
        );
        localStorage.setItem(
          'vault_pin_payload',
          JSON.stringify(Array.from(payload.encrypted_key as number[])),
        );
        localStorage.setItem('pin_enabled', 'true');
      }
    } catch (e) {
      pinError = `PIN setup failed: ${e instanceof Error ? e.message : String(e)}`;
      return;
    }
  }

  // 2. Process Recovery Key
  try {
    const cleanRecovery = recoveryKey.replace(/[- ]/g, '').toLowerCase();
    const encoder = new TextEncoder();
    const data = encoder.encode(cleanRecovery);
    const hashBuffer = await crypto.subtle.digest('SHA-256', data);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    const recoveryHash = hashArray
      .map((b) => b.toString(16).padStart(2, '0'))
      .join('');

    if (invoke) {
      const payload = await invoke('generate_recovery_key_payload', {
        recoveryKey: cleanRecovery,
      });
      localStorage.setItem('vault_recovery_hash', recoveryHash);
      localStorage.setItem(
        'vault_recovery_salt',
        JSON.stringify(Array.from(payload.recovery_salt as number[])),
      );
      localStorage.setItem(
        'vault_recovery_payload',
        JSON.stringify(Array.from(payload.encrypted_key as number[])),
      );
    }
  } catch (e) {
    console.error('Failed to save recovery payload:', e);
  }

  step = 4;
};

const handleFinish = () => {
  if (onComplete) {
    onComplete();
  }
};
</script>

<div class="flex flex-col justify-center w-full py-2 space-y-6 text-zinc-100 max-w-xl mx-auto">
  <!-- Step indicator -->
  <div class="flex items-center justify-between border-b border-zinc-800 pb-3">
    <div class="flex items-center gap-2">
      <Lock class="h-5 w-5 text-[#06b6d4]" />
      <span class="text-sm font-bold text-white tracking-tight font-semibold">KeyVault Wizard</span>
    </div>
    <div class="flex items-center gap-1.5 font-sans">
      {#each [1, 2, 3, 4] as num}
        <div class="flex items-center gap-1.5">
          <div 
            class="w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-bold border transition-all duration-300"
            class:bg-[#06b6d4]={step >= num}
            class:border-[#06b6d4]={step >= num}
            class:text-[#18181b]={step >= num}
            class:border-zinc-700={step < num}
            class:text-zinc-500={step < num}
          >
            {num}
          </div>
          {#if num < 4}
            <div class="w-4 h-[1px]" class:bg-[#06b6d4]={step > num} class:bg-zinc-800={step <= num}></div>
          {/if}
        </div>
      {/each}
    </div>
  </div>

  {#if step === 1}
    <!-- STEP 1: CHOOSE PATH -->
    <div class="space-y-4">
      <div class="space-y-1.5">
        <h2 class="text-base font-bold text-white tracking-tight font-semibold">Get Started with KeyVault</h2>
        <p class="text-xs text-zinc-400 leading-relaxed">
          Select how you want to configure your security credentials. Your data is always encrypted locally on this device.
        </p>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 pt-2">
        <button
          onclick={() => handleChoosePath('new')}
          class="flex flex-col items-start text-left p-4 bg-zinc-900/40 border border-zinc-800 rounded-lg hover:border-[#06b6d4]/50 hover:bg-zinc-900 transition-all duration-200 cursor-pointer group"
        >
          <div class="p-2 bg-[#06b6d4]/10 rounded-md text-[#06b6d4] group-hover:scale-110 transition-transform mb-3">
            <ShieldCheck class="h-5 w-5" />
          </div>
          <span class="text-xs font-semibold text-white mb-1">Create a New Local Vault</span>
          <span class="text-[11px] text-zinc-400 leading-normal">
            For fresh installations. Initializes a zero-knowledge local password storage database.
          </span>
        </button>

        <button
          onclick={() => handleChoosePath('restore')}
          class="flex flex-col items-start text-left p-4 bg-zinc-900/40 border border-zinc-800 rounded-lg hover:border-[#06b6d4]/50 hover:bg-zinc-900 transition-all duration-200 cursor-pointer group"
        >
          <div class="p-2 bg-green-500/10 rounded-md text-green-400 group-hover:scale-110 transition-transform mb-3">
            <RefreshCw class="h-5 w-5" />
          </div>
          <span class="text-xs font-semibold text-white mb-1">Restore from Google Drive</span>
          <span class="text-[11px] text-zinc-400 leading-normal">
            Pull and restore your existing encrypted credentials database from cloud sync storage.
          </span>
        </button>
      </div>
    </div>

  {:else if step === 2}
    <!-- STEP 2: SECURITY CONFIGURATION -->
    {#if path === 'new'}
      <div class="space-y-4 font-sans">
        <div class="space-y-1.5 flex justify-between items-start">
          <div>
            <h2 class="text-base font-bold text-white tracking-tight font-semibold">Create Master Password</h2>
            <p class="text-xs text-zinc-400 leading-relaxed">
              Set a master password. It derives the AES-256 key to secure your credentials locally.
            </p>
          </div>
          <button onclick={() => step = 1} class="text-xs text-zinc-400 hover:text-white bg-transparent border-0 flex items-center gap-1 cursor-pointer">
            <ArrowLeft class="h-3 w-3" /> Back
          </button>
        </div>

        <div class="space-y-4">
          <div class="space-y-1.5">
            <label for="master-password" class="text-xs font-semibold text-zinc-400 uppercase tracking-wider block">Master Password</label>
            <div class="relative">
              <input
                id="master-password"
                type={showPassword ? "text" : "password"}
                bind:value={masterPassword}
                placeholder="Secure Master Password"
                class="w-full bg-[#09090b] border border-[#27272a] rounded-lg px-3 py-2 text-sm text-[#fafafa] placeholder-[#52525b] outline-none transition-colors focus:border-[#d4d4d8] disabled:opacity-50 pr-10"
              />
              <button
                class="h-7 w-7 absolute right-1.5 top-1/2 -translate-y-1/2 text-zinc-400 hover:text-white bg-transparent border-0 cursor-pointer flex items-center justify-center"
                onclick={() => (showPassword = !showPassword)}
              >
                {#if showPassword}
                  <EyeOff class="h-3.5 w-3.5" />
                {:else}
                  <Eye class="h-3.5 w-3.5" />
                {/if}
              </button>
            </div>
          </div>

          <!-- Password Strength Meter -->
          <div class="space-y-2 p-3 bg-zinc-950 border border-zinc-900 rounded-lg">
            <div class="flex items-center justify-between text-[10px]">
              <span class="text-zinc-400 font-medium">Password Strength</span>
              <span class="font-bold uppercase" class:text-red-400={strengthScore <= 2} class:text-yellow-400={strengthScore === 3 || strengthScore === 4} class:text-green-400={strengthScore === 5}>
                {#if strengthScore === 0}Empty{:else if strengthScore <= 2}Weak{:else if strengthScore <= 4}Medium{:else}Strong{/if}
              </span>
            </div>
            <div class="grid grid-cols-5 gap-1">
              {#each Array(5) as _, i}
                <div 
                  class="h-1 rounded-full transition-all duration-300"
                  class:bg-red-500={strengthScore > i && strengthScore <= 2}
                  class:bg-yellow-500={strengthScore > i && (strengthScore === 3 || strengthScore === 4)}
                  class:bg-green-500={strengthScore > i && strengthScore === 5}
                  class:bg-zinc-800={strengthScore <= i}
                ></div>
              {/each}
            </div>
            
            <div class="grid grid-cols-2 gap-x-2 gap-y-1 text-[9px] pt-1">
              <span class="flex items-center gap-1.5" class:text-green-400={passwordValidations.length} class:text-zinc-500={!passwordValidations.length}>
                <CheckCircle2 class="h-2.5 w-2.5 text-green-400" /> 8+ Characters
              </span>
              <span class="flex items-center gap-1.5" class:text-green-400={passwordValidations.uppercase} class:text-zinc-500={!passwordValidations.uppercase}>
                <CheckCircle2 class="h-2.5 w-2.5 text-green-400" /> Uppercase (A-Z)
              </span>
              <span class="flex items-center gap-1.5" class:text-green-400={passwordValidations.lowercase} class:text-zinc-500={!passwordValidations.lowercase}>
                <CheckCircle2 class="h-2.5 w-2.5 text-green-400" /> Lowercase (a-z)
              </span>
              <span class="flex items-center gap-1.5" class:text-green-400={passwordValidations.number} class:text-zinc-500={!passwordValidations.number}>
                <CheckCircle2 class="h-2.5 w-2.5 text-green-400" /> Number (0-9)
              </span>
              <span class="flex items-center gap-1.5 col-span-2" class:text-green-400={passwordValidations.special} class:text-zinc-500={!passwordValidations.special}>
                <CheckCircle2 class="h-2.5 w-2.5 text-green-400" /> Special character (!@#$%^&*)
              </span>
            </div>
          </div>

          <div class="space-y-1.5">
            <label for="confirm-password" class="text-xs font-semibold text-zinc-400 uppercase tracking-wider block">Confirm Master Password</label>
            <input
              id="confirm-password"
              type="password"
              bind:value={confirmPassword}
              placeholder="Retype password"
              class="w-full bg-[#09090b] border border-[#27272a] rounded-lg px-3 py-2 text-sm text-[#fafafa] placeholder-[#52525b] outline-none transition-colors focus:border-[#d4d4d8]"
            />
          </div>

          {#if vaultState.error}
            <div class="p-3 bg-red-950/20 border border-red-800/40 text-red-400 rounded-md text-[11px] flex gap-2 items-start">
              <AlertTriangle class="h-4 w-4 shrink-0" />
              <span>{vaultState.error}</span>
            </div>
          {/if}

          <button 
            disabled={!isPasswordValid}
            class="w-full bg-[#fafafa] text-[#18181b] hover:bg-zinc-200 text-xs font-semibold py-2 rounded-lg flex items-center justify-center gap-1.5 transition-all cursor-pointer disabled:opacity-50 border-0"
            onclick={handleInitializeNew}
          >
            Create Secure Vault <ArrowRight class="h-3.5 w-3.5" />
          </button>
        </div>
      </div>
    {:else}
      <!-- GOOGLE DRIVE RESTORE -->
      <div class="space-y-4 font-sans">
        <div class="space-y-1.5 flex justify-between items-start">
          <div>
            <h2 class="text-base font-bold text-white tracking-tight font-semibold">Restore Cloud Backup</h2>
            <p class="text-xs text-zinc-400 leading-relaxed">
              Authenticate and download your encrypted database configuration from Google Drive.
            </p>
          </div>
          <button onclick={() => step = 1} class="text-xs text-zinc-400 hover:text-white bg-transparent border-0 flex items-center gap-1 cursor-pointer">
            <ArrowLeft class="h-3 w-3" /> Back
          </button>
        </div>

        {#if searchStatus === 'searching'}
          <div class="flex flex-col items-center justify-center p-8 gap-4 bg-zinc-950/50 border border-zinc-900 rounded-lg">
            <RefreshCw class="h-6 w-6 animate-spin text-[#06b6d4]" />
            <div class="text-center space-y-1">
              <p class="text-xs text-zinc-200 font-semibold">Connecting to Google Drive...</p>
              <p class="text-[10px] text-zinc-400">Searching your appDataFolder for vault.db</p>
            </div>
          </div>
        {:else if searchStatus === 'not_found'}
          <div class="p-4 bg-yellow-950/20 border border-yellow-800/30 text-yellow-400 rounded-lg text-xs space-y-3">
            <div class="flex gap-2 items-start">
              <Info class="h-4 w-4 shrink-0 mt-0.5 text-yellow-400" />
              <div>
                <p class="font-bold font-semibold">No Backup Found</p>
                <p class="text-zinc-300 mt-1 leading-normal">
                  We scanned your Google Drive secure storage and could not find an existing `vault.db` file. You can start fresh by creating a new vault.
                </p>
              </div>
            </div>
            <div class="flex gap-2 pt-1">
              <button onclick={() => handleChoosePath('new')} class="bg-[#fafafa] text-[#18181b] hover:bg-zinc-200 text-[10px] font-semibold py-1.5 px-3 rounded cursor-pointer border-0">
                Create New Vault Instead
              </button>
              <button onclick={handleGoogleSignIn} class="text-[10px] text-zinc-300 hover:text-white bg-transparent hover:bg-zinc-900 border border-zinc-800 py-1.5 px-3 rounded cursor-pointer">
                Retry Scan
              </button>
            </div>
          </div>
        {:else if searchStatus === 'error'}
          <div class="p-4 bg-red-950/20 border border-red-800/40 text-red-400 rounded-lg text-xs space-y-3">
            <div class="flex gap-2 items-start">
              <AlertTriangle class="h-4 w-4 shrink-0 mt-0.5" />
              <div>
                <p class="font-bold font-semibold">Cloud Connection Failed</p>
                <p class="text-zinc-300 mt-1 leading-normal">{searchError}</p>
              </div>
            </div>
            <button onclick={handleGoogleSignIn} class="bg-[#fafafa] text-[#18181b] hover:bg-zinc-200 text-[10px] font-semibold py-1.5 px-3 rounded cursor-pointer border-0">
              Retry Connection
            </button>
          </div>
        {:else if searchStatus === 'found'}
          <!-- Vault Found! Decrypt configuration -->
          <div class="space-y-4">
            <div class="p-3 bg-zinc-950 border border-zinc-900 rounded-lg flex items-center justify-between">
              <div class="flex items-center gap-2">
                <div class="w-2 h-2 rounded-full bg-green-500"></div>
                <span class="text-xs font-semibold text-zinc-200">Vault found on Google Drive</span>
              </div>
              <span class="text-[10px] text-zinc-500 font-mono">Size: ~ {vaultState.pendingRemotePayload?.length || 0} bytes</span>
            </div>

            <div class="space-y-1.5">
              <label for="recovery-password" class="text-xs font-semibold text-zinc-400 uppercase tracking-wider block">Vault Password</label>
              <div class="relative">
                <input
                  id="recovery-password"
                  type={showRecoveryPassword ? "text" : "password"}
                  bind:value={recoveryPassword}
                  placeholder="Enter vault password to unlock"
                  class="w-full bg-[#09090b] border border-[#27272a] rounded-lg px-3 py-2 text-sm text-[#fafafa] placeholder-[#52525b] outline-none pr-10"
                />
                <button
                  class="h-7 w-7 absolute right-1.5 top-1/2 -translate-y-1/2 text-zinc-400 hover:text-white bg-transparent border-0 cursor-pointer flex items-center justify-center"
                  onclick={() => (showRecoveryPassword = !showRecoveryPassword)}
                >
                  {#if showRecoveryPassword}
                    <EyeOff class="h-3.5 w-3.5" />
                  {:else}
                    <Eye class="h-3.5 w-3.5" />
                  {/if}
                </button>
              </div>
            </div>

            {#if decryptError}
              <div class="p-3 bg-red-950/20 border border-red-800/40 text-red-400 rounded-md text-[11px] flex gap-2 items-start">
                <AlertTriangle class="h-4 w-4 shrink-0" />
                <span>{decryptError}</span>
              </div>
            {/if}

            <button 
              disabled={isRestoring || !recoveryPassword}
              class="w-full bg-[#fafafa] text-[#18181b] hover:bg-zinc-200 text-xs font-semibold py-2 rounded-lg flex items-center justify-center gap-1.5 transition-all cursor-pointer disabled:opacity-50 border-0"
              onclick={handleDecryptAndRestore}
            >
              {#if isRestoring}
                <RefreshCw class="h-4.5 w-4.5 animate-spin" /> Restoring & Decrypting...
              {:else}
                Decrypt & Restore Vault <ArrowRight class="h-3.5 w-3.5" />
              {/if}
            </button>
          </div>
        {:else}
          <!-- Idle / Initial Google Sign in button -->
          <div class="flex flex-col items-center justify-center p-8 gap-4 bg-zinc-950 border border-zinc-900 rounded-lg">
            <RefreshCw class="h-8 w-8 text-zinc-600 animate-pulse" />
            <button onclick={handleGoogleSignIn} class="bg-[#fafafa] text-[#18181b] hover:bg-zinc-200 text-xs font-semibold py-2 px-6 rounded-lg cursor-pointer border-0">
              Sign In with Google
            </button>
          </div>
        {/if}
      </div>
    {/if}

  {:else if step === 3}
    <!-- STEP 3: SECURITY HARDENING -->
    <div class="space-y-4 font-sans">
      <div class="space-y-1.5">
        <h2 class="text-base font-bold text-white tracking-tight font-semibold">Security Hardening</h2>
        <p class="text-xs text-zinc-400 leading-relaxed">
          Record your emergency recovery key and set up device-level local convenience unlock options.
        </p>
      </div>

      <!-- Recovery Key card -->
      <div class="space-y-3 p-4 bg-zinc-950 border border-zinc-900 rounded-lg">
        <div class="flex justify-between items-center">
          <label class="text-xs font-bold text-[#06b6d4] uppercase tracking-wider flex items-center gap-1">
            <Lock class="h-3 w-3" /> Emergency Recovery Key
          </label>
          <button 
            onclick={handleCopyRecoveryKey} 
            class="h-7 text-[10px] text-zinc-300 hover:text-white bg-transparent hover:bg-zinc-900 border border-zinc-800 rounded py-1 px-3 flex items-center gap-1 cursor-pointer"
          >
            {#if showRecoveryCopied}
              <Check class="h-3.5 w-3.5 text-green-400" /> Copied
            {:else}
              <Copy class="h-3.5 w-3.5" /> Copy Key
            {/if}
          </button>
        </div>
        
        <div class="bg-[#09090b] border border-zinc-900 p-3 rounded-md text-center">
          <span class="text-xs font-mono font-bold tracking-widest text-zinc-200 select-all">{recoveryKey}</span>
        </div>

        <p class="text-[10px] text-zinc-400 leading-normal flex items-start gap-1.5">
          <Info class="h-3.5 w-3.5 text-[#06b6d4] shrink-0 mt-0.5" />
          Write this code down. If you forget your Master Password, this is the ONLY way to decrypt and recover your database.
        </p>

        <!-- Verification check -->
        <div class="space-y-1.5 pt-1 border-t border-zinc-900 mt-2">
          <label for="verify-key" class="text-[10px] font-semibold text-zinc-500 uppercase tracking-wider block">Verify Recovery Key (paste or type to verify)</label>
          <input 
            id="verify-key" 
            type="text" 
            bind:value={verifiedRecoveryKey}
            placeholder="XXXX-XXXX-XXXX..."
            class="w-full bg-[#09090b] border border-[#27272a] rounded-lg px-3 py-1.5 text-xs text-[#fafafa] placeholder-[#52525b] outline-none font-mono uppercase"
          />
        </div>
      </div>

      <!-- Biometrics & PIN setup -->
      <div class="space-y-3 p-4 bg-zinc-950 border border-zinc-900 rounded-lg">
        <span class="text-xs font-semibold text-zinc-400 uppercase tracking-wider block">Convenience Local Unlock</span>
        
        {#if bioSupported}
          <div class="flex items-center justify-between py-2 border-b border-zinc-900/60">
            <div>
              <p class="text-xs font-semibold text-white flex items-center gap-1">
                <Fingerprint class="h-3.5 w-3.5 text-[#06b6d4]" /> Platform Biometrics
              </p>
              <p class="text-[10px] text-zinc-400 leading-normal">Unlock secure items instantly via Touch ID or Windows Hello.</p>
            </div>
            <button 
              onclick={handleEnableBiometrics}
              class="h-7 text-[10px] font-semibold cursor-pointer border-0 rounded px-3 py-1 {bioEnabled ? 'bg-green-500 text-white hover:bg-green-600' : 'bg-zinc-800 text-zinc-300 hover:bg-zinc-700'}"
            >
              {bioEnabled ? 'Enabled' : 'Enable'}
            </button>
          </div>
        {/if}

        <div class="space-y-3 pt-2">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-xs font-semibold text-white flex items-center gap-1">
                <KeyRound class="h-3.5 w-3.5 text-[#06b6d4]" /> Setup Quick PIN
              </p>
              <p class="text-[10px] text-zinc-400 leading-normal">Set a 4 to 6 digit PIN to unlock your vault quickly on this machine.</p>
            </div>
            <input 
              type="checkbox" 
              bind:checked={pinEnabled} 
              class="rounded border-zinc-800 bg-zinc-950 text-[#06b6d4] focus:ring-[#06b6d4] cursor-pointer" 
            />
          </div>

          {#if pinEnabled}
            <div class="grid grid-cols-2 gap-3 pt-1">
              <div class="space-y-1">
                <label for="quick-pin" class="text-[9px] font-semibold text-zinc-500 uppercase tracking-wider block">Enter PIN</label>
                <input 
                  id="quick-pin" 
                  type="password" 
                  maxlength={6}
                  bind:value={pin}
                  placeholder="4-6 digits"
                  class="w-full bg-[#09090b] border border-[#27272a] rounded-lg px-2 py-1.5 text-xs text-[#fafafa] placeholder-[#52525b] outline-none"
                />
              </div>
              <div class="space-y-1">
                <label for="confirm-pin" class="text-[9px] font-semibold text-zinc-500 uppercase tracking-wider block">Confirm PIN</label>
                <input 
                  id="confirm-pin" 
                  type="password" 
                  maxlength={6}
                  bind:value={confirmPin}
                  placeholder="Confirm PIN"
                  class="w-full bg-[#09090b] border border-[#27272a] rounded-lg px-2 py-1.5 text-xs text-[#fafafa] placeholder-[#52525b] outline-none"
                />
              </div>
            </div>
          {/if}
        </div>

        {#if pinError}
          <div class="p-2 bg-red-950/20 border border-red-800/40 text-red-400 rounded-md text-[10px] flex gap-1.5 items-start mt-2">
            <AlertTriangle class="h-3.5 w-3.5 shrink-0" />
            <span>{pinError}</span>
          </div>
        {/if}
      </div>

      <button 
        disabled={!isRecoveryVerified}
        class="w-full bg-[#fafafa] text-[#18181b] hover:bg-zinc-200 text-xs font-semibold py-2 rounded-lg flex items-center justify-center gap-1.5 transition-all cursor-pointer disabled:opacity-50 border-0"
        onclick={handleSaveHardening}
      >
        Save & Complete Setup <ArrowRight class="h-3.5 w-3.5" />
      </button>
    </div>

  {:else}
    <!-- STEP 4: SETUP COMPLETE / SUCCESS -->
    <div class="grow flex flex-col items-center justify-center text-center p-6 gap-6 bg-[#09090b]">
      <div class="flex items-center justify-center w-14 h-14 rounded-full bg-zinc-900 border border-zinc-800 text-[#06b6d4]">
        <ShieldCheck class="h-8 w-8" />
      </div>
      
      <div class="space-y-2">
        <h2 class="text-base font-bold text-white tracking-tight font-semibold">KeyVault Setup Complete!</h2>
        <p class="text-xs text-zinc-400 leading-relaxed max-w-[280px] mx-auto">
          Your credentials database is now successfully initialized and protected by your zero-knowledge Master Password.
        </p>
      </div>

      <div class="w-full bg-zinc-950 border border-zinc-900 rounded-lg p-3 text-left space-y-1.5 max-w-[280px] mx-auto">
        <p class="text-[10px] font-bold text-zinc-400 uppercase tracking-wider font-semibold">Your Dashboard Features:</p>
        <ul class="text-[10px] text-zinc-300 space-y-1 pl-4 list-disc">
          <li>Automated Google Drive Syncing</li>
          <li>Local cryptographic storage</li>
          <li>Secure key derivation via Argon2id</li>
        </ul>
      </div>

      <button
        onclick={handleFinish}
        class="w-full py-2 px-3 text-xs font-semibold bg-[#fafafa] text-[#18181b] rounded-lg hover:bg-zinc-200 active:scale-[0.98] transition-all duration-150 cursor-pointer max-w-[280px] border-0"
      >
        Get Started
      </button>
    </div>
  {/if}
</div>

<style>
  /* Optional CSS adjustments */
</style>
