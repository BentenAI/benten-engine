//! Phase-4-Foundation R6-FP-E build script. Conditionally invokes
//! `tauri_build::build()` when the `tauri` cargo feature is on. Under
//! the default-mode build path (no feature) this is a no-op so the
//! workspace doesn't pay the Tauri codegen cost.

fn main() {
    #[cfg(feature = "tauri")]
    tauri_build::build();
}
