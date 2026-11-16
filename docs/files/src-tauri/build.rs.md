# src-tauri/build.rs

Cargo build script. It calls `tauri_build::build()`, which reads the Tauri
configuration and capabilities and prepares the application context at
compile time.
