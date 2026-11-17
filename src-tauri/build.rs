//! Build script: runs `tauri-build`, which prepares the Tauri configuration,
//! capabilities and icons that `tauri::generate_context!` embeds (see
//! `src/lib.rs`).

fn main() {
    tauri_build::build()
}
