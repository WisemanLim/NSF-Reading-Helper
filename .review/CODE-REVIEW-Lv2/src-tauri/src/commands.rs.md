# src-tauri/src/commands.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 17개의 Tauri IPC 명령 정의. 입력 deserialize → `core::*` 호출 → 결과 직렬화.
- LOC: 143
- 주요 export: `analyze`, `symbol_search`, `symbol_get`, `symbol_examples`, `history_list`, `favorites_add`, `favorites_remove`, `favorites_list`, `notes_upsert`, `notes_list`, `settings_get`, `settings_set`, `tts_speak`, `tts_stop`, `pack_import`, `pack_export`, `fts_rebuild` + `AppState`
- 의존 모듈: `core::{analyzer, db, error, models, pack, settings, tts}`
- 외부 패키지: `tauri`
- 부수효과: DB I/O, OS 명령 실행(tts), FS I/O(pack)
- 동시성: 모든 명령이 `async fn` 이지만 본문은 sync(spawn_blocking 미사용) — 데스크톱은 단일 사용자 + 작은 DB 라 무방.
- 테스트: 없음

---

```rust path=src-tauri/src/commands.rs lines=1-20
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
```

**분석 / Analysis**

- `use ... as settings_store` — 모듈 이름과 동일한 `settings: Settings` 변수 충돌 피하기 위함. 좋은 패턴.
- `AppState { db: Arc<Db> }` — Tauri `manage` 가 `Send + Sync + 'static` 요구. `Arc<Db>` 가 충족(`Db` 가 `Mutex<Connection>`).

```rust path=src-tauri/src/commands.rs lines=22-30
#[tauri::command]
pub async fn analyze(
    state: State<'_, AppState>,
    input: AnalyzeRequest,
) -> AppResult<AnalyzeResult> {
    let r = analyzer::analyze(&state.db, &input)?;
    state.db.save_history(&input, &r)?;
    Ok(r)
}
```

**분석 / Analysis**

- `#[tauri::command]` — IPC 핸들러 매크로. 인자명을 JS-side camelCase 와 매핑.
- `State<'_, AppState>` — Tauri 가 관리하는 상태 가져오기. 라이프타임 `'_` 는 함수 호출 동안만 유효.
- 흐름: 1) analyzer 가 tokens+speech seed 생성 2) DB 에 history insert.
- 실패 정책: 둘 중 하나라도 fail → `?` 가 AppError 로 전파 → JS 측 `invoke().catch(err)` 로 에러 객체.
- *짚어볼 점*: 분석은 성공해도 history 저장은 실패할 수 있음(예: 디스크 가득). 이 경우 사용자는 결과를 못 봄 — 사실 history 는 best-effort 가 더 적절.

```rust path=src-tauri/src/commands.rs lines=32-39
#[tauri::command]
pub async fn symbol_search(
    state: State<'_, AppState>,
    q: String,
    opts: Option<SearchOptions>,
) -> AppResult<Vec<SymbolHit>> {
    state.db.symbol_search(&q, opts.as_ref())
}
```

**분석 / Analysis**

- `opts: Option<SearchOptions>` + `opts.as_ref()` → `Option<&SearchOptions>` 로 변환. `Db::symbol_search` 의 시그니처와 일치.

```rust path=src-tauri/src/commands.rs lines=41-55
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
```

**분석 / Analysis**

- 직접 위임. 사용자 권한 검증 없음 — 단일 사용자 데스크톱이라 의미 없음.

```rust path=src-tauri/src/commands.rs lines=57-94
#[tauri::command]
pub async fn history_list(state: State<'_, AppState>) -> AppResult<Vec<HistoryItem>> {
    state.db.history_list(100)
}

#[tauri::command]
pub async fn favorites_add(state: State<'_, AppState>, symbol_id: i64) -> AppResult<()> {
    state.db.favorites_add(symbol_id)
}

#[tauri::command]
pub async fn favorites_remove(state: State<'_, AppState>, symbol_id: i64) -> AppResult<()> {
    state.db.favorites_remove(symbol_id)
}

#[tauri::command]
pub async fn favorites_list(state: State<'_, AppState>) -> AppResult<Vec<SymbolHit>> {
    state.db.favorites_list()
}

#[tauri::command]
pub async fn notes_upsert(state: State<'_, AppState>, note: Note) -> AppResult<Note> {
    state.db.notes_upsert(note)
}

#[tauri::command]
pub async fn notes_list(state: State<'_, AppState>) -> AppResult<Vec<Note>> {
    state.db.notes_list()
}
```

**분석 / Analysis**

- `history_list(100)` — 고정 100개. 클라이언트가 limit 을 받지 않음. 큰 history 시 페이지네이션 없음.
- 모든 메서드가 1:1 위임. 안티 패턴은 아니지만 ts-rs 등으로 자동 생성하면 보일러플레이트 절감 가능.

```rust path=src-tauri/src/commands.rs lines=96-108
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
```

**분석 / Analysis**

- `settings_set` 가 입력으로 받은 `settings` 를 그대로 반환 — 서버측 변환 없음. 클라이언트의 store 에 그대로 적용.

```rust path=src-tauri/src/commands.rs lines=110-118
#[tauri::command]
pub async fn tts_speak(text: String, opts: TtsOptions) -> AppResult<()> {
    tts::speak(&text, &opts).await
}

#[tauri::command]
pub async fn tts_stop() -> AppResult<()> {
    tts::stop().await
}
```

**분석 / Analysis**

- AppState 미사용 — DB 와 무관.
- `tts::speak` 의 OS process spawn 이 실패하면 fire-and-forget 으로 무시(`tts.rs` 내부).

```rust path=src-tauri/src/commands.rs lines=120-143
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
```

**분석 / Analysis**

- `PathBuf::from(path)` — JS 측 string → Rust path. *짚어볼 점*: 경로 검증 없음. Tauri 의 `tauri-plugin-fs` 권한(`capabilities/default.json`)이 실제 파일 접근을 제한해야 함. 본 명령은 직접 fs read/write 를 `core/pack.rs` 가 수행 — capabilities 에 의해 강제되지 않음. **사용자가 임의 경로를 통과시킬 수 있다는 점**은 데스크톱 단일 사용자 환경에선 위협 아니지만, Tauri capability 모델을 우회한다는 점을 주의.
- `pack_name.unwrap_or("nsf-export")` — 기본값.
- `fts_rebuild` — JS 가 호출하면 FTS 인덱스 재구축. SettingsPage 에 노출되지 않음(웹 전용 메서드 `webClient.ftsRebuild` 가 사용) — 데스크톱에서 호출 경로 미존재. 다른 미래 사용 가능성을 위한 hook.

**짚어볼 점 / Watch-outs**
- pack_import/export 의 path 가 임의 입력 — Tauri capability bypass 가능. 데스크톱 단독 사용자 환경에선 OK.
- 모든 명령이 `async` 지만 본문은 sync(Mutex 락) — Tokio 런타임에서 잠시 블록될 수 있음. 데스크톱 단일 사용자라 영향 미미.
- `tauri::generate_handler![...]` 와 본 파일의 명령 목록 매뉴얼 동기화.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 얇은 어댑터, 시그니처 단순, 책임 분리.
- 위험: capability bypass, async-but-sync, 한정 분량의 history(100).
- 개선 권고:
  1. pack 경로를 Tauri dialog 가 반환한 검증된 path 만 허용(클라이언트 신뢰).
  2. `tokio::task::spawn_blocking` 으로 DB 호출 감싸기(여러 사용자 환경 대비).
  3. history limit 매개변수화.
- 재읽기 권고 레벨: 2.
