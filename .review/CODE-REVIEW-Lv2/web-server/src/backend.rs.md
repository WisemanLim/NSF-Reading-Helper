# web-server/src/backend.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: `DbBackend` enum 으로 sqlite(blocking) ↔ postgres(native async) 디스패치 통일. + 백엔드-agnostic async analyzer.
- LOC: 340
- 주요 export: `DbBackend`, `open_from_env`, `analyze`
- 의존 모듈: `core::{db, error, models}`, `db_pg`
- 외부 패키지: `tokio`, `once_cell`, `regex`
- 부수효과: DB 호출, env 읽기
- 동시성: 모든 메서드 async, sqlite 분기는 `tokio::task::spawn_blocking`

---

```rust path=web-server/src/backend.rs lines=1-18
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
```

**분석 / Analysis**

- `use ... as SqliteDb` — `SqliteDb` 별명으로 가독성 향상. `PgDb` 와 대비.

```rust path=web-server/src/backend.rs lines=20-25
#[derive(Clone)]
pub enum DbBackend {
    Sqlite(Arc<SqliteDb>),
    Postgres(Arc<PgDb>),
}
```

**분석 / Analysis**

- `Arc<...>` 양쪽 — Clone 가능. AppState 가 Clone (Axum 요구).
- enum 디스패치 — vtable 없는 monomorphic. 컴파일러가 분기 인라인.

```rust path=web-server/src/backend.rs lines=27-53
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
```

**분석 / Analysis**

- URL 스킴 분기:
  - `postgres://` / `postgresql://` → Postgres.
  - `sqlite:///abs` → 슬래시 3개 = absolute.
  - `sqlite:relative` → relative path.
  - empty → 기본 sqlite path.
  - else → 에러.
- *주의*: `sqlite://` (no path) 입력 시 빈 PathBuf 가 됨. `open` 시 에러로 떨어짐.
- *흔한 실수*: postgres URL 파싱은 `tokio-postgres::Config::from_str` 가 책임 — 잘못된 URL 은 connect 단계에서 에러.

```rust path=web-server/src/backend.rs lines=55-66
async fn blocking<R, F>(f: F) -> AppResult<R>
where
    F: FnOnce() -> AppResult<R> + Send + 'static,
    R: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| AppError::Generic(format!("join: {e}")))?
}
```

**분석 / Analysis**

- **`spawn_blocking`** — sqlite 의 sync API 를 비동기 컨텍스트에서 안전 호출. 기본 풀(`tokio::task` 의 blocking pool) 사용.
- `'static` 바운드 — 클로저가 자체 데이터를 owned 해야 함. 호출자에서 `.clone()` 으로 데이터 owned 사본 만들어 전달 필요(아래 메서드 참조).

```rust path=web-server/src/backend.rs lines=69-237
impl DbBackend {
    pub async fn symbol_search(&self, q: &str, opts: Option<&SearchOptions>) -> AppResult<Vec<SymbolHit>> {
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

    pub async fn symbol_detail(&self, id: i64) -> AppResult<SymbolDetail> { ... }
    pub async fn symbol_examples(&self, id: i64) -> AppResult<Vec<SymbolExample>> { ... }
    pub async fn save_history(&self, req: &AnalyzeRequest, result: &AnalyzeResult) -> AppResult<i64> {
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
    pub async fn history_list(&self, limit: i64) -> AppResult<Vec<HistoryItem>> { ... }
    pub async fn favorites_add(&self, id: i64) -> AppResult<()> { ... }
    pub async fn favorites_remove(&self, id: i64) -> AppResult<()> { ... }
    pub async fn favorites_list(&self) -> AppResult<Vec<SymbolHit>> { ... }
    pub async fn notes_upsert(&self, note: Note) -> AppResult<Note> { ... }
    pub async fn notes_list(&self) -> AppResult<Vec<Note>> { ... }
    pub async fn settings_get(&self, key: &str) -> AppResult<Option<String>> { ... }
    pub async fn settings_set(&self, key: &str, value: &str) -> AppResult<()> { ... }
    pub async fn lookup_by_latex(&self, latex: &str) -> AppResult<Option<SymbolRow>> { ... }
    pub async fn lookup_by_unicode(&self, ch: &str) -> AppResult<Option<SymbolRow>> { ... }
    pub async fn rebuild_fts(&self) -> AppResult<()> { ... }
}
```

**분석 / Analysis**

- **모든 메서드가 패턴 반복**:
  - Sqlite 분기: Arc clone, 인자 owned 변환, blocking 호출.
  - Postgres 분기: 직접 await.
- *대안 패턴*: trait 객체(`Arc<dyn DbBackendImpl>`) 도 가능했지만, async trait 의 boxing 비용/`async_trait` 매크로 의존을 피하기 위해 enum 선택. **합리적 trade-off**.
- *흔한 실수*: `let d = d.clone();` 는 `Arc::clone` (ref count 증가). `*d.clone()` 이 아니라 Arc 자체 clone.
- 인자 owned 변환:
  - `q.to_string()` — &str → String.
  - `opts.cloned()` — Option<&T> → Option<T> (T: Clone).
  - `clone_request(req)` — &AnalyzeRequest → AnalyzeRequest. helper 사용.

```rust path=web-server/src/backend.rs lines=239-247
static LATEX_CMD: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\([A-Za-z]+)").unwrap());
static UNICODE_MATH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[\u{2200}-\u{22FF}\u{2190}-\u{21FF}\u{0391}-\u{03C9}∑∫∮∂∇∞°]").unwrap()
});
```

**분석 / Analysis**

- **`core::analyzer::analyze` 의 정규식과 동일** — **코드 중복**. 두 곳에서 동일한 패턴을 별도 컴파일.
- 한 곳에서 갱신하면 다른 곳을 잊을 위험. INDEX.md 4.1 참조.

```rust path=web-server/src/backend.rs lines=246-288
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
        speech: AnalyzeSpeech { ko: speech_seed, en: None },
        tokens,
    })
}
```

**분석 / Analysis**

- `core::analyzer::analyze` 의 **async 버전**. 차이점: 각 토큰 lookup 이 `await` (sqlite 는 blocking 호출).
- **N+1 await**: M 개 LaTeX 명령 + K 개 unicode → M+K await. 직렬 처리. Postgres 분기는 each lookup 이 별도 query → latency ~수 ms 씩 누적.
- *개선 여지*: `futures::stream::iter(...).buffer_unordered(8).collect()` 식 병렬화.

```rust path=web-server/src/backend.rs lines=290-340
fn token(index: i64, text: String, role: &str, sym: Option<SymbolRow>) -> AnalyzeToken { ... }

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
        speech: AnalyzeSpeech { ko: r.speech.ko.clone(), en: r.speech.en.clone() },
        tokens: r.tokens.iter().map(clone_token).collect(),
    }
}

fn clone_token(t: &AnalyzeToken) -> AnalyzeToken { ... }
```

**분석 / Analysis**

- **수동 deep clone 헬퍼들** — *왜*: `AnalyzeRequest`/`Result`/`Token` 가 `Clone` 을 derive 했더라면 헬퍼 불필요. `Deserialize` 만 derive 되어 있어 (`models.rs` 참조), 명시적 derive 누락.
- *흔한 실수*: derive 누락한 채 helper 작성 — 향후 필드 추가 시 helper 도 갱신 필요. 매우 fragile. **`Clone` derive 추가가 더 좋은 해결**.

**짚어볼 점 / Watch-outs**
- 🔴 **regex 코드 중복** (`core::analyzer` 와 동일).
- 🟡 **analyze 의 N+1 await** — 직렬 lookup.
- 🟡 **수동 clone 헬퍼** — derive Clone 으로 대체 가능.
- 🟡 **모든 메서드 패턴 반복** — macro 또는 trait 로 감소 가능.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: enum 디스패치로 async 통일, `spawn_blocking` 활용, 명확한 분기.
- 위험: 코드 중복, fragile clone helpers, 직렬 lookup.
- 개선 권고:
  1. `core::analyzer` 의 regex 상수를 `pub` 으로 export 하고 본 모듈에서 재사용.
  2. `AnalyzeRequest`/`Result`/`Token` 에 `Clone` derive 추가, helper 함수 제거.
  3. analyze 의 lookup 을 `buffer_unordered` 로 동시 8개 등 병렬화.
  4. 메서드 보일러플레이트를 macro_rules 로 단축.
- 재읽기 권고 레벨: 2.
