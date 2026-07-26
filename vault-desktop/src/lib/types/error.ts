/**
 * Unified application error returned from Tauri commands.
 */
export interface AppError {
  /**
   * Machine-readable error code to drive UI logic.
   */
  code:
    | 'STATE_LOCK'
    | 'VAULT_NOT_UNLOCKED'
    | 'NO_SALT_FOUND'
    | 'IO_ERROR'
    | 'INVALID_VAULT_FILE'
    | 'SERIALIZATION_ERROR'
    | 'CRYPTO_ERROR'
    | 'AUTH_LISTENER_ERROR'
    | 'AUTH_TIMEOUT'
    | 'NETWORK_ERROR'
    | 'HTTP_ERROR'
    | 'GENERATOR_ERROR'
    | 'TOTP_ERROR'
    | 'BIOMETRICS_UNSUPPORTED'
    | 'BIOMETRIC_ERROR'
    | 'NO_FILE_SELECTED'
    | 'CSV_IMPORT_FAILED'
    | 'VAULT_CORE_ERROR'
    | 'UNKNOWN_ERROR';

  /**
   * Human-readable error message appropriate for display.
   */
  message: string;

  /**
   * Developer-facing technical detail (e.g. standard IO message, OS description).
   */
  details?: string;
}
