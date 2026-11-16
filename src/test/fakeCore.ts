/**
 * Stateful, in-memory stand-in for the core, for end-to-end tests of the
 * interface. It keeps the vault, its master password, the entries and the
 * clipboard, and answers like the core: commands of a locked vault fail with
 * `locked` and a wrong password with `wrong_password`.
 */
import type { Entry, EntryInput } from "../api/types";
import { commandError, mockCommands } from "./tauriMock";

/** A backup that `restore_backup` brings back, as the native dialog would. */
export interface FakeBackup {
  password: string;
  entries: Entry[];
}

export interface FakeCoreState {
  exists: boolean;
  unlocked: boolean;
  password: string;
  entries: Map<string, Entry>;
  /** What the core copied to the clipboard. */
  clipboard: string | null;
  /** The backup offered by the file dialog, if any. */
  backup: FakeBackup | null;
}

const EDITABLE_FIELDS = ["title", "username", "password", "url", "notes"];

/** A fixed timestamp: the fake never reads the clock. */
const NOW = Date.UTC(2026, 0, 1);

/** Installs the fake core and returns its state for the test to inspect. */
export function installFakeCore(
  initial: Partial<FakeCoreState> = {},
): FakeCoreState {
  const state: FakeCoreState = {
    exists: false,
    unlocked: false,
    password: "",
    entries: new Map(),
    clipboard: null,
    backup: null,
    ...initial,
  };
  let nextId = 0;

  function requireUnlocked() {
    if (!state.unlocked) throw commandError("locked", "the vault is locked");
  }

  function entryFrom(id: string, input: unknown): Entry {
    const fields = input as EntryInput;
    const unknown = Object.keys(fields).filter(
      (key) => !EDITABLE_FIELDS.includes(key),
    );
    if (unknown.length > 0) {
      throw commandError("invalid_input", `unknown field ${unknown[0]}`);
    }
    return { ...fields, id, createdAt: NOW, updatedAt: NOW };
  }

  function findEntry(id: unknown): Entry {
    requireUnlocked();
    const entry = state.entries.get(id as string);
    if (!entry) throw commandError("not_found", "entry not found");
    return entry;
  }

  mockCommands({
    vault_status: () => ({ exists: state.exists, unlocked: state.unlocked }),
    create_vault: ({ password }) => {
      if (state.exists) throw commandError("vault_exists", "a vault exists");
      Object.assign(state, {
        exists: true,
        unlocked: true,
        password: password as string,
      });
      return null;
    },
    unlock_vault: ({ password }) => {
      state.unlocked = false;
      if (password !== state.password) {
        throw commandError("wrong_password", "the password is incorrect");
      }
      state.unlocked = true;
      return null;
    },
    lock_vault: () => {
      state.unlocked = false;
      state.clipboard = null;
      return null;
    },
    change_master_password: ({ current, new: next }) => {
      requireUnlocked();
      if (current !== state.password) {
        throw commandError("wrong_password", "the password is incorrect");
      }
      state.password = next as string;
      return null;
    },
    touch_activity: () => null,
    list_entries: () => {
      requireUnlocked();
      return [...state.entries.values()]
        .sort((a, b) => a.title.localeCompare(b.title))
        .map(({ id, title, username, url, updatedAt }) => ({
          id,
          title,
          username,
          url,
          updatedAt,
        }));
    },
    get_entry: ({ id }) => findEntry(id),
    create_entry: ({ entry }) => {
      requireUnlocked();
      const id = String(++nextId).padStart(32, "0");
      state.entries.set(id, entryFrom(id, entry));
      return id;
    },
    update_entry: ({ id, entry }) => {
      findEntry(id);
      state.entries.set(id as string, entryFrom(id as string, entry));
      return null;
    },
    delete_entry: ({ id }) => {
      findEntry(id);
      state.entries.delete(id as string);
      return null;
    },
    copy_entry_field: ({ id, field }) => {
      const entry = findEntry(id);
      state.clipboard = entry[field as "username" | "password"];
      return null;
    },
    list_files: () => {
      requireUnlocked();
      return [];
    },
    restore_backup: ({ password }) => {
      if (!state.backup) return false;
      if (password !== state.backup.password) {
        throw commandError("wrong_password", "the password is incorrect");
      }
      Object.assign(state, {
        exists: true,
        unlocked: false,
        password: state.backup.password,
        entries: new Map(state.backup.entries.map((e) => [e.id, e])),
      });
      return true;
    },
  });
  return state;
}
