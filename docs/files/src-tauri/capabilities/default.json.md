# src-tauri/capabilities/default.json

Capability of the main window. Besides the app's own commands (allowed by
default), it only grants `core:event:allow-listen` and
`core:event:allow-unlisten`, which the interface needs for the `vault-locked`
event. It does not grant window, path, file system, dialog or clipboard
access: file dialogs and the clipboard are used from the core.
