#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod error;
mod storage;
mod vault_session;

use std::sync::Mutex;
use vault_session::VaultSession;

pub type VaultState = Mutex<VaultSession>;

fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(VaultSession::new()))
        .invoke_handler(tauri::generate_handler![
            commands::auth::unlock_vault,
            commands::auth::lock_vault,
            commands::auth::check_vault_exists,
            commands::auth::reset_vault,
            commands::auth::verify_password,
            commands::vault::get_vault,
            commands::vault::get_vault_items,
            commands::vault::save_vault_item,
            commands::vault::delete_vault_item,
            commands::vault::restore_vault_item,
            commands::vault::purge_vault_item,
            commands::vault::export_vault_csv,
            commands::vault::import_vault_csv,
            commands::vault::select_and_import_csv,
            commands::sync::merge_vaults,
            commands::sync::overwrite_local_vault,
            commands::sync::get_encrypted_vault_payload,
            commands::sync::decrypt_remote_vault,
            commands::sync::start_gdrive_auth,
            commands::sync::gdrive_token_request,
            commands::crypto::generate_credential,
            commands::crypto::generate_totp,
            commands::crypto::is_biometrics_supported,
            commands::crypto::authenticate_biometrics,
            commands::crypto::generate_recovery_key_payload,
            commands::crypto::recover_vault_with_key
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
