# src/api/client.ts

Typed access to the Tauri commands.

- `call()` runs a command through `invoke` and converts any rejection into an
  `ApiError` with the stable `code` and the `message` from the core.
- `toApiError()` normalizes anything thrown.
- `isLockedError()` detects the `locked` code.
- `isWrongPasswordError()` detects the `wrong_password` code.
- `errorMessage()` formats an error as a sentence for people.
