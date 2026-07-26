use crate::VaultState;
use crate::error::AppError;
use tauri::{AppHandle, State};
use vault_core::vault::{Vault, VaultItem};

#[tauri::command]
pub fn get_vault(state: State<'_, VaultState>) -> Result<Vault, AppError> {
    let session = state.lock().map_err(|_| AppError::StateLock)?;
    let vault = session.get_vault().ok_or(AppError::VaultNotUnlocked)?;
    Ok(vault.clone())
}

#[tauri::command]
pub fn get_vault_items(state: State<'_, VaultState>) -> Result<Vec<VaultItem>, AppError> {
    let session = state.lock().map_err(|_| AppError::StateLock)?;
    let vault = session.get_vault().ok_or(AppError::VaultNotUnlocked)?;
    Ok(vault.items.clone())
}

#[tauri::command]
pub fn save_vault_item(
    item: VaultItem,
    state: State<'_, VaultState>,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    let mut session = state.lock().map_err(|_| AppError::StateLock)?;
    session.save_item(item, &app_handle)?;
    Ok(())
}

#[tauri::command]
pub fn delete_vault_item(
    id: String,
    deleted_at: String,
    state: State<'_, VaultState>,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    let mut session = state.lock().map_err(|_| AppError::StateLock)?;
    session.delete_item(&id, &deleted_at, &app_handle)?;
    Ok(())
}

#[tauri::command]
pub fn restore_vault_item(
    id: String,
    restored_at: String,
    state: State<'_, VaultState>,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    let mut session = state.lock().map_err(|_| AppError::StateLock)?;
    session.restore_item(&id, &restored_at, &app_handle)?;
    Ok(())
}

#[tauri::command]
pub fn purge_vault_item(
    id: String,
    purged_at: String,
    state: State<'_, VaultState>,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    let mut session = state.lock().map_err(|_| AppError::StateLock)?;
    session.purge_item(&id, &purged_at, &app_handle)?;
    Ok(())
}

#[tauri::command]
pub fn export_vault_csv(state: State<'_, VaultState>) -> Result<String, AppError> {
    let session = state.lock().map_err(|_| AppError::StateLock)?;
    let vault = session.get_vault().ok_or(AppError::VaultNotUnlocked)?;
    Ok(vault.to_csv())
}

#[tauri::command]
pub fn import_vault_csv(
    csv_text: String,
    current_time: String,
    state: State<'_, VaultState>,
    app_handle: AppHandle,
) -> Result<usize, AppError> {
    let mut session = state.lock().map_err(|_| AppError::StateLock)?;
    let vault = session.vault.as_mut().ok_or(AppError::VaultNotUnlocked)?;
    let count = vault
        .import_csv(&csv_text, &current_time)
        .map_err(AppError::CsvImport)?;
    session.save(&app_handle)?;
    Ok(count)
}

#[tauri::command]
pub fn select_and_import_csv(
    current_time: String,
    state: State<'_, VaultState>,
    app_handle: AppHandle,
) -> Result<usize, AppError> {
    let file_path = rfd::FileDialog::new()
        .add_filter("CSV Files", &["csv"])
        .pick_file();

    let path = match file_path {
        Some(p) => p,
        None => return Err(AppError::NoFileSelected),
    };

    let csv_text = std::fs::read_to_string(&path)?;

    let mut session = state.lock().map_err(|_| AppError::StateLock)?;
    let vault = session.vault.as_mut().ok_or(AppError::VaultNotUnlocked)?;
    let count = vault
        .import_csv(&csv_text, &current_time)
        .map_err(AppError::CsvImport)?;
    session.save(&app_handle)?;
    Ok(count)
}
