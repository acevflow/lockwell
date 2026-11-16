/** Password entry commands. */
import { call } from "./client";
import type { Entry, EntryInput, EntrySummary } from "./types";

export function listEntries(): Promise<EntrySummary[]> {
  return call<EntrySummary[]>("list_entries");
}

export function getEntry(id: string): Promise<Entry> {
  return call<Entry>("get_entry", { id });
}

export function createEntry(entry: EntryInput): Promise<string> {
  return call<string>("create_entry", { entry });
}

export function updateEntry(id: string, entry: EntryInput): Promise<void> {
  return call<void>("update_entry", { id, entry });
}

export function deleteEntry(id: string): Promise<void> {
  return call<void>("delete_entry", { id });
}
