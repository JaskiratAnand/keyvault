use crate::error::AppError;
use vault_core::crypto::{Argon2Params, EncryptionKey, decrypt, derive_key, encrypt};
use vault_core::vault::{Vault, VaultItem};

pub struct VaultSession {
    pub vault: Option<Vault>,
    pub key: Option<Vec<u8>>,
    pub salt: Option<Vec<u8>>,
}

impl VaultSession {
    pub fn new() -> Self {
        Self {
            vault: None,
            key: None,
            salt: None,
        }
    }

    pub fn lock(&mut self) {
        if let Some(mut key) = self.key.take() {
            use zeroize::Zeroize;
            key.zeroize();
        }
        self.vault = None;
        self.salt = None;
    }

    pub fn save_item<R: tauri::Runtime>(
        &mut self,
        item: VaultItem,
        app_handle: &tauri::AppHandle<R>,
    ) -> Result<(), AppError> {
        let vault = self.vault.as_mut().ok_or(AppError::VaultNotUnlocked)?;
        let id = item.id().to_string();
        if vault.items.iter().any(|e| e.id() == id) {
            vault.update_item(item).map_err(AppError::from)?;
        } else {
            vault.add_item(item);
        }
        self.save(app_handle)?;
        Ok(())
    }

    pub fn delete_item<R: tauri::Runtime>(
        &mut self,
        id: &str,
        deleted_at: &str,
        app_handle: &tauri::AppHandle<R>,
    ) -> Result<(), AppError> {
        let vault = self.vault.as_mut().ok_or(AppError::VaultNotUnlocked)?;
        vault.delete_item(id, deleted_at).map_err(AppError::from)?;
        self.save(app_handle)?;
        Ok(())
    }

    pub fn restore_item<R: tauri::Runtime>(
        &mut self,
        id: &str,
        restored_at: &str,
        app_handle: &tauri::AppHandle<R>,
    ) -> Result<(), AppError> {
        let vault = self.vault.as_mut().ok_or(AppError::VaultNotUnlocked)?;
        vault
            .restore_item(id, restored_at)
            .map_err(AppError::from)?;
        self.save(app_handle)?;
        Ok(())
    }

    pub fn purge_item<R: tauri::Runtime>(
        &mut self,
        id: &str,
        purged_at: &str,
        app_handle: &tauri::AppHandle<R>,
    ) -> Result<(), AppError> {
        let vault = self.vault.as_mut().ok_or(AppError::VaultNotUnlocked)?;
        vault.purge_item(id, purged_at).map_err(AppError::from)?;
        self.save(app_handle)?;
        Ok(())
    }

    pub fn unlock<R: tauri::Runtime>(
        &mut self,
        password: &str,
        app_handle: &tauri::AppHandle<R>,
    ) -> Result<(), AppError> {
        let encrypted_data = crate::storage::read_vault(app_handle)?;

        if encrypted_data.is_empty() {
            // New vault: create an empty Vault
            let vault = Vault::new();

            // Derive a default key from the password to encrypt it
            let mut salt = [0u8; 16];
            vault_core::crypto::generate_random_bytes(&mut salt)?;
            let derived = derive_key(password, &salt, Argon2Params::default())?;

            self.vault = Some(vault);
            self.key = Some(derived.inner.clone());
            self.salt = Some(salt.to_vec());

            // Write the empty vault package to disk
            self.save(app_handle)?;
            return Ok(());
        }

        // Package format: "KV01" (4 bytes) + Salt (16 bytes) + Nonce (12 bytes) + Ciphertext
        if encrypted_data.len() < 32 || &encrypted_data[0..4] != b"KV01" {
            return Err(AppError::InvalidVaultFile(
                "Missing magic header KV01".into(),
            ));
        }

        let salt = &encrypted_data[4..20];
        let payload_part = &encrypted_data[20..];

        let derived = derive_key(password, salt, Argon2Params::default())?;

        if payload_part.len() < 12 {
            return Err(AppError::InvalidVaultFile("Invalid payload length".into()));
        }

        let (nonce, ciphertext) = payload_part.split_at(12);
        let key = EncryptionKey {
            inner: derived.inner.clone(),
        };

        let decrypted_bytes = decrypt(&key, ciphertext, nonce)
            .map_err(|_| AppError::Crypto("Invalid password".to_string()))?;

        let vault: Vault = serde_json::from_slice(&decrypted_bytes)?;

        self.vault = Some(vault);
        self.key = Some(derived.inner.clone());
        self.salt = Some(salt.to_vec());

        Ok(())
    }

    pub fn get_vault(&self) -> Option<&Vault> {
        self.vault.as_ref()
    }

    pub fn save<R: tauri::Runtime>(
        &self,
        app_handle: &tauri::AppHandle<R>,
    ) -> Result<(), AppError> {
        let vault = self.vault.as_ref().ok_or(AppError::VaultNotUnlocked)?;
        let key_bytes = self.key.as_ref().ok_or(AppError::VaultNotUnlocked)?;
        let salt = self.salt.as_ref().ok_or(AppError::NoSaltFound)?;

        let plaintext = serde_json::to_vec(vault)?;

        let mut nonce = [0u8; 12];
        vault_core::crypto::generate_random_bytes(&mut nonce)?;

        let key = EncryptionKey {
            inner: key_bytes.clone(),
        };
        let ciphertext = encrypt(&key, &plaintext, &nonce)?;

        let mut payload = b"KV01".to_vec();
        payload.extend_from_slice(salt);
        payload.extend_from_slice(&nonce);
        payload.extend(ciphertext);

        crate::storage::write_vault(app_handle, &payload)?;
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::path::PathBuf;
    use vault_core::vault::{SecureNote, VaultItem};

    pub fn create_mock_app() -> tauri::AppHandle<tauri::test::MockRuntime> {
        let app = tauri::test::mock_app();
        app.handle().clone()
    }

    fn get_unique_test_path(test_name: &str) -> PathBuf {
        let mut bytes = [0u8; 8];
        let _ = vault_core::crypto::generate_random_bytes(&mut bytes);
        let hex_str: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        std::env::temp_dir().join(format!("vault_test_{}_{}.enc", test_name, hex_str))
    }

    struct TestGuard {
        path: PathBuf,
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

    fn create_test_note() -> VaultItem {
        VaultItem::SecureNote(SecureNote {
            id: "test-note-1".to_string(),
            title: "My Secure Note".to_string(),
            notes: "Extremely secret note content".to_string(),
            tags: vec!["secret".to_string()],
            created_at: "2026-07-20T03:00:00Z".to_string(),
            updated_at: "2026-07-20T03:00:00Z".to_string(),
        })
    }

    #[test]
    fn test_new_session_is_locked() {
        let session = VaultSession::new();
        assert!(session.vault.is_none());
        assert!(session.key.is_none());
        assert!(session.salt.is_none());
        assert!(session.get_vault().is_none());
    }

    #[test]
    fn test_unlock_new_vault_creates_empty_vault_and_saves() {
        let _guard = TestGuard::new("unlock_new");
        let app = create_mock_app();

        let mut session = VaultSession::new();
        let unlock_result = session.unlock("mysecretpass", &app);
        assert!(unlock_result.is_ok());

        assert!(session.vault.is_some());
        assert!(session.key.is_some());
        assert!(session.salt.is_some());

        let vault = session.get_vault().unwrap();
        assert_eq!(vault.items.len(), 0);

        // Verify storage file exists now
        let path = crate::storage::storage_path(&app);
        assert!(path.exists());
    }

    #[test]
    fn test_unlock_existing_vault_succeeds() {
        let _guard = TestGuard::new("unlock_existing");
        let app = create_mock_app();

        // 1. Create a new vault and add an item, then save
        let mut session_1 = VaultSession::new();
        session_1.unlock("mysecretpass", &app).unwrap();

        let item = create_test_note();
        session_1.save_item(item.clone(), &app).unwrap();

        // 2. Open a second session and unlock with same password
        let mut session_2 = VaultSession::new();
        session_2.unlock("mysecretpass", &app).unwrap();

        // 3. Verify it decrypted and loaded the item
        let vault = session_2.get_vault().unwrap();
        assert_eq!(vault.items.len(), 1);
        assert_eq!(vault.items[0].id(), "test-note-1");
    }

    #[test]
    fn test_unlock_existing_vault_wrong_password_fails() {
        let _guard = TestGuard::new("wrong_password");
        let app = create_mock_app();

        // Create vault
        let mut session_1 = VaultSession::new();
        session_1.unlock("mysecretpass", &app).unwrap();
        session_1.save(&app).unwrap();

        // Attempt wrong password unlock
        let mut session_2 = VaultSession::new();
        let unlock_result = session_2.unlock("wrongpass", &app);
        assert!(unlock_result.is_err());
        match unlock_result {
            Err(AppError::Crypto(msg)) => assert!(msg.contains("Invalid password")),
            other => panic!("Expected AppError::Crypto, got {:?}", other),
        }
    }

    #[test]
    fn test_unlock_invalid_vault_file_fails() {
        let _guard = TestGuard::new("invalid_file");
        let app = create_mock_app();

        // Write junk data
        let path = crate::storage::storage_path(&app);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, b"NOT_A_VAULT_FILE_AT_ALL").unwrap();

        let mut session = VaultSession::new();
        let unlock_result = session.unlock("mysecretpass", &app);
        assert!(unlock_result.is_err());
        match unlock_result {
            Err(AppError::InvalidVaultFile(msg)) => {
                assert!(msg.contains("Missing magic header KV01"))
            }
            other => panic!("Expected AppError::InvalidVaultFile, got {:?}", other),
        }
    }

    #[test]
    fn test_lock_clears_session_state_and_zeroizes_key() {
        let _guard = TestGuard::new("lock_clears");
        let app = create_mock_app();

        let mut session = VaultSession::new();
        session.unlock("mysecretpass", &app).unwrap();
        assert!(session.key.is_some());

        session.lock();
        assert!(session.vault.is_none());
        assert!(session.key.is_none());
        assert!(session.salt.is_none());
    }

    #[test]
    fn test_vault_item_operations_workflow() {
        let _guard = TestGuard::new("item_operations");
        let app = create_mock_app();

        let mut session = VaultSession::new();
        session.unlock("mysecretpass", &app).unwrap();

        let item = create_test_note();

        // Save
        session.save_item(item.clone(), &app).unwrap();
        assert_eq!(session.get_vault().unwrap().items.len(), 1);

        // Update
        let mut updated_item = item.clone();
        if let VaultItem::SecureNote(ref mut note) = updated_item {
            note.title = "Updated Title".to_string();
            note.updated_at = "2026-07-20T03:05:00Z".to_string();
        }
        session.save_item(updated_item, &app).unwrap();

        let vault = session.get_vault().unwrap();
        assert_eq!(vault.items.len(), 1);
        if let VaultItem::SecureNote(ref note) = vault.items[0] {
            assert_eq!(note.title, "Updated Title");
        }

        // Delete
        session
            .delete_item("test-note-1", "2026-07-20T03:10:00Z", &app)
            .unwrap();
        let vault = session.get_vault().unwrap();
        assert_eq!(vault.items.len(), 0);
        assert_eq!(vault.trash.len(), 1);

        // Restore
        session
            .restore_item("test-note-1", "2026-07-20T03:15:00Z", &app)
            .unwrap();
        let vault = session.get_vault().unwrap();
        assert_eq!(vault.items.len(), 1);
        assert_eq!(vault.trash.len(), 0);

        // Purge
        session
            .purge_item("test-note-1", "2026-07-20T03:20:00Z", &app)
            .unwrap();
        let vault = session.get_vault().unwrap();
        assert_eq!(vault.items.len(), 0);
        assert_eq!(vault.trash.len(), 0);
    }
}
