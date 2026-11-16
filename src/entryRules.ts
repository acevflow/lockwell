/** Entry field limits and search, shared by the entry components. */
import type { EntrySummary } from "./api/types";

/** Field limits in characters, mirrored from `src-tauri/src/vault/entries.rs`. */
export const ENTRY_LIMITS = {
  title: 200,
  username: 500,
  password: 4096,
  url: 2048,
  notes: 20000,
} as const;

/** Case-insensitive match of `query` on title, username and website. */
export function filterEntries(
  entries: EntrySummary[],
  query: string,
): EntrySummary[] {
  const needle = query.trim().toLowerCase();
  if (needle === "") {
    return entries;
  }
  return entries.filter((entry) =>
    [entry.title, entry.username, entry.url].some((value) =>
      value.toLowerCase().includes(needle),
    ),
  );
}
