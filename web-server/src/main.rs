//! NSF Reading Helper — HTTP server.
//!
//! Backend storage is selected at runtime via `NSF_DATABASE_URL`:
//!   - empty / `sqlite:...`   → rusqlite (same engine the Tauri desktop uses)
//!   - `postgres://...`       → tokio-postgres + pg_trgm + fuzzystrmatch
//!
//! All HTTP handlers go through `DbBackend` which uniformly exposes async
//! methods, so the route layer is backend-agnostic.

mod backend;
mod core;
mod db_pg;
mod seed_pg;

use std::net::SocketAddr;
use std::path::PathBuf;

use axum::extract::{Path as AxPath, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::Value;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::backend::{analyze as analyze_with_backend, open_from_env, DbBackend};
use crate::core::error::AppError;
use crate::core::models::{
    AnalyzeRequest, AnalyzeResult, HistoryItem, Note, SearchOptions, Settings,
    SymbolDetail, SymbolExample, SymbolHit,
};

const SETTINGS_KEY: &str = "user_settings_v1";

#[derive(Clone)]
struct AppState {
    db: DbBackend,
}

struct ApiError(AppError);
impl From<AppError> for ApiError {
    fn from(e: AppError) -> Self {
        Self(e)
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.0 {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = serde_json::json!({ "error": self.0.to_string() });
        (status, Json(body)).into_response()
    }
}
type ApiResult<T> = Result<T, ApiError>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,axum=info,tower_http=info".into()),
        )
        .init();

    let data_dir = std::env::var("NSF_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/data"));
    std::fs::create_dir_all(&data_dir).expect("create data dir");
    let default_sqlite_path = data_dir.join("nsf-reading.sqlite");

    let db = open_from_env(&default_sqlite_path)
        .await
        .expect("open db backend");

    let state = AppState { db };

    let api = Router::new()
        .route("/healthz", get(healthz))
        .route("/expressions/analyze", post(analyze))
        .route("/symbols/search", get(symbol_search))
        .route("/symbols/:id", get(symbol_get))
        .route("/symbols/:id/examples", get(symbol_examples))
        .route("/history", get(history_list))
        .route("/favorites", get(favorites_list).post(favorites_add))
        .route("/favorites/:id", delete(favorites_remove))
        .route("/notes", get(notes_list).post(notes_upsert))
        .route("/settings", get(settings_get).put(settings_set))
        .route("/handwriting/classify", post(handwriting_classify));

    let app = Router::new()
        .nest("/api", api)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let bind = std::env::var("NSF_BIND").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let addr: SocketAddr = bind.parse().expect("parse bind");
    tracing::info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}

async fn healthz() -> &'static str {
    "ok"
}

async fn analyze(
    State(s): State<AppState>,
    Json(req): Json<AnalyzeRequest>,
) -> ApiResult<Json<AnalyzeResult>> {
    let mut r = analyze_with_backend(&s.db, &req).await?;
    let expr_id = s.db.save_history(&req, &r).await?;
    r.expression_id = Some(expr_id);
    Ok(Json(r))
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    domain: Option<String>,
    #[serde(rename = "symbolType")]
    symbol_type: Option<String>,
    limit: Option<i64>,
}
async fn symbol_search(
    State(s): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> ApiResult<Json<Vec<SymbolHit>>> {
    let opts = SearchOptions {
        domain: q.domain,
        symbol_type: q.symbol_type,
        limit: q.limit,
    };
    Ok(Json(s.db.symbol_search(&q.q, Some(&opts)).await?))
}

async fn symbol_get(
    State(s): State<AppState>,
    AxPath(id): AxPath<i64>,
) -> ApiResult<Json<SymbolDetail>> {
    Ok(Json(s.db.symbol_detail(id).await?))
}

async fn symbol_examples(
    State(s): State<AppState>,
    AxPath(id): AxPath<i64>,
) -> ApiResult<Json<Vec<SymbolExample>>> {
    Ok(Json(s.db.symbol_examples(id).await?))
}

async fn history_list(State(s): State<AppState>) -> ApiResult<Json<Vec<HistoryItem>>> {
    Ok(Json(s.db.history_list(100).await?))
}

#[derive(Deserialize)]
struct FavoriteBody {
    #[serde(rename = "symbolId")]
    symbol_id: i64,
}
async fn favorites_add(
    State(s): State<AppState>,
    Json(body): Json<FavoriteBody>,
) -> ApiResult<StatusCode> {
    s.db.favorites_add(body.symbol_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn favorites_remove(
    State(s): State<AppState>,
    AxPath(id): AxPath<i64>,
) -> ApiResult<StatusCode> {
    s.db.favorites_remove(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn favorites_list(State(s): State<AppState>) -> ApiResult<Json<Vec<SymbolHit>>> {
    Ok(Json(s.db.favorites_list().await?))
}

async fn notes_upsert(
    State(s): State<AppState>,
    Json(note): Json<Note>,
) -> ApiResult<Json<Note>> {
    Ok(Json(s.db.notes_upsert(note).await?))
}
async fn notes_list(State(s): State<AppState>) -> ApiResult<Json<Vec<Note>>> {
    Ok(Json(s.db.notes_list().await?))
}

async fn settings_get(State(s): State<AppState>) -> ApiResult<Json<Settings>> {
    let raw = s.db.settings_get(SETTINGS_KEY).await?;
    let settings: Settings = raw
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    Ok(Json(settings))
}

async fn settings_set(
    State(s): State<AppState>,
    Json(settings): Json<Settings>,
) -> ApiResult<Json<Settings>> {
    let raw = serde_json::to_string(&settings)
        .map_err(|e| AppError::Generic(format!("serialize settings: {e}")))?;
    s.db.settings_set(SETTINGS_KEY, &raw).await?;
    Ok(Json(settings))
}

/// Proxy to detexify.kirelabs.org for handwritten-symbol classification.
async fn handwriting_classify(Json(body): Json<Value>) -> Response {
    let endpoint = std::env::var("NSF_DETEXIFY_URL")
        .unwrap_or_else(|_| "https://detexify.kirelabs.org/classify.json".into());
    let client = match reqwest::Client::builder()
        .user_agent("nsf-reading-helper/0.1")
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };
    match client.post(&endpoint).json(&body).send().await {
        Ok(resp) => {
            let status = resp.status();
            match resp.json::<Value>().await {
                Ok(json) => (
                    StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::OK),
                    Json(json),
                )
                    .into_response(),
                Err(e) => (
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({ "error": format!("detexify response: {e}") })),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": format!("detexify upstream: {e}") })),
        )
            .into_response(),
    }
}
