/** Hook that reacts to the core locking the vault without a user request. */
import { useEffect } from "react";
import { onVaultLocked, type LockReason } from "../api/security";

/**
 * Calls `onLocked` when the core locks the vault on its own (inactivity or
 * Windows lock), so the interface drops its decrypted state at once.
 *
 * `onLocked` should be stable between renders: the listener is registered
 * again whenever it changes. If the component unmounts before the listener is
 * ready, the listener is removed as soon as it is.
 */
export function useVaultLockedEvent(onLocked: (reason: LockReason) => void) {
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let active = true;

    onVaultLocked(onLocked).then(
      (stop) => {
        if (active) {
          unlisten = stop;
        } else {
          stop();
        }
      },
      (error: unknown) => {
        // Without the event the interface still learns about the lock on
        // its next command, which fails with `locked`.
        console.error("lockwell: cannot listen for vault lock events", error);
      },
    );

    return () => {
      active = false;
      unlisten?.();
    };
  }, [onLocked]);
}
