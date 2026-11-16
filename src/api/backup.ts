/** Encrypted backup commands. The core shows the native file dialogs. */
import { call } from "./client";

/** Asks where to save a backup; resolves to `false` if cancelled. */
export function createBackup(): Promise<boolean> {
  return call<boolean>("create_backup");
}

/**
 * Asks for a backup file and replaces the vault with it, leaving it locked.
 * `password` is the master password when the backup was made. Resolves to
 * `false` if cancelled.
 */
export function restoreBackup(password: string): Promise<boolean> {
  return call<boolean>("restore_backup", { password });
}
