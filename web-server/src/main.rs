//! NSF Reading Helper — HTTP server.
//!
//! Routes mirror the Tauri commands so the React frontend can reuse the same
//! `ipc` shape over fetch().

mod core;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{Path as AxPath, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::core::analyzer;
use crate::core::db::Db;
use crate::core::error::AppError;
use crate::core::models::{
    AnalyzeRequest, AnalyzeResult, HistoryItem, Note, SearchOptions, Settings,
    SymbolDetail, SymbolExample, SymbolHit,
};
use crate::core::settings as settings_store;

#[derive(Clone)]
struct AppState {
    db: Arc<Db>,
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
    let db_path = data_dir.join("nsf-reading.sqlite");
    let db = Arc::new(Db::open(&db_path).expect("open db"));
    db.seed_if_empty().expect("seed dictionary");

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
        .route("/settings", get(settings_get).put(settings_set));

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
    let mut r = analyzer::analyze(&s.db, &req)?;
    let expr_id = s.db.save_history(&req, &r)?;
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
    Ok(Json(s.db.symbol_search(&q.q, Some(&opts))?))
}

async fn symbol_get(
    State(s): State<AppState>,
    AxPath(id): AxPath<i64>,
) -> ApiResult<Json<SymbolDetail>> {
    Ok(Json(s.db.symbol_detail(id)?))
}

async fn symbol_examples(
    State(s): State<AppState>,
    AxPath(id): AxPath<i64>,
) -> ApiResult<Json<Vec<SymbolExample>>> {
    Ok(Json(s.db.symbol_examples(id)?))
}

async fn history_list(
    State(s): State<AppState>,
) -> ApiResult<Json<Vec<HistoryItem>>> {
    Ok(Json(s.db.history_list(100)?))
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
    s.db.favorites_add(body.symbol_id)?;
    Ok(StatusCode::NO_CONTENT)
}
async fn favorites_remove(
    State(s): State<AppState>,
    AxPath(id): AxPath<i64>,
) -> ApiResult<StatusCode> {
    s.db.favorites_remove(id)?;
    Ok(StatusCode::NO_CONTENT)
}
async fn favorites_list(
    State(s): State<AppState>,
) -> ApiResult<Json<Vec<SymbolHit>>> {
    Ok(Json(s.db.favorites_list()?))
}

async fn notes_upsert(
    State(s): State<AppState>,
    Json(note): Json<Note>,
) -> ApiResult<Json<Note>> {
    Ok(Json(s.db.notes_upsert(note)?))
}
async fn notes_list(State(s): State<AppState>) -> ApiResult<Json<Vec<Note>>> {
    Ok(Json(s.db.notes_list()?))
}

async fn settings_get(State(s): State<AppState>) -> ApiResult<Json<Settings>> {
    Ok(Json(settings_store::load(&s.db)?))
}
async fn settings_set(
    State(s): State<AppState>,
    Json(settings): Json<Settings>,
) -> ApiResult<Json<Settings>> {
    settings_store::save(&s.db, &settings)?;
    Ok(Json(settings))
}
