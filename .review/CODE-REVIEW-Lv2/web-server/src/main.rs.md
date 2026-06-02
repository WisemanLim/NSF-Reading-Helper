# web-server/src/main.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: Axum HTTP 서버. 라우트 정의, CORS/trace 미들웨어, `ApiError` IntoResponse 매핑, 12 handlers.
- LOC: 256
- 주요 export: `main()`
- 의존 모듈: `backend`, `core::error`, `core::models`, `core::seed` 등
- 외부 패키지: `axum`, `tokio`, `tower_http`, `serde`, `serde_json`, `reqwest`, `tracing*`
- 부수효과: 디렉토리 생성, DB open, TCP listen, 외부 HTTP 호출(Detexify proxy)
- 동시성: tokio 멀티 스레드

---

```rust path=web-server/src/main.rs lines=1-9
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
```

**분석 / Analysis**

- 모듈 4개. `core` 는 `core.rs` 의 `#[path]`-alias로 desktop core 를 그대로 사용.

```rust path=web-server/src/main.rs lines=15-35
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
```

**분석 / Analysis**

- `Path as AxPath` — Axum 의 path extractor 와 `std::path::Path` 충돌 회피.
- `analyze as analyze_with_backend` — local fn 과 backend fn 동명 충돌 회피.
- `SETTINGS_KEY` — `settings_store::KEY` 와 동일 문자열(`user_settings_v1`). 두 곳에서 별도 const → 일치 책임이 사람.

```rust path=web-server/src/main.rs lines=37-58
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
```

**분석 / Analysis**

- `ApiError` 가 `AppError` 의 wrapper — Axum `IntoResponse` 만 별도 impl. 좋은 패턴.
- HTTP 상태: `NotFound → 404`, 그 외 → `500`. **Validation 에러 분기 없음** — 잘못된 JSON 도 500 으로 떨어짐. 향후 `AppError::BadRequest(..)` 추가 시 매핑 보강 가능.
- 응답 본문: `{ "error": "..." }` — JS-side 가 `r.text()` 또는 `r.json().error` 둘 다 처리 가능(현 webClient 는 `r.text()` 사용).

```rust path=web-server/src/main.rs lines=60-78
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
```

**분석 / Analysis**

- `#[tokio::main]` — main 을 비동기 런타임 진입점으로 변환.
- 기본 데이터 디렉토리 `/data` — 컨테이너 친화. compose volume 마운트 예상.
- `expect("create data dir")` / `expect("open db backend")` — **운영 환경에서 panic 으로 종료**. 컨테이너 재시작이 자동 복구. 명시적 retry 부재.

```rust path=web-server/src/main.rs lines=80-103
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
```

**분석 / Analysis**

- 라우트 11개 (healthz 포함). `nest("/api", api)` → 모든 경로가 `/api/*` 아래.
- *주의*: `healthz` 가 `/api/healthz` 가 됨. README/배포 설정과 일치 확인 필요.
- **CORS Any/Any/Any** — credentials 미사용이므로 위험 제한적. 운영 환경에서는 origin 제한 권장.
- `TraceLayer::new_for_http()` — 모든 요청 자동 로그.

```rust path=web-server/src/main.rs lines=105-110
    let bind = std::env::var("NSF_BIND").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let addr: SocketAddr = bind.parse().expect("parse bind");
    tracing::info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
```

**분석 / Analysis**

- 환경변수 `NSF_BIND` 로 바인드 변경 가능. 컨테이너 친화.
- 세 expect 가 시작 단계 panic. graceful shutdown 미구현.

```rust path=web-server/src/main.rs lines=112-124
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
```

**분석 / Analysis**

- `healthz` 는 단순 텍스트. 컨테이너 헬스체크용.
- `analyze` 가 **`expression_id` 를 채워 반환** — 데스크톱 `commands::analyze` 와 달리 client 가 id 알 수 있음. 비대칭성.

```rust path=web-server/src/main.rs lines=126-144
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
```

**분석 / Analysis**

- `#[serde(rename = "symbolType")]` — URL query string 의 camelCase 키 → snake_case 필드. webClient 의 `qs()` 와 일치.
- 쿼리 길이 제한 없음 — 클라이언트가 매우 긴 q 보내면 levenshtein UDF 가 길어질 수 있음(sqlite 경우).

```rust path=web-server/src/main.rs lines=146-194
async fn symbol_get(...) -> ApiResult<Json<SymbolDetail>> { ... }
async fn symbol_examples(...) -> ApiResult<Json<Vec<SymbolExample>>> { ... }
async fn history_list(State(s): State<AppState>) -> ApiResult<Json<Vec<HistoryItem>>> {
    Ok(Json(s.db.history_list(100).await?))
}

#[derive(Deserialize)]
struct FavoriteBody { #[serde(rename = "symbolId")] symbol_id: i64 }
async fn favorites_add(...) -> ApiResult<StatusCode> {
    s.db.favorites_add(body.symbol_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn favorites_remove(...) -> ApiResult<StatusCode> {
    s.db.favorites_remove(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn favorites_list(...) -> ApiResult<Json<Vec<SymbolHit>>> { ... }
async fn notes_upsert(...) -> ApiResult<Json<Note>> { ... }
async fn notes_list(...) -> ApiResult<Json<Vec<Note>>> { ... }
```

**분석 / Analysis**

- 모든 핸들러 매우 얇음 — DbBackend 위임.
- `204 No Content` 응답을 favorites add/remove 가 반환 — webClient 의 `if (r.status === 204) return;` 분기와 일치.
- `history_list(100)` — 데스크톱과 동일 하드코딩.

```rust path=web-server/src/main.rs lines=196-215
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
```

**분석 / Analysis**

- 데스크톱 `settings_store::load/save` 와 동일 패턴 reproduced inline.
- Settings 직렬화 실패는 `Generic` 으로 매핑.

```rust path=web-server/src/main.rs lines=217-256
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
                ).into_response(),
                Err(e) => (
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({ "error": format!("detexify response: {e}") })),
                ).into_response(),
            }
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": format!("detexify upstream: {e}") })),
        ).into_response(),
    }
}
```

**분석 / Analysis**

- **외부 호출 프록시** — `NSF_DETEXIFY_URL` 로 endpoint 교체 가능.
  - 운영자가 신뢰한다는 가정.
  - 사용자 쿼리는 URL 결정에 영향 없음 → SSRF 위험은 운영자 환경에 한정.
- **타임아웃 10초** — 명시적. 무한 대기 차단.
- **user-agent** 명시 — 좋은 매너.
- **응답을 JSON 파싱 후 다시 JSON 보냄** — *비효율*: byte-stream pass-through 가 더 빠름. JSON 검증의 가치도 낮음(클라이언트가 정규화).
- 매 요청마다 새 `reqwest::Client` 생성 — *비효율*: connection pool 재사용 못 함. `OnceCell<reqwest::Client>` 로 재사용 권장.
- status code 매핑: upstream status 그대로 전달, 단 invalid status는 OK 로 fallback (`unwrap_or(OK)`) — 200 으로 가장하는 셈. **버그성**: 502 정상 응답이 200 으로 사용자에게 전달될 수 있음.
- response.json 실패 → 502 BAD_GATEWAY. 일관.

**짚어볼 점 / Watch-outs**
- 🟡 매 요청마다 `reqwest::Client` 생성.
- 🟡 status code fallback 이 OK 로 처리.
- 🟡 CORS Any/Any/Any 운영 정책 검토.
- 🟡 graceful shutdown 부재.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 명확한 라우터, ApiError 매핑 깔끔, env 기반 설정.
- 위험: 외부 호출 비효율, status 매핑 fallback.
- 개선 권고:
  1. `OnceCell<reqwest::Client>` 또는 lazy_static client.
  2. byte pass-through 프록시(`Bytes` 사용).
  3. `Validation` variant 추가하여 400 매핑.
  4. graceful shutdown: `axum::serve(listener, app).with_graceful_shutdown(...)` + SIGTERM.
  5. CORS origin 화이트리스트.
- 재읽기 권고 레벨: 2.
