# src-tauri/src/lib.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: Tauri Builder 셋업. 로깅, 데이터 디렉토리, DB 오픈+seed, 플러그인 등록, IPC 명령 등록.
- LOC: 58
- 주요 export: `run()` (also `pub mod core`, `mod commands`)
- 의존 모듈: `commands`, `core::db`, `tauri`, `tauri_plugin_*`
- 외부 패키지: `tauri`, `tauri_plugin_{opener,dialog,fs,updater}`, `tracing_subscriber`
- 부수효과: 로그 초기화, FS create_dir_all, DB open + seed
- 동시성: 전체가 `tauri::Builder` 내부 sync setup. DB 는 Arc 로 공유.

---

```rust path=src-tauri/src/lib.rs lines=1-9
mod commands;
pub mod core;

use std::sync::Arc;
use tauri::Manager;

use crate::commands::AppState;
use crate::core::db::Db;
```

**분석 / Analysis**

- `pub mod core` — 외부(`web-server/src/core.rs` 의 `#[path]` alias) 가 컴파일 가능하도록 public.
- `tauri::Manager` 는 `app.manage(...)` / `state::<T>()` 트레잇 메서드.

```rust path=src-tauri/src/lib.rs lines=10-18
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();
```

**분석 / Analysis**

- **모바일 진입점**: `mobile` cfg 가 set 되면(`Cargo.toml` 의 tauri-build feature) 본 함수가 Java/Swift bridge entry 가 됨. 데스크톱에선 `main.rs` 가 호출.
- 로그: `RUST_LOG` 환경변수 우선, 없으면 `info`. `tracing-subscriber` 의 표준 init.
- *흔한 실수*: `init()` 이 dev 와 prod 양쪽에서 호출됨. prod 릴리스 GUI 앱에선 stdout 가 보이지 않을 수 있음(Windows GUI subsystem) — 로그 파일로 redirect 미고려.

```rust path=src-tauri/src/lib.rs lines=19-23
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
```

**분석 / Analysis**

- 4개 플러그인:
  - `opener` — URL/파일 외부 프로그램으로 열기. 본 코드는 직접 호출 없음(데드 의존성 후보).
  - `dialog` — `SettingsPage.tsx` 의 file picker 가 사용.
  - `fs` — file system 권한 시스템 활성(`capabilities/default.json` 정책 적용).
  - `updater` — `lib/updater.ts` 가 사용. `Builder::new().build()` 는 기본 설정(endpoint 는 `tauri.conf.json` 에서).

```rust path=src-tauri/src/lib.rs lines=24-36
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
```

**분석 / Analysis**

- `app.handle().path().app_data_dir()` — Tauri 2 의 path resolver. OS별 user data dir:
  - macOS: `~/Library/Application Support/<bundleId>`
  - Windows: `%APPDATA%/<bundleId>`
  - Linux: `~/.local/share/<bundleId>`
- `unwrap_or_else(|_| std::env::temp_dir().join(...))` — 실패 시 임시 디렉토리로 강등. *짚어볼 점*: 사용자가 모름. 권한 거부 시 데이터가 임시폴더에 들어가 재부팅마다 휘발될 수 있음.
- `create_dir_all(&data_dir).ok()` — 이미 있으면 OK, 권한 없으면 ignore. 다음 단계 `Db::open` 이 실패하면 expect 가 panic.
- `Db::open(&db_path).expect("open db")` — **fatal panic** 으로 처리. 사용자에겐 충돌 화면. 더 우아한 처리: dialog 로 에러 표시 후 종료.
- `db.seed_if_empty().expect(...)` — 첫 부팅 시드. 동일 처리 정책.
- `Arc::new(db)` — `Db` 를 여러 명령에서 공유. `Db` 가 `Mutex<Connection>` 을 이미 가지고 있어 Arc 가 외부 동기화.
- `app.manage(AppState { db })` — Tauri 의 상태 저장소. 명령 핸들러에서 `State<'_, AppState>` 로 추출.

```rust path=src-tauri/src/lib.rs lines=37-58
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
```

**분석 / Analysis**

- `tauri::generate_handler![...]` — 매크로가 IPC 라우터 생성. JS 측 `invoke("analyze", ...)` 이름이 함수명과 매칭.
- `tauri::generate_context!()` — 빌드 시 `tauri.conf.json` 을 읽어 컴파일타임 컨텍스트 생성. 변경 시 재빌드.
- `.expect("error while running tauri application")` — 실행 실패 panic. 일반적으로 윈도우 생성 실패 등 fatal 만.
- *흔한 실수*: 명령 목록과 `commands.rs` 의 `#[tauri::command]` 함수 목록이 매뉴얼 동기화. 누락 시 JS invoke 가 에러로 떨어짐.

**짚어볼 점 / Watch-outs**
- panic 정책: open db / seed / run 실패 시 모두 panic. 운영 환경에선 dialog 로 메시지 후 종료가 더 친화적.
- `app_data_dir` 실패 시 silent fallback to temp — 데이터 손실 위험.
- `opener` 플러그인 등록은 하나 실제 사용처 없음.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: Tauri 2 표준 구조, 명령 등록 명료.
- 위험: 에러 패닉 정책, opener 데드 의존.
- 개선 권고:
  1. `app_data_dir` 실패를 Tauri dialog 로 표시 후 종료.
  2. `Db::open` 실패 시 동일 처리.
  3. opener 사용처 추가하거나 의존 제거.
- 재읽기 권고 레벨: 2.
