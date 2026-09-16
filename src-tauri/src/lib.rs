// Library entry — Tauri 2 convention is to expose a `run()` function that
// the platform-specific `main.rs` calls.

pub mod commands;
pub mod models;
pub mod platform;
pub mod safety;
pub mod scanners;

use commands::SessionState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(SessionState::default())
        .invoke_handler(tauri::generate_handler![
            commands::volume_stats,
            commands::scan,
            commands::is_app_running,
            commands::cleanup,
            commands::open_trash,
            commands::app_meta,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Mac Cleanup");
}
