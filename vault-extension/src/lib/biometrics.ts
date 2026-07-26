export async function isBiometricsSupported(): Promise<boolean> {
  if (typeof window === 'undefined' || !window.PublicKeyCredential) {
    return false;
  }
  try {
    return await PublicKeyCredential.isUserVerifyingPlatformAuthenticatorAvailable();
  } catch {
    return false;
  }
}

export async function registerBiometrics(): Promise<boolean> {
  const supported = await isBiometricsSupported();
  if (!supported) {
    throw new Error(
      'Platform biometrics are not supported or enabled on this device.',
    );
  }

  const challenge = new Uint8Array(16);
  crypto.getRandomValues(challenge);

  const hostname =
    typeof window !== 'undefined' ? window.location.hostname : '';

  const createOptions: PublicKeyCredentialCreationOptions = {
    challenge,
    rp: {
      name: 'KeyVault',
      id: hostname || undefined,
    },
    user: {
      id: new Uint8Array([1, 2, 3, 4]),
      name: 'keyvault-user',
      displayName: 'KeyVault User',
    },
    pubKeyCredParams: [
      { type: 'public-key', alg: -7 }, // ES256
      { type: 'public-key', alg: -257 }, // RS256
    ],
    authenticatorSelection: {
      authenticatorAttachment: 'platform',
      userVerification: 'required',
    },
    timeout: 60000,
  };

  const credential = (await navigator.credentials.create({
    publicKey: createOptions,
  })) as PublicKeyCredential | null;

  if (!credential) {
    throw new Error('Biometric registration cancelled.');
  }

  const credentialId = Array.from(new Uint8Array(credential.rawId));
  if (typeof browser !== 'undefined' && browser.storage?.local) {
    await browser.storage.local.set({
      biometric_credential_id: credentialId,
      biometrics_enabled: true,
    });
  }

  return true;
}

export async function verifyBiometrics(): Promise<boolean> {
  if (typeof browser === 'undefined' || !browser.storage?.local) {
    throw new Error('Extension storage is unavailable.');
  }

  const data = await browser.storage.local.get([
    'biometric_credential_id',
    'biometrics_enabled',
  ]);
  if (!data.biometrics_enabled || !data.biometric_credential_id) {
    throw new Error('Biometrics are not enabled.');
  }

  const credentialId = new Uint8Array(data.biometric_credential_id as number[]);
  const challenge = new Uint8Array(16);
  crypto.getRandomValues(challenge);

  const getOptions: PublicKeyCredentialRequestOptions = {
    challenge,
    allowCredentials: [
      {
        type: 'public-key',
        id: credentialId,
      },
    ],
    userVerification: 'required',
    timeout: 60000,
  };

  const assertion = await navigator.credentials.get({
    publicKey: getOptions,
  });

  return !!assertion;
}

export async function disableBiometrics(): Promise<void> {
  if (typeof browser !== 'undefined' && browser.storage?.local) {
    await browser.storage.local.remove([
      'biometric_credential_id',
      'biometrics_enabled',
    ]);
  }
}

export async function isBiometricsEnabled(): Promise<boolean> {
  if (typeof browser !== 'undefined' && browser.storage?.local) {
    const data = await browser.storage.local.get(['biometrics_enabled']);
    return !!data.biometrics_enabled;
  }
  return false;
}
