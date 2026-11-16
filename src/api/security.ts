/** Activity, clipboard and lock-event bindings. */
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { call } from "./client";

/** Event the core emits when it locks the vault on its own. */
export const VAULT_LOCKED_EVENT = "vault-locked";

/** Why the core locked the vault. */
export type LockReason = "idle" | "systemLock";

/** A field of an entry that can be copied to the clipboard. */
export type CopyableField = "username" | "password";

/** Seconds after which the core clears a copied secret. */
export const CLIPBOARD_CLEAR_SECONDS = 30;

/** Records user activity, postponing the automatic lock. */
export function touchActivity(): Promise<void> {
  return call<void>("touch_activity");
}

/**
 * Copies a field of an entry to the clipboard. The value goes from the core
 * to the clipboard directly, without passing through the interface.
 */
export function copyEntryField(
  id: string,
  field: CopyableField,
): Promise<void> {
  return call<void>("copy_entry_field", { id, field });
}

/** Calls `handler` whenever the core locks the vault on its own. */
export function onVaultLocked(
  handler: (reason: LockReason) => void,
): Promise<UnlistenFn> {
  return listen<{ reason: LockReason }>(VAULT_LOCKED_EVENT, (event) =>
    handler(event.payload.reason),
  );
}
