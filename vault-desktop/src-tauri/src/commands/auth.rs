use crate::VaultState;
use crate::error::AppError;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn unlock_vault(
    password: String,
    app_handle: AppHandle,
    state: State<'_, VaultState>,
) -> Result<String, AppError> {
    let mut session = state.lock().map_err(|_| AppError::StateLock)?;
    session.unlock(&password, &app_handle)?;
    Ok("Unlocked".into())
}

#[tauri::command]
pub fn lock_vault(state: State<'_, VaultState>) -> Result<String, AppError> {
    let mut session = state.lock().map_err(|_| AppError::StateLock)?;
    session.lock();
    Ok("Locked".into())
}

#[tauri::command]
pub fn check_vault_exists(app_handle: AppHandle) -> bool {
    crate::storage::storage_path(&app_handle).exists()
}

#[tauri::command]
pub fn reset_vault(
    app_handle: AppHandle,
    state: State<'_, VaultState>,
) -> Result<String, AppError> {
    let mut session = state.lock().map_err(|_| AppError::StateLock)?;
    session.lock();
    let path = crate::storage::storage_path(&app_handle);
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok("Reset Completed".into())
}

#[tauri::command]
pub fn verify_password(password: String, state: State<'_, VaultState>) -> Result<bool, AppError> {
    let session = state.lock().map_err(|_| AppError::StateLock)?;
    let active_key = session.key.as_ref().ok_or(AppError::VaultNotUnlocked)?;
    let salt = session.salt.as_ref().ok_or(AppError::NoSaltFound)?;

    let derived = vault_core::crypto::derive_key(
        &password,
        salt,
        vault_core::crypto::Argon2Params::default(),
    )?;

    Ok(derived.inner == *active_key)
}
