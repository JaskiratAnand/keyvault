use crate::error::AppError;
use std::cell::RefCell;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tauri::Manager;
use tauri::path::BaseDirectory;

thread_local! {
    static TEST_PATH_OVERRIDE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

#[cfg(test)]
pub fn set_test_path(path: PathBuf) {
    TEST_PATH_OVERRIDE.with(|p| *p.borrow_mut() = Some(path));
}

#[cfg(test)]
pub fn clear_test_path() {
    TEST_PATH_OVERRIDE.with(|p| *p.borrow_mut() = None);
}

pub fn storage_path<R: tauri::Runtime>(app_handle: &tauri::AppHandle<R>) -> PathBuf {
    let override_path = TEST_PATH_OVERRIDE.with(|p| p.borrow().clone());
    if let Some(p) = override_path {
        return p;
    }
    app_handle
        .path()
        .resolve("vault.enc", BaseDirectory::AppLocalData)
        .unwrap()
}

pub fn read_vault<R: tauri::Runtime>(
    app_handle: &tauri::AppHandle<R>,
) -> Result<Vec<u8>, AppError> {
    let path = storage_path(app_handle);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read(&path)?;
    Ok(data)
}

pub fn write_vault<R: tauri::Runtime>(
    app_handle: &tauri::AppHandle<R>,
    data: &[u8],
) -> Result<(), AppError> {
    let final_path = storage_path(app_handle);

    // Ensure target parent directory exists (e.g., AppLocalData folder)
    if let Some(parent) = final_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let temp_path = final_path.with_extension("enc.tmp");

    // Open/Create the temp file, write data, and force sync to physical disk
    {
        let mut file = fs::File::create(&temp_path)?;
        file.write_all(data)?;
        file.sync_all()?;
    }

    // Rename temp file to target destination atomically
    fs::rename(&temp_path, &final_path)?;

    Ok(())
}
