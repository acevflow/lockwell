/** Vault commands: status, create, unlock, lock, change master password. */
import { call } from "./client";
import type { VaultStatus } from "./types";

/** Asks whether a vault exists and whether it is unlocked. */
export function getVaultStatus(): Promise<VaultStatus> {
  return call<VaultStatus>("vault_status");
}

/**
 * Creates the vault with `password` as master password and unlocks it.
 * Fails with `invalid_input` or `vault_exists`.
 */
export function createVault(password: string): Promise<void> {
  return call<void>("create_vault", { password });
}

/** Unlocks the vault. Fails with `wrong_password` if the password is wrong. */
export function unlockVault(password: string): Promise<void> {
  return call<void>("unlock_vault", { password });
}

/** Locks the vault; the core wipes its key and clears a copied secret. */
export function lockVault(): Promise<void> {
  return call<void>("lock_vault");
}

/**
 * Changes the master password. `current` is checked again by the core, which
 * fails with `wrong_password` if it does not match.
 */
export function changeMasterPassword(
  current: string,
  next: string,
): Promise<void> {
  return call<void>("change_master_password", { current, new: next });
}
