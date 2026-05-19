//! Unified async DB dispatch — picks between rusqlite (local / desktop-parity)
//! and tokio-postgres (production) at runtime via `NSF_DATABASE_URL`.
//!
//! Sqlite paths run on a blocking task pool (rusqlite is synchronous); Postgres
//! is native async. Both expose the same method surface so handlers in `main.rs`
//! don't have to branch.

use std::sync::Arc;

use crate::core::db::Db as SqliteDb;
use crate::core::error::{AppError, AppResult};
use crate::core::models::{
    AnalyzeRequest, AnalyzeResult, AnalyzeSpeech, AnalyzeToken, HistoryItem, Note,
    SearchOptions, SymbolDetail, SymbolExample, SymbolHit, SymbolRow,
};
use crate::db_pg::PgDb;
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Clone)]
pub enum DbBackend {
    Sqlite(Arc<SqliteDb>),
    Postgres(Arc<PgDb>),
}

/// `NSF_DATABASE_URL` schemes:
///   sqlite:///abs/path/file.db | sqlite:relative.db | (unset → default sqlite at NSF_DATA_DIR)
///   postgres://user:pass@host:5432/dbname
pub async fn open_from_env(default_sqlite_path: &std::path::Path) -> AppResult<DbBackend> {
    let url = std::env::var("NSF_DATABASE_URL").unwrap_or_default();
    if url.starts_with("postgres://") || url.starts_with("postgresql://") {
        tracing::info!("opening postgres backend");
        let pg = PgDb::connect(&url).await?;
        pg.seed_if_empty().await?;
        Ok(DbBackend::Postgres(Arc::new(pg)))
    } else {
        let path = if let Some(rest) = url.strip_prefix("sqlite://") {
            std::path::PathBuf::from(rest)
        } else if let Some(rest) = url.strip_prefix("sqlite:") {
            std::path::PathBuf::from(rest)
        } else if !url.is_empty() {
            return Err(AppError::Generic(format!(
                "unrecognized NSF_DATABASE_URL: {url}"
            )));
        } else {
            default_sqlite_path.to_path_buf()
        };
        tracing::info!("opening sqlite backend at {}", path.display());
        let db = SqliteDb::open(&path)?;
        db.seed_if_empty()?;
        Ok(DbBackend::Sqlite(Arc::new(db)))
    }
}

// ── helpers ──────────────────────────────────────────────────────────────

async fn blocking<R, F>(f: F) -> AppResult<R>
where
    F: FnOnce() -> AppResult<R> + Send + 'static,
    R: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| AppError::Generic(format!("join: {e}")))?
}

// ── dispatch impl ────────────────────────────────────────────────────────

impl DbBackend {
    pub async fn symbol_search(
        &self,
        q: &str,
        opts: Option<&SearchOptions>,
    ) -> AppResult<Vec<SymbolHit>> {
        match self {
            DbBackend::Sqlite(d) => {
                let d = d.clone();
                let q = q.to_string();
                let opts = opts.cloned();
                blocking(move || d.symbol_search(&q, opts.as_ref())).await
            }
            DbBackend::Postgres(d) => d.symbol_search(q, opts).await,
        }
    }

    pub async fn symbol_detail(&self, id: i64) -> AppResult<SymbolDetail> {
        match self {
            DbBackend::Sqlite(d) => {
                let d = d.clone();
                blocking(move || d.symbol_detail(id)).await
            }
            DbBackend::Postgres(d) => d.symbol_detail(id).await,
        }
    }

    pub async fn symbol_examples(&self, id: i64) -> AppResult<Vec<SymbolExample>> {
        match self {
            DbBackend::Sqlite(d) => {
                let d = d.clone();
                blocking(move || d.symbol_examples(id)).await
            }
            DbBackend::Postgres(d) => d.symbol_examples(id).await,
        }
    }

    pub async fn save_history(
        &self,
        req: &AnalyzeRequest,
        result: &AnalyzeResult,
    ) -> AppResult<i64> {
        match self {
            DbBackend::Sqlite(d) => {
                let d = d.clone();
                let req = clone_request(req);
                let result = clone_result(result);
                blocking(move || d.save_history(&req, &result)).await
            }
            DbBackend::Postgres(d) => d.save_history(req, result).await,
        }
    }

    pub async fn history_list(&self, limit: i64) -> AppResult<Vec<HistoryItem>> {
        match self {
            DbBackend::Sqlite(d) => {
                let d = d.clone();
                blocking(move || d.history_list(limit)).await
            }
            DbBackend::Postgres(d) => d.history_list(limit).await,
        }
    }

    pub async fn favorites_add(&self, id: i64) -> AppResult<()> {
        match self {
            DbBackend::Sqlite(d) => {
                let d = d.clone();
                blocking(move || d.favorites_add(id)).await
            }
            DbBackend::Postgres(d) => d.favorites_add(id).await,
        }
    }

    pub async fn favorites_remove(&self, id: i64) -> AppResult<()> {
        match self {
            DbBackend::Sqlite(d) => {
                let d = d.clone();
                blocking(move || d.favorites_remove(id)).await
            }
            DbBackend::Postgres(d) => d.favorites_remove(id).await,
        }
    }

    pub async fn favorites_list(&self) -> AppResult<Vec<SymbolHit>> {
        match self {
            DbBackend::Sqlite(d) => {
                let d = d.clone();
                blocking(move || d.favorites_list()).await
            }
            DbBackend::Postgres(d) => d.favorites_list().await,
        }
    }

    pub async fn notes_upsert(&self, note: Note) -> AppResult<Note> {
        match self {
            DbBackend::Sqlite(d) => {
                let d = d.clone();
                let n = note;
                blocking(move || d.notes_upsert(n)).await
            }
            DbBackend::Postgres(d) => d.notes_upsert(note).await,
        }
    }

    pub async fn notes_list(&self) -> AppResult<Vec<Note>> {
        match self {
            DbBackend::Sqlite(d) => {
                let d = d.clone();
                blocking(move || d.notes_list()).await
            }
            DbBackend::Postgres(d) => d.notes_list().await,
        }
    }

    pub async fn settings_get(&self, key: &str) -> AppResult<Option<String>> {
        match self {
            DbBackend::Sqlite(d) => {
                let d = d.clone();
                let key = key.to_string();
                blocking(move || d.settings_get(&key)).await
            }
            DbBackend::Postgres(d) => d.settings_get(key).await,
        }
    }

    pub async fn settings_set(&self, key: &str, value: &str) -> AppResult<()> {
        match self {
            DbBackend::Sqlite(d) => {
                let d = d.clone();
                let k = key.to_string();
                let v = value.to_string();
                blocking(move || d.settings_set(&k, &v)).await
            }
            DbBackend::Postgres(d) => d.settings_set(key, value).await,
        }
    }

    pub async fn lookup_by_latex(&self, latex: &str) -> AppResult<Option<SymbolRow>> {
        match self {
            DbBackend::Sqlite(d) => {
                let d = d.clone();
                let l = latex.to_string();
                blocking(move || d.lookup_by_latex(&l)).await
            }
            DbBackend::Postgres(d) => d.lookup_by_latex(latex).await,
        }
    }

    pub async fn lookup_by_unicode(&self, ch: &str) -> AppResult<Option<SymbolRow>> {
        match self {
            DbBackend::Sqlite(d) => {
                let d = d.clone();
                let c = ch.to_string();
                blocking(move || d.lookup_by_unicode(&c)).await
            }
            DbBackend::Postgres(d) => d.lookup_by_unicode(ch).await,
        }
    }

    pub async fn rebuild_fts(&self) -> AppResult<()> {
        match self {
            DbBackend::Sqlite(d) => {
                let d = d.clone();
                blocking(move || d.rebuild_fts()).await
            }
            DbBackend::Postgres(d) => d.rebuild_fts().await,
        }
    }
}

// ── analyzer (async, backend-agnostic) ───────────────────────────────────

static LATEX_CMD: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\([A-Za-z]+)").unwrap());
static UNICODE_MATH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[\u{2200}-\u{22FF}\u{2190}-\u{21FF}\u{0391}-\u{03C9}∑∫∮∂∇∞°]").unwrap()
});

/// Same shape as `core::analyzer::analyze` but DB-agnostic and async — looks
/// each token up through [`DbBackend`].
pub async fn analyze(db: &DbBackend, req: &AnalyzeRequest) -> AppResult<AnalyzeResult> {
    let raw = req.raw_input.trim();
    let normalized_latex = Some(raw.to_string());

    let mut tokens = Vec::new();
    let mut idx: i64 = 0;

    for m in LATEX_CMD.captures_iter(raw) {
        let cmd = format!("\\{}", &m[1]);
        let resolved = db.lookup_by_latex(&cmd).await?;
        tokens.push(token(idx, cmd, "command", resolved));
        idx += 1;
    }
    for m in UNICODE_MATH.find_iter(raw) {
        let ch = m.as_str().to_string();
        let resolved = db.lookup_by_unicode(&ch).await?;
        tokens.push(token(idx, ch, "unicode", resolved));
        idx += 1;
    }

    let speech_seed = if tokens.is_empty() {
        raw.to_string()
    } else {
        tokens
            .iter()
            .map(|t| t.name_ko.clone().unwrap_or_else(|| t.token_text.clone()))
            .collect::<Vec<_>>()
            .join(", ")
    };

    Ok(AnalyzeResult {
        expression_id: None,
        normalized_latex,
        normalized_mathml: None,
        speech: AnalyzeSpeech {
            ko: speech_seed,
            en: None,
        },
        tokens,
    })
}

fn token(
    index: i64,
    text: String,
    role: &str,
    sym: Option<SymbolRow>,
) -> AnalyzeToken {
    AnalyzeToken {
        index,
        token_text: text,
        role: Some(role.into()),
        symbol_id: sym.as_ref().map(|s| s.id),
        name_ko: sym.as_ref().map(|s| s.canonical_name_ko.clone()),
        reading_ko: sym.as_ref().and_then(|s| s.reading_ko.clone()),
        meaning_ko: sym.as_ref().map(|s| s.short_meaning_ko.clone()),
    }
}

// ── owned-clone helpers (sqlite spawn_blocking needs 'static) ────────────

fn clone_request(r: &AnalyzeRequest) -> AnalyzeRequest {
    AnalyzeRequest {
        raw_input: r.raw_input.clone(),
        format: r.format.clone(),
        locale: r.locale.clone(),
    }
}

fn clone_result(r: &AnalyzeResult) -> AnalyzeResult {
    AnalyzeResult {
        expression_id: r.expression_id,
        normalized_latex: r.normalized_latex.clone(),
        normalized_mathml: r.normalized_mathml.clone(),
        speech: AnalyzeSpeech {
            ko: r.speech.ko.clone(),
            en: r.speech.en.clone(),
        },
        tokens: r.tokens.iter().map(clone_token).collect(),
    }
}

fn clone_token(t: &AnalyzeToken) -> AnalyzeToken {
    AnalyzeToken {
        index: t.index,
        token_text: t.token_text.clone(),
        role: t.role.clone(),
        symbol_id: t.symbol_id,
        name_ko: t.name_ko.clone(),
        reading_ko: t.reading_ko.clone(),
        meaning_ko: t.meaning_ko.clone(),
    }
}
