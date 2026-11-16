# src/hooks/useApiError.ts

Error state for the screens of the unlocked vault. `report(error)` shows the
message, except for a `locked` error, which calls `onLocked` so the app returns
to the unlock screen.
