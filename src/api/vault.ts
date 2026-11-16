/** Vault commands: status, create, unlock, lock, change master password. */
import { call } from "./client";
import type { VaultStatus } from "./types";

export function getVaultStatus(): Promise<VaultStatus> {
  return call<VaultStatus>("vault_status");
}

export function createVault(password: string): Promise<void> {
  return call<void>("create_vault", { password });
}

export function unlockVault(password: string): Promise<void> {
  return call<void>("unlock_vault", { password });
}

export function lockVault(): Promise<void> {
  return call<void>("lock_vault");
}

export function changeMasterPassword(
  current: string,
  next: string,
): Promise<void> {
  return call<void>("change_master_password", { current, new: next });
}
