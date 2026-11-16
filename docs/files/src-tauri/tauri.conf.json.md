# src-tauri/tauri.conf.json

Tauri configuration: product name, version (0.1.0), identifier
`com.acevflow.lockwell`, the Vite dev server and build output, the main window
(1000 × 680, at least 720 × 480) and the installers.

Security settings:

- `csp`: strict Content Security Policy for the built app. Scripts, styles,
  fonts and images only from the app itself (images also from `data:`),
  connections only to the Tauri IPC, and no objects, frames, forms or base URL
  changes.
- `devCsp`: the same for `tauri dev`, plus inline styles and the WebSocket
  that Vite needs for hot reload.
- `freezePrototype`: freezes `Object.prototype` to block prototype pollution.
