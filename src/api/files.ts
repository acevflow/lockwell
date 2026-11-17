/** Encrypted file commands. The core shows the native file dialogs. */
import { call } from "./client";
import type { FileSummary } from "./types";

/** Lists the stored files (name, size and date only). */
export function listFiles(): Promise<FileSummary[]> {
  return call<FileSummary[]>("list_files");
}

/** Asks for files and adds them; resolves to the added files ([] if cancelled). */
export function importFiles(): Promise<FileSummary[]> {
  return call<FileSummary[]>("import_files");
}

/** Asks where to save a decrypted copy; resolves to `false` if cancelled. */
export function exportFile(id: string): Promise<boolean> {
  return call<boolean>("export_file", { id });
}

/** Deletes a stored file and its encrypted data. */
export function deleteFile(id: string): Promise<void> {
  return call<void>("delete_file", { id });
}
