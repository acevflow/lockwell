# src-tauri/src/main.rs

Entry point of the desktop binary. It calls `lockwell_lib::run()`. In release
builds it uses the Windows GUI subsystem so that no console window opens.
