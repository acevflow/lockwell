/** Hook that loads data when a component mounts. */
import { useEffect } from "react";

/**
 * Runs `load` once when the component mounts and passes its result to
 * `onLoaded`, or its failure to `onError`. Results that arrive after the
 * component has unmounted are ignored.
 *
 * `load`, `onLoaded` and `onError` must be stable between renders, as the
 * load runs again whenever one of them changes.
 */
export function useInitialLoad<T>(
  load: () => Promise<T>,
  onLoaded: (value: T) => void,
  onError: (cause: unknown) => void,
) {
  useEffect(() => {
    let active = true;
    load().then(
      (value) => {
        if (active) onLoaded(value);
      },
      (cause: unknown) => {
        if (active) onError(cause);
      },
    );
    return () => {
      active = false;
    };
  }, [load, onLoaded, onError]);
}
