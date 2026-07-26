use crate::VaultState;
use crate::error::AppError;
use tauri::State;
use vault_core::vault::Vault;

#[tauri::command]
pub fn generate_credential(
    config: vault_core::generator::GeneratorConfig,
) -> Result<serde_json::Value, AppError> {
    let (credential, entropy) =
        vault_core::generator::generate(&config).map_err(|e| AppError::Generator(e.to_string()))?;
    Ok(serde_json::json!({
        "credential": credential,
        "entropy": entropy
    }))
}

#[tauri::command]
pub fn generate_totp(secret: String, timestamp: u64) -> Result<String, AppError> {
    vault_core::totp::generate_totp(&secret, timestamp).map_err(AppError::Totp)
}

#[tauri::command]
pub async fn is_biometrics_supported() -> bool {
    #[cfg(target_os = "macos")]
    {
        use localauthentication::LAContext;
        if let Ok(context) = LAContext::new() {
            let policy = localauthentication::LAPolicy::DeviceOwnerAuthenticationWithBiometrics;
            context.can_evaluate_policy(policy).is_ok()
        } else {
            false
        }
    }
    #[cfg(target_os = "windows")]
    {
        use windows::Security::Credentials::UI::{
            UserConsentVerifier, UserConsentVerifierAvailability,
        };
        match UserConsentVerifier::CheckAvailabilityAsync() {
            Ok(op) => match op.await {
                Ok(availability) => availability == UserConsentVerifierAvailability::Available,
                Err(_) => false,
            },
            Err(_) => false,
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        false
    }
}

#[tauri::command]
pub async fn authenticate_biometrics<R: tauri::Runtime>(
    _app_handle: tauri::AppHandle<R>,
) -> Result<bool, AppError> {
    #[cfg(target_os = "macos")]
    {
        use localauthentication::LAContext;
        let context = LAContext::new()
            .map_err(|e| AppError::Biometric(format!("Failed to create LAContext: {:?}", e)))?;
        let policy = localauthentication::LAPolicy::DeviceOwnerAuthentication;
        let reason = "Access secure vault credentials";

        let success = context
            .evaluate_policy(policy, reason)
            .map_err(|e| AppError::Biometric(format!("Biometrics evaluation failed: {:?}", e)))?;

        Ok(success)
    }
    #[cfg(target_os = "windows")]
    {
        use tauri::Manager;
        use windows::Security::Credentials::UI::{
            UserConsentVerificationResult, UserConsentVerifier,
        };
        use windows::Win32::Foundation::HWND;
        use windows::Win32::System::WinRT::IUserConsentVerifierInterop;
        use windows::core::HSTRING;

        let main_window = _app_handle.get_webview_window("main");
        let hwnd_raw = main_window
            .as_ref()
            .and_then(|w| w.hwnd().ok())
            .map(|h| h.0 as isize)
            .unwrap_or(0);

        let message = HSTRING::from("Access secure vault credentials");

        let mut op_opt = None;
        if hwnd_raw != 0 {
            if let Ok(interop) =
                windows::core::factory::<UserConsentVerifier, IUserConsentVerifierInterop>()
            {
                unsafe {
                    let hwnd = HWND(hwnd_raw as *mut std::ffi::c_void);
                    match interop.RequestVerificationForWindowAsync::<windows_future::IAsyncOperation<UserConsentVerificationResult>>(hwnd, &message) {
                        Ok(op) => op_opt = Some(op),
                        Err(e) => return Err(AppError::Biometric(format!("Failed to start verification interop: {}", e))),
                    }
                }
            }
        }

        if let Some(op) = op_opt {
            match op.await {
                Ok(result) => return Ok(result == UserConsentVerificationResult::Verified),
                Err(e) => return Err(AppError::Biometric(format!("Verification failed: {}", e))),
            }
        }

        match UserConsentVerifier::RequestVerificationAsync(&message) {
            Ok(op) => match op.await {
                Ok(result) => Ok(result == UserConsentVerificationResult::Verified),
                Err(e) => Err(AppError::Biometric(format!("Verification failed: {}", e))),
            },
            Err(e) => Err(AppError::Biometric(format!(
                "Failed to start verification: {}",
                e
            ))),
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Err(AppError::BiometricsUnsupported)
    }
}

#[derive(serde::Serialize)]
pub struct RecoveryPayload {
    recovery_salt: Vec<u8>,
    encrypted_key: Vec<u8>,
}

#[tauri::command]
pub fn generate_recovery_key_payload(
    recovery_key: String,
    state: State<'_, VaultState>,
) -> Result<RecoveryPayload, AppError> {
    let session = state.lock().map_err(|_| AppError::StateLock)?;
    let active_key = session.key.as_ref().ok_or(AppError::VaultNotUnlocked)?;

    let clean_key = recovery_key
        .replace("-", "")
        .replace(" ", "")
        .to_lowercase();

    let mut recovery_salt = [0u8; 16];
    vault_core::crypto::generate_random_bytes(&mut recovery_salt)?;

    let derived = vault_core::crypto::derive_key(
        &clean_key,
        &recovery_salt,
        vault_core::crypto::Argon2Params::default(),
    )?;

    let encryption_key = vault_core::crypto::EncryptionKey {
        inner: derived.inner.clone(),
    };

    let mut nonce = [0u8; 12];
    vault_core::crypto::generate_random_bytes(&mut nonce)?;

    let ciphertext = vault_core::crypto::encrypt(&encryption_key, active_key, &nonce)?;

    // Combine nonce and ciphertext into encrypted_key
    let mut encrypted_key = nonce.to_vec();
    encrypted_key.extend(ciphertext);

    Ok(RecoveryPayload {
        recovery_salt: recovery_salt.to_vec(),
        encrypted_key,
    })
}

#[tauri::command]
pub fn recover_vault_with_key<R: tauri::Runtime>(
    recovery_key: String,
    recovery_salt: Vec<u8>,
    encrypted_key: Vec<u8>,
    state: State<'_, VaultState>,
    app_handle: tauri::AppHandle<R>,
) -> Result<Vault, AppError> {
    if encrypted_key.len() < 12 {
        return Err(AppError::InvalidVaultFile(
            "Invalid encrypted key format".into(),
        ));
    }

    let clean_key = recovery_key
        .replace("-", "")
        .replace(" ", "")
        .to_lowercase();

    let derived = vault_core::crypto::derive_key(
        &clean_key,
        &recovery_salt,
        vault_core::crypto::Argon2Params::default(),
    )?;

    let encryption_key = vault_core::crypto::EncryptionKey {
        inner: derived.inner.clone(),
    };

    let (nonce, ciphertext) = encrypted_key.split_at(12);

    let active_key_bytes = vault_core::crypto::decrypt(&encryption_key, ciphertext, nonce)
        .map_err(|_| AppError::Crypto("Invalid recovery key".to_string()))?;

    // Now read the encrypted local vault file and decrypt it using the recovered active_key
    let encrypted_data = crate::storage::read_vault(&app_handle)?;
    if encrypted_data.is_empty() {
        return Err(AppError::InvalidVaultFile(
            "No local vault file found".into(),
        ));
    }

    if encrypted_data.len() < 32 || &encrypted_data[0..4] != b"KV01" {
        return Err(AppError::InvalidVaultFile(
            "Invalid vault file: missing magic header KV01".into(),
        ));
    }

    let salt = &encrypted_data[4..20];
    let payload_part = &encrypted_data[20..];

    if payload_part.len() < 12 {
        return Err(AppError::InvalidVaultFile("Invalid payload length".into()));
    }

    let (vault_nonce, vault_ciphertext) = payload_part.split_at(12);
    let key = vault_core::crypto::EncryptionKey {
        inner: active_key_bytes.clone(),
    };

    let decrypted_bytes = vault_core::crypto::decrypt(&key, vault_ciphertext, vault_nonce)
        .map_err(|_| AppError::Crypto("Failed to decrypt vault with recovered key".to_string()))?;

    let vault: Vault = serde_json::from_slice(&decrypted_bytes)?;

    // Save recovered session
    let mut session = state.lock().map_err(|_| AppError::StateLock)?;
    session.vault = Some(vault.clone());
    session.key = Some(active_key_bytes);
    session.salt = Some(salt.to_vec());

    Ok(vault)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault_session::VaultSession;
    use std::sync::Mutex;
    use tauri::Manager;
    use vault_core::generator::GeneratorConfig;
    use vault_core::vault::{SecureNote, VaultItem};

    fn create_mock_app() -> tauri::AppHandle<tauri::test::MockRuntime> {
        tauri::test::mock_builder()
            .manage(Mutex::new(VaultSession::new()))
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap()
            .handle()
            .clone()
    }

    fn get_unique_test_path(test_name: &str) -> std::path::PathBuf {
        let mut bytes = [0u8; 8];
        let _ = vault_core::crypto::generate_random_bytes(&mut bytes);
        let hex_str: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        std::env::temp_dir().join(format!("vault_test_{}_{}.enc", test_name, hex_str))
    }

    struct TestGuard {
        path: std::path::PathBuf,
    }

    impl TestGuard {
        fn new(test_name: &str) -> Self {
            let path = get_unique_test_path(test_name);
            crate::storage::set_test_path(path.clone());
            Self { path }
        }
    }

    impl Drop for TestGuard {
        fn drop(&mut self) {
            crate::storage::clear_test_path();
            if self.path.exists() {
                let _ = std::fs::remove_file(&self.path);
            }
            let tmp = self.path.with_extension("enc.tmp");
            if tmp.exists() {
                let _ = std::fs::remove_file(&tmp);
            }
        }
    }

    #[test]
    fn test_generate_credential_pin() {
        let config = GeneratorConfig::Pin { length: 6 };
        let res = generate_credential(config).unwrap();

        let credential = res.get("credential").unwrap().as_str().unwrap();
        let entropy = res.get("entropy").unwrap().as_f64().unwrap();

        assert_eq!(credential.len(), 6);
        assert!(credential.chars().all(|c| c.is_ascii_digit()));
        assert!(entropy > 0.0);
    }

    #[test]
    fn test_generate_totp_code() {
        // Base32 secret for "hello" is "NBSWY3DP"
        let secret = "NBSWY3DP".to_string();
        // Timestamp: 1234567890 (seconds)
        let totp = generate_totp(secret, 1234567890).unwrap();
        assert_eq!(totp.len(), 6);
        assert!(totp.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_recovery_flow_success_and_failure() {
        let _guard = TestGuard::new("recovery_flow");
        let app = create_mock_app();
        let state = app.state::<crate::VaultState>();

        // 1. Create a vault, unlock it, and add an item
        {
            let mut session = state.lock().unwrap();
            session.unlock("vault-pass", &app).unwrap();
            session
                .save_item(
                    VaultItem::SecureNote(SecureNote {
                        id: "test-note-rec".to_string(),
                        title: "Recovery Note".to_string(),
                        notes: "My secret note".to_string(),
                        tags: vec![],
                        created_at: "2026-07-20T00:00:00Z".to_string(),
                        updated_at: "2026-07-20T00:00:00Z".to_string(),
                    }),
                    &app,
                )
                .unwrap();
        }

        // 2. Generate recovery key payload
        let recovery_key = "MY-RECOVERY-KEY-123".to_string();
        let payload = generate_recovery_key_payload(recovery_key.clone(), state.clone()).unwrap();

        assert!(!payload.recovery_salt.is_empty());
        assert!(payload.encrypted_key.len() >= 12);

        // Lock the vault to clear active session
        {
            let mut session = state.lock().unwrap();
            session.lock();
        }

        // 3. Attempt recovery with wrong key
        let wrong_key = "WRONG-RECOVERY-KEY-999".to_string();
        let recover_fail = recover_vault_with_key(
            wrong_key,
            payload.recovery_salt.clone(),
            payload.encrypted_key.clone(),
            state.clone(),
            app.clone(),
        );
        assert!(recover_fail.is_err());
        match recover_fail {
            Err(AppError::Crypto(msg)) => assert!(msg.contains("Invalid recovery key")),
            other => panic!("Expected AppError::Crypto, got {:?}", other),
        }

        // 4. Recovery with correct key succeeds
        let recovered_vault = recover_vault_with_key(
            recovery_key,
            payload.recovery_salt,
            payload.encrypted_key,
            state.clone(),
            app.clone(),
        )
        .unwrap();

        assert_eq!(recovered_vault.items.len(), 1);
        assert_eq!(recovered_vault.items[0].id(), "test-note-rec");

        // Verify state is restored and active key works
        let session = state.lock().unwrap();
        assert!(session.vault.is_some());
        assert!(session.key.is_some());
        assert_eq!(session.get_vault().unwrap().items[0].id(), "test-note-rec");
    }
}
