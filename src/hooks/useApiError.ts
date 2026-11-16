import { useCallback, useState } from "react";
import { errorMessage, isLockedError } from "../api/client";

/**
 * Error state for screens of the unlocked vault. A `locked` error means the
 * core locked the vault (for example, after inactivity), so it calls
 * `onLocked` instead of showing a message.
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
