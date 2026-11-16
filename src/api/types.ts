/** Types shared with the Rust core. Field names match its camelCase JSON. */

/** Whether a vault exists and whether it is unlocked. */
export interface VaultStatus {
  exists: boolean;
  unlocked: boolean;
}

/** Fields of an entry as entered by the user. */
export interface EntryInput {
  title: string;
  username: string;
  password: string;
  url: string;
  notes: string;
}

/** An entry as listed: no password and no notes. */
export interface EntrySummary {
  id: string;
  title: string;
  username: string;
  url: string;
  /** Milliseconds since the Unix epoch. */
  updatedAt: number;
}

/** A complete entry, including its password. */
export interface Entry extends EntryInput {
  id: string;
  /** Milliseconds since the Unix epoch. */
  createdAt: number;
  /** Milliseconds since the Unix epoch. */
  updatedAt: number;
}

/** Stable error codes returned by the core. */
export type ErrorCode =
  | "wrong_password"
  | "locked"
  | "vault_exists"
  | "vault_not_found"
  | "invalid_input"
  | "not_found"
  | "corrupt"
  | "unsupported_version"
  | "crypto"
  | "storage"
  | "io"
  | "unknown";
