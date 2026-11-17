//! Binary entry point of the desktop app. All logic lives in the
//! `lockwell_lib` library crate (`src/lib.rs`).

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    lockwell_lib::run()
}
