/** Hook shared by the screens of the unlocked vault to show command errors. */
import { useCallback, useState } from "react";
import { errorMessage, isLockedError } from "../api/client";

/**
 * Error state for screens of the unlocked vault. A `locked` error means the
 * core locked the vault (for example, after inactivity), so it calls
 * `onLocked` instead of showing a message.
 *
 * @param onLocked - Called when a reported error is `locked`. It should be
 *   stable between renders, because `report` changes with it.
 * @returns The current `error` message (or `null`), `report` to register a
 *   caught error, and `clear` to dismiss the message.
 */
export function useApiError(onLocked: () => void) {
  const [error, setError] = useState<string | null>(null);

  const report = useCallback(
    (cause: unknown) => {
      if (isLockedError(cause)) {
        onLocked();
      } else {
        setError(errorMessage(cause));
      }
    },
    [onLocked],
  );

  const clear = useCallback(() => setError(null), []);

  return { error, report, clear };
}
