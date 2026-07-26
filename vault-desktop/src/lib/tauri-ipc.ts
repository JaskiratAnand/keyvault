import type { AppError } from './types/error.js';

const getTauriInvoke = () => (window as any).__TAURI__?.core?.invoke;

/**
 * Extracts a displayable error message from any caught command rejection.
 * Ensures compatibility with legacy string errors and structured objects.
 */
export function getErrorMsg(err: unknown): string {
  if (err && typeof err === 'object') {
    if ('message' in err && typeof (err as any).message === 'string') {
      return (err as any).message;
    }
    return JSON.stringify(err);
  }
  return String(err);
}

/**
 * Invokes a Tauri command and returns typed AppErrors upon failure.
 */
export async function safeInvoke<T>(
  cmd: string,
  args?: Record<string, any>,
): Promise<T> {
  const invoke = getTauriInvoke();
  if (!invoke) {
    throw {
      code: 'UNKNOWN_ERROR',
      message: 'Tauri backend not available in this environment.',
    } as AppError;
  }

  try {
    return await invoke(cmd, args);
  } catch (err: any) {
    if (err && typeof err === 'object' && 'code' in err && 'message' in err) {
      throw err as AppError;
    }
    throw {
      code: 'UNKNOWN_ERROR',
      message: typeof err === 'string' ? err : 'An unexpected error occurred.',
      details: JSON.stringify(err),
    } as AppError;
  }
}
