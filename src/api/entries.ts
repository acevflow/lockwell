/**
 * Password entry commands. All of them need an unlocked vault and fail with
 * `locked` otherwise.
 */
import { call } from "./client";
import type { Entry, EntryInput, EntrySummary } from "./types";

/** Lists the entries without passwords or notes. */
export function listEntries(): Promise<EntrySummary[]> {
  return call<EntrySummary[]>("list_entries");
}

/** Reads one complete entry, including its password. */
export function getEntry(id: string): Promise<Entry> {
  return call<Entry>("get_entry", { id });
}

/** Creates an entry and resolves to its new id. */
export function createEntry(entry: EntryInput): Promise<string> {
  return call<string>("create_entry", { entry });
}

/** Replaces every field of an entry. */
export function updateEntry(id: string, entry: EntryInput): Promise<void> {
  return call<void>("update_entry", { id, entry });
}

/** Deletes an entry for good. */
export function deleteEntry(id: string): Promise<void> {
  return call<void>("delete_entry", { id });
}
