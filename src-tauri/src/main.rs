//! Reclaim binary entry point; all logic lives in lib.rs (reclaim_lib).
// Prevents an extra console window on Windows in release. DO NOT REMOVE.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    reclaim_lib::run()
}
