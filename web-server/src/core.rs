//! Re-exports the desktop core via `#[path]` so the same dictionary/analyzer
//! logic powers both the Tauri app and this HTTP server.

#[path = "../../src-tauri/src/core/error.rs"]
pub mod error;

#[path = "../../src-tauri/src/core/models.rs"]
pub mod models;

#[path = "../../src-tauri/src/core/seed.rs"]
pub mod seed;

#[path = "../../src-tauri/src/core/db.rs"]
pub mod db;

#[path = "../../src-tauri/src/core/analyzer.rs"]
pub mod analyzer;

#[path = "../../src-tauri/src/core/settings.rs"]
pub mod settings;
