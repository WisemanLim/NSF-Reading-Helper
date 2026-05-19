use std::sync::Arc;

use tauri::State;

use std::path::PathBuf;

use crate::core::analyzer;
use crate::core::db::Db;
use crate::core::error::AppResult;
use crate::core::models::{
    AnalyzeRequest, AnalyzeResult, HistoryItem, Note, SearchOptions, Settings,
    SymbolDetail, SymbolExample, SymbolHit, TtsOptions,
};
use crate::core::pack::{self, PackImportResult, PackManifest};
use crate::core::settings as settings_store;
use crate::core::tts;

pub struct AppState {
    pub db: Arc<Db>,
}

#[tauri::command]
pub async fn analyze(
    state: State<'_, AppState>,
    input: AnalyzeRequest,
) -> AppResult<AnalyzeResult> {
    let r = analyzer::analyze(&state.db, &input)?;
    state.db.save_history(&input, &r)?;
    Ok(r)
}

#[tauri::command]
pub async fn symbol_search(
    state: State<'_, AppState>,
    q: String,
    opts: Option<SearchOptions>,
) -> AppResult<Vec<SymbolHit>> {
    state.db.symbol_search(&q, opts.as_ref())
}

#[tauri::command]
pub async fn symbol_get(
    state: State<'_, AppState>,
    id: i64,
) -> AppResult<SymbolDetail> {
    state.db.symbol_detail(id)
}

#[tauri::command]
pub async fn symbol_examples(
    state: State<'_, AppState>,
    id: i64,
) -> AppResult<Vec<SymbolExample>> {
    state.db.symbol_examples(id)
}

#[tauri::command]
pub async fn history_list(state: State<'_, AppState>) -> AppResult<Vec<HistoryItem>> {
    state.db.history_list(100)
}

#[tauri::command]
pub async fn favorites_add(
    state: State<'_, AppState>,
    symbol_id: i64,
) -> AppResult<()> {
    state.db.favorites_add(symbol_id)
}

#[tauri::command]
pub async fn favorites_remove(
    state: State<'_, AppState>,
    symbol_id: i64,
) -> AppResult<()> {
    state.db.favorites_remove(symbol_id)
}

#[tauri::command]
pub async fn favorites_list(state: State<'_, AppState>) -> AppResult<Vec<SymbolHit>> {
    state.db.favorites_list()
}

#[tauri::command]
pub async fn notes_upsert(
    state: State<'_, AppState>,
    note: Note,
) -> AppResult<Note> {
    state.db.notes_upsert(note)
}

#[tauri::command]
pub async fn notes_list(state: State<'_, AppState>) -> AppResult<Vec<Note>> {
    state.db.notes_list()
}

#[tauri::command]
pub async fn settings_get(state: State<'_, AppState>) -> AppResult<Settings> {
    settings_store::load(&state.db)
}

#[tauri::command]
pub async fn settings_set(
    state: State<'_, AppState>,
    settings: Settings,
) -> AppResult<Settings> {
    settings_store::save(&state.db, &settings)?;
    Ok(settings)
}

#[tauri::command]
pub async fn tts_speak(text: String, opts: TtsOptions) -> AppResult<()> {
    tts::speak(&text, &opts).await
}

#[tauri::command]
pub async fn tts_stop() -> AppResult<()> {
    tts::stop().await
}

#[tauri::command]
pub async fn pack_import(
    state: State<'_, AppState>,
    path: String,
) -> AppResult<PackImportResult> {
    let p = PathBuf::from(path);
    pack::import_pack(&state.db, &p)
}

#[tauri::command]
pub async fn pack_export(
    state: State<'_, AppState>,
    path: String,
    name: Option<String>,
) -> AppResult<PackManifest> {
    let p = PathBuf::from(path);
    let pack_name = name.unwrap_or_else(|| "nsf-export".to_string());
    pack::export_pack(&state.db, &p, &pack_name)
}

#[tauri::command]
pub async fn fts_rebuild(state: State<'_, AppState>) -> AppResult<()> {
    state.db.rebuild_fts()
}
