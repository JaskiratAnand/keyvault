use serde::Serialize;
use thiserror::Error;

/// Structured application errors returned from Tauri commands.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Failed to acquire application state lock.")]
    StateLock,

    #[error("Vault is not unlocked.")]
    VaultNotUnlocked,

    #[error("No active salt found in the vault session.")]
    NoSaltFound,

    #[error("File system I/O error occurred.")]
    Io(#[from] std::io::Error),

    #[error("Invalid vault file: {0}")]
    InvalidVaultFile(String),

    #[error("Failed to parse or format JSON data.")]
    Serialization(#[from] serde_json::Error),

    #[error("Cryptographic operation failed: {0}")]
    Crypto(String),

    #[error("Authentication listener failed: {0}")]
    AuthListener(String),

    #[error("Authentication timed out.")]
    AuthTimeout,

    #[error("Google API request failed: {0}")]
    Network(#[from] ureq::Error),

    #[error("Google API returned HTTP error {status}: {body}")]
    Http { status: u16, body: String },

    #[error("Failed to generate password/credential: {0}")]
    Generator(String),

    #[error("Failed to generate TOTP code: {0}")]
    Totp(String),

    #[error("Biometric authentication is not supported on this platform.")]
    BiometricsUnsupported,

    #[error("Biometric authentication failed: {0}")]
    Biometric(String),

    #[error("No file was selected.")]
    NoFileSelected,

    #[error("Failed to import CSV: {0}")]
    CsvImport(String),

    #[error("Core vault operation failed: {0}")]
    VaultCore(String),
}

/// Helper struct used to flat-serialize AppError into a uniform camelCase JSON shape:
/// { "code": "SCREAMING_SNAKE", "message": "Human message", "details": "Stack/System log" }
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SerializedError {
    code: &'static str,
    message: String,
    details: Option<String>,
}

impl AppError {
    /// Maps each error variant to a machine-readable code string.
    pub fn code(&self) -> &'static str {
        match self {
            AppError::StateLock => "STATE_LOCK",
            AppError::VaultNotUnlocked => "VAULT_NOT_UNLOCKED",
            AppError::NoSaltFound => "NO_SALT_FOUND",
            AppError::Io(_) => "IO_ERROR",
            AppError::InvalidVaultFile(_) => "INVALID_VAULT_FILE",
            AppError::Serialization(_) => "SERIALIZATION_ERROR",
            AppError::Crypto(_) => "CRYPTO_ERROR",
            AppError::AuthListener(_) => "AUTH_LISTENER_ERROR",
            AppError::AuthTimeout => "AUTH_TIMEOUT",
            AppError::Network(_) => "NETWORK_ERROR",
            AppError::Http { .. } => "HTTP_ERROR",
            AppError::Generator(_) => "GENERATOR_ERROR",
            AppError::Totp(_) => "TOTP_ERROR",
            AppError::BiometricsUnsupported => "BIOMETRICS_UNSUPPORTED",
            AppError::Biometric(_) => "BIOMETRIC_ERROR",
            AppError::NoFileSelected => "NO_FILE_SELECTED",
            AppError::CsvImport(_) => "CSV_IMPORT_FAILED",
            AppError::VaultCore(_) => "VAULT_CORE_ERROR",
        }
    }

    /// Pulls out underlying details or library errors to assist with debugging.
    pub fn details(&self) -> Option<String> {
        match self {
            AppError::Io(err) => Some(err.to_string()),
            AppError::InvalidVaultFile(details) => Some(details.clone()),
            AppError::Serialization(err) => Some(err.to_string()),
            AppError::Crypto(details) => Some(details.clone()),
            AppError::AuthListener(details) => Some(details.clone()),
            AppError::Network(err) => Some(err.to_string()),
            AppError::Http { body, .. } => Some(body.clone()),
            AppError::Generator(details) => Some(details.clone()),
            AppError::Totp(details) => Some(details.clone()),
            AppError::Biometric(details) => Some(details.clone()),
            AppError::CsvImport(details) => Some(details.clone()),
            AppError::VaultCore(details) => Some(details.clone()),
            _ => None,
        }
    }
}

// Delegate serialization of the error to our SerializedError struct
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SerializedError {
            code: self.code(),
            message: self.to_string(),
            details: self.details(),
        }
        .serialize(serializer)
    }
}

// Automatic mappings for core library errors to simplify command logic
impl From<getrandom::Error> for AppError {
    fn from(err: getrandom::Error) -> Self {
        AppError::Crypto(format!("Random generation failure: {}", err))
    }
}

impl From<aes_gcm::Error> for AppError {
    fn from(err: aes_gcm::Error) -> Self {
        AppError::Crypto(format!("AES-GCM error: {}", err))
    }
}

impl From<argon2::Error> for AppError {
    fn from(err: argon2::Error) -> Self {
        AppError::Crypto(format!("Argon2 error: {}", err))
    }
}

impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::VaultCore(err)
    }
}

impl From<&str> for AppError {
    fn from(err: &str) -> Self {
        AppError::VaultCore(err.to_string())
    }
}
