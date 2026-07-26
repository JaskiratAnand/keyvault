use crate::VaultState;
use crate::error::AppError;
use std::io::{Read, Write};
use std::net::TcpListener;
use tauri::State;
use vault_core::vault::Vault;

fn open_browser(url: &str) {
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd")
        .args(&["/C", "start", "", url])
        .spawn();
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
}

#[tauri::command]
pub fn merge_vaults<R: tauri::Runtime>(
    remote_vault: Vault,
    state: State<'_, VaultState>,
    app_handle: tauri::AppHandle<R>,
) -> Result<Vault, AppError> {
    let mut session = state.lock().map_err(|_| AppError::StateLock)?;
    {
        let vault = session.vault.as_mut().ok_or(AppError::VaultNotUnlocked)?;
        vault.merge(remote_vault);
    }
    session.save(&app_handle)?;
    let vault = session.get_vault().ok_or(AppError::VaultNotUnlocked)?;
    Ok(vault.clone())
}

#[tauri::command]
pub fn overwrite_local_vault<R: tauri::Runtime>(
    remote_vault: Vault,
    remote_key: Vec<u8>,
    remote_salt: Vec<u8>,
    state: State<'_, VaultState>,
    app_handle: tauri::AppHandle<R>,
) -> Result<(), AppError> {
    let mut session = state.lock().map_err(|_| AppError::StateLock)?;
    session.vault = Some(remote_vault);
    session.key = Some(remote_key);
    session.salt = Some(remote_salt);
    session.save(&app_handle)?;
    Ok(())
}

#[tauri::command]
pub fn decrypt_remote_vault(
    payload: Vec<u8>,
    password: Option<String>,
    state: State<'_, VaultState>,
) -> Result<(Vault, Vec<u8>, Vec<u8>), AppError> {
    if payload.len() < 32 || &payload[0..4] != b"KV01" {
        return Err(AppError::InvalidVaultFile(
            "Missing magic header KV01".into(),
        ));
    }

    let salt = payload[4..20].to_vec();
    let payload_part = &payload[20..];

    if payload_part.len() < 12 {
        return Err(AppError::InvalidVaultFile("Invalid payload length".into()));
    }

    let (nonce, ciphertext) = payload_part.split_at(12);

    let key_bytes = if let Some(pass) = password {
        let derived = vault_core::crypto::derive_key(
            &pass,
            &salt,
            vault_core::crypto::Argon2Params::default(),
        )?;
        derived.inner.clone()
    } else {
        let session = state.lock().map_err(|_| AppError::StateLock)?;
        session.key.clone().ok_or(AppError::VaultNotUnlocked)?
    };

    let key = vault_core::crypto::EncryptionKey {
        inner: key_bytes.clone(),
    };

    let decrypted_bytes = vault_core::crypto::decrypt(&key, ciphertext, nonce)
        .map_err(|_| AppError::Crypto("Invalid password or key mismatch".to_string()))?;

    let vault: Vault = serde_json::from_slice(&decrypted_bytes)?;

    Ok((vault, salt, key_bytes))
}

#[tauri::command]
pub fn get_encrypted_vault_payload<R: tauri::Runtime>(
    app_handle: tauri::AppHandle<R>,
) -> Result<Vec<u8>, AppError> {
    crate::storage::read_vault(&app_handle)
}

#[tauri::command]
pub fn start_gdrive_auth(client_id: String) -> Result<String, AppError> {
    // Bind TcpListener to port 40305 dynamically
    let listener = TcpListener::bind("127.0.0.1:40305").map_err(|e| {
        AppError::AuthListener(format!("Failed to bind to local port 40305: {}", e))
    })?;

    let scopes = "https://www.googleapis.com/auth/drive.appdata https://www.googleapis.com/auth/userinfo.email https://www.googleapis.com/auth/userinfo.profile";

    // URL encode scopes manually to avoid external crate dependency
    let encoded_scopes = scopes
        .replace(" ", "%20")
        .replace("/", "%2F")
        .replace(":", "%3A");

    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri=http://127.0.0.1:40305&response_type=code&scope={}",
        client_id, encoded_scopes
    );

    // Open user's system web browser
    open_browser(&auth_url);

    // Set listener to non-blocking to allow timeout implementation
    listener
        .set_nonblocking(true)
        .map_err(|e| AppError::AuthListener(format!("Failed to set non-blocking mode: {}", e)))?;

    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(60);

    // Loop to continuously accept connections until we get the authorization code
    loop {
        if start.elapsed() > timeout {
            return Err(AppError::AuthTimeout);
        }

        let (mut stream, _) = match listener.accept() {
            Ok(res) => res,
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }
            Err(e) => {
                return Err(AppError::AuthListener(format!(
                    "Failed to accept incoming connection: {}",
                    e
                )));
            }
        };

        // Set a read timeout so a slow or keep-alive connection doesn't block us forever
        if stream
            .set_read_timeout(Some(std::time::Duration::from_secs(3)))
            .is_err()
        {
            continue;
        }

        let mut buffer = [0; 2048];
        let bytes_read = match stream.read(&mut buffer) {
            Ok(n) => n,
            Err(_) => continue, // Ignore read failures or timeouts on this socket
        };

        let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);

        // Ignore requests for favicon.ico to prevent breaking the flow
        if request_str.contains("GET /favicon.ico") {
            let response =
                "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
            stream.write_all(response.as_bytes()).ok();
            stream.flush().ok();
            continue;
        }

        // Simple parsing for "code=" in query parameters
        if let Some(code_pos) = request_str.find("code=") {
            let code_start = code_pos + 5;
            let mut code_end = request_str.len();
            if let Some(pos) = request_str[code_start..].find('&') {
                code_end = code_start + pos;
            } else if let Some(pos) = request_str[code_start..].find(' ') {
                code_end = code_start + pos;
            }
            let code = request_str[code_start..code_end].to_string();

            // Respond with a clean HTML success message matching KeyVault style
            let success_page = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\nConnection: close\r\n\r\n\
                <!doctype html>\
                <html>\
                <head>\
                    <meta charset=\"UTF-8\">\
                    <title>KeyVault Authentication Complete</title>\
                </head>\
                <body style=\"font-family: -apple-system, BlinkMacSystemFont, sans-serif; background-color: #09090b; color: #fafafa; display: flex; justify-content: center; align-items: center; height: 100vh; margin: 0;\">\
                    <div style=\"text-align: center; border: 1px solid #27272a; padding: 32px; border-radius: 8px; background-color: #18181b; max-width: 400px; box-shadow: 0 4px 10px rgba(0,0,0,0.3);\">\
                        <h2 style=\"color: #06b6d4; margin-top: 0; margin-bottom: 12px; font-weight: 600;\">Authentication Successful</h2>\
                        <p style=\"font-size: 15px; margin-bottom: 20px; line-height: 1.5;\">KeyVault has received your authorization code. You can close this tab and return to the application.</p>\
                        <span style=\"font-size: 12px; color: #a1a1aa; border: 1px solid #27272a; padding: 6px 12px; border-radius: 4px; background-color: #09090b;\">Connection Secured & Closed</span>\
                    </div>\
                </body>\
                </html>";

            stream.write_all(success_page.as_bytes()).ok();
            stream.flush().ok();

            return Ok(code);
        }
    }
}

#[tauri::command]
pub fn gdrive_token_request(
    client_id: String,
    grant_type: String,
    code: Option<String>,
    refresh_token: Option<String>,
    client_secret: Option<String>,
) -> Result<serde_json::Value, AppError> {
    let url = "https://oauth2.googleapis.com/token";

    let mut params = vec![
        ("client_id", client_id.as_str()),
        ("grant_type", grant_type.as_str()),
    ];

    // Keep temporary String values alive for references by storing them here
    let redirect_uri_str = "http://127.0.0.1:40305".to_string();
    let code_str;
    let refresh_token_str;
    let client_secret_str = client_secret.unwrap_or_default();

    if grant_type == "authorization_code" {
        if let Some(ref c) = code {
            code_str = c.clone();
            params.push(("code", code_str.as_str()));
        }
        params.push(("redirect_uri", redirect_uri_str.as_str()));
    } else if grant_type == "refresh_token" {
        if let Some(ref rt) = refresh_token {
            refresh_token_str = rt.clone();
            params.push(("refresh_token", refresh_token_str.as_str()));
        }
    }

    if !client_secret_str.trim().is_empty() {
        params.push(("client_secret", client_secret_str.as_str()));
    }

    let response_result = ureq::post(url).send_form(&params);
    let response = match response_result {
        Ok(res) => res,
        Err(ureq::Error::Status(code, response)) => {
            let error_body = response
                .into_string()
                .unwrap_or_else(|_| "Could not read error body".to_string());
            return Err(AppError::Http {
                status: code,
                body: error_body,
            });
        }
        Err(e) => return Err(AppError::Network(e)),
    };

    let data: serde_json::Value = response.into_json()?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault_session::VaultSession;
    use std::sync::Mutex;
    use tauri::Manager;
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

    fn create_test_note(id: &str, title: &str, updated_at: &str) -> VaultItem {
        VaultItem::SecureNote(SecureNote {
            id: id.to_string(),
            title: title.to_string(),
            notes: "Secret notes".to_string(),
            tags: vec![],
            created_at: "2026-07-20T00:00:00Z".to_string(),
            updated_at: updated_at.to_string(),
        })
    }

    #[test]
    fn test_overwrite_local_vault() {
        let _guard = TestGuard::new("overwrite_vault");
        let app = create_mock_app();
        let state = app.state::<crate::VaultState>();

        // 1. Setup session
        {
            let mut session = state.lock().unwrap();
            session.unlock("my-pass", &app).unwrap();
        }

        // 2. Prepare remote vault, key, and salt
        let mut remote_vault = Vault::new();
        remote_vault.add_item(create_test_note(
            "remote-note",
            "Remote Note Title",
            "2026-07-20T05:00:00Z",
        ));
        let remote_key = vec![1u8; 32];
        let remote_salt = vec![2u8; 16];

        // 3. Overwrite
        overwrite_local_vault(
            remote_vault.clone(),
            remote_key.clone(),
            remote_salt.clone(),
            state.clone(),
            app.clone(),
        )
        .unwrap();

        // 4. Verify in-memory state and file storage updates
        {
            let session = state.lock().unwrap();
            assert_eq!(session.key.as_ref().unwrap(), &remote_key);
            assert_eq!(session.salt.as_ref().unwrap(), &remote_salt);
            assert_eq!(session.get_vault().unwrap().items[0].id(), "remote-note");
        }

        // Verify it can be unlocked using remote key/password (if it matches storage format)
        let path = crate::storage::storage_path(&app);
        assert!(path.exists());
    }

    #[test]
    fn test_merge_vaults() {
        let _guard = TestGuard::new("merge_vaults");
        let app = create_mock_app();
        let state = app.state::<crate::VaultState>();

        // 1. Setup session with local item
        {
            let mut session = state.lock().unwrap();
            session.unlock("my-pass", &app).unwrap();
            session
                .save_item(
                    create_test_note("note-1", "Local Title", "2026-07-20T01:00:00Z"),
                    &app,
                )
                .unwrap();
        }

        // 2. Prepare remote vault with conflict and a new item
        let mut remote_vault = Vault::new();
        remote_vault.add_item(create_test_note(
            "note-1",
            "Remote Newer Title",
            "2026-07-20T02:00:00Z",
        ));
        remote_vault.add_item(create_test_note(
            "note-2",
            "New Remote Note",
            "2026-07-20T01:30:00Z",
        ));

        // 3. Merge
        let merged = merge_vaults(remote_vault, state.clone(), app.clone()).unwrap();

        // 4. Verify merged vault has updated note-1 title and added note-2
        assert_eq!(merged.items.len(), 2);

        let item_1 = merged.items.iter().find(|i| i.id() == "note-1").unwrap();
        if let VaultItem::SecureNote(sn) = item_1 {
            assert_eq!(sn.title, "Remote Newer Title");
        } else {
            panic!("Expected SecureNote");
        }

        let item_2 = merged.items.iter().find(|i| i.id() == "note-2").unwrap();
        if let VaultItem::SecureNote(sn) = item_2 {
            assert_eq!(sn.title, "New Remote Note");
        } else {
            panic!("Expected SecureNote");
        }
    }

    #[test]
    fn test_decrypt_remote_vault_flow() {
        let _guard = TestGuard::new("decrypt_remote");
        let app = create_mock_app();
        let state = app.state::<crate::VaultState>();

        // 1. Create a vault, unlock it, and add a note
        {
            let mut session = state.lock().unwrap();
            session.unlock("password123", &app).unwrap();
            session
                .save_item(
                    create_test_note("note-dec", "Decrypt Title", "2026-07-20T03:00:00Z"),
                    &app,
                )
                .unwrap();
        }

        // 2. Retrieve payload
        let payload = get_encrypted_vault_payload(app.clone()).unwrap();
        assert!(!payload.is_empty());

        // 3. Decrypt remote vault using explicit password
        let (decrypted_vault, salt, key) = decrypt_remote_vault(
            payload.clone(),
            Some("password123".to_string()),
            state.clone(),
        )
        .unwrap();

        assert_eq!(decrypted_vault.items.len(), 1);
        assert_eq!(decrypted_vault.items[0].id(), "note-dec");
        assert!(!salt.is_empty());
        assert!(!key.is_empty());

        // 4. Decrypt remote vault using wrong password fails
        let decrypt_fail = decrypt_remote_vault(
            payload.clone(),
            Some("wrongpassword".to_string()),
            state.clone(),
        );
        assert!(decrypt_fail.is_err());

        // 5. Decrypt remote vault using active session key (password Option is None)
        let (decrypted_vault_session, _, _) =
            decrypt_remote_vault(payload, None, state.clone()).unwrap();
        assert_eq!(decrypted_vault_session.items.len(), 1);
        assert_eq!(decrypted_vault_session.items[0].id(), "note-dec");
    }

    #[test]
    fn test_resolve_salt_mismatch_preserves_remote_salt_for_subsequent_unlocks() {
        let _guard = TestGuard::new("mismatch_preserve_salt");
        let app = create_mock_app();
        let state = app.state::<crate::VaultState>();

        // 1. Create a remote vault payload created with "remote-pass-123"
        let remote_pass = "remote-pass-123";
        let mut remote_salt = [0u8; 16];
        vault_core::crypto::generate_random_bytes(&mut remote_salt).unwrap();

        let remote_key = vault_core::crypto::derive_key(
            remote_pass,
            &remote_salt,
            vault_core::crypto::Argon2Params::default(),
        )
        .unwrap();

        let mut remote_vault = Vault::new();
        remote_vault.add_item(create_test_note(
            "remote-item-1",
            "Remote Note",
            "2026-07-26T12:00:00Z",
        ));

        let remote_vault_json = serde_json::to_vec(&remote_vault).unwrap();
        let mut nonce = [0u8; 12];
        vault_core::crypto::generate_random_bytes(&mut nonce).unwrap();
        let ciphertext = vault_core::crypto::encrypt(&remote_key, &remote_vault_json, &nonce).unwrap();

        let mut remote_payload = b"KV01".to_vec();
        remote_payload.extend_from_slice(&remote_salt);
        remote_payload.extend_from_slice(&nonce);
        remote_payload.extend(ciphertext);

        // 2. Initialize desktop session with a DIFFERENT local password
        {
            let mut session = state.lock().unwrap();
            session.unlock("local-pass-different", &app).unwrap();
        }

        // 3. Decrypt remote payload using remote password
        let (decrypted_vault, salt, key) = decrypt_remote_vault(
            remote_payload.clone(),
            Some(remote_pass.to_string()),
            state.clone(),
        )
        .unwrap();

        assert_eq!(salt, remote_salt.to_vec());

        // 4. Overwrite local vault with remote vault, key, AND salt
        overwrite_local_vault(
            decrypted_vault,
            key,
            salt,
            state.clone(),
            app.clone(),
        )
        .unwrap();

        // 5. Retrieve stored vault payload from disk
        let saved_payload = get_encrypted_vault_payload(app.clone()).unwrap();

        // 6. Verify that decrypting the stored payload with remote_pass SUCCEEDS!
        let (re_decrypted_vault, re_salt, _) = decrypt_remote_vault(
            saved_payload,
            Some(remote_pass.to_string()),
            state.clone(),
        )
        .unwrap();

        assert_eq!(re_salt, remote_salt.to_vec());
        assert_eq!(re_decrypted_vault.items.len(), 1);
        assert_eq!(re_decrypted_vault.items[0].id(), "remote-item-1");
    }
}
