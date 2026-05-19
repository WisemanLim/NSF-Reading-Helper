mod commands;
pub mod core;

use std::sync::Arc;
use tauri::Manager;

use crate::commands::AppState;
use crate::core::db::Db;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let handle = app.handle();
            let data_dir = handle
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::temp_dir().join("nsf-reading-helper"));
            std::fs::create_dir_all(&data_dir).ok();
            let db_path = data_dir.join("nsf-reading.sqlite");
            let db = Db::open(&db_path).expect("open db");
            db.seed_if_empty().expect("seed dictionary");
            app.manage(AppState { db: Arc::new(db) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::analyze,
            commands::symbol_search,
            commands::symbol_get,
            commands::symbol_examples,
            commands::history_list,
            commands::favorites_add,
            commands::favorites_remove,
            commands::favorites_list,
            commands::notes_upsert,
            commands::notes_list,
            commands::settings_get,
            commands::settings_set,
            commands::tts_speak,
            commands::tts_stop,
            commands::pack_import,
            commands::pack_export,
            commands::fts_rebuild,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
