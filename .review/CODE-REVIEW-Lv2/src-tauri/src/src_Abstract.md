# src-tauri/src/ — 디렉토리 개요 / Directory Abstract

> [← INDEX.md](../../INDEX.md)

## 파일 목록 / File List

| 파일 / File | LOC | 역할 / Role | 리뷰 |
|---|---|---|---|
| main.rs | 5 | OS 바이너리 진입점 | [main.rs.md](main.rs.md) |
| lib.rs | 58 | Tauri Builder + IPC 핸들러 등록 | [lib.rs.md](lib.rs.md) |
| commands.rs | 143 | `#[tauri::command]` 정의 (17개) | [commands.rs.md](commands.rs.md) |
| core/ | — | 도메인 로직 | [core/core_Abstract.md](core/core_Abstract.md) |

## 디렉토리 구조 / Structure Summary

`src-tauri/src/` 는 셸-레이어 + 명령 어댑터입니다. 비즈니스 로직은 `core/*` 모듈로 위임하고 이곳은:
1. `lib::run()` 에서 Tauri Builder 를 구성, 플러그인 등록, app_data_dir SQLite 오픈, 초기 시드.
2. `commands.rs` 에서 17개 IPC 명령을 정의 — 모두 얇은 위임(인풋→`core` 호출→리턴).
3. 모든 명령은 `AppResult<...>` 를 반환하고 `AppError: Serialize` 가 직렬화 가능 오류 메시지를 프런트에 노출.

## 주요 의존 관계 / Key Dependencies

- 들어오는 호출 / Inbound: Tauri webview 의 `invoke(...)`.
- 내려가는 호출 / Outbound: 전부 `core::*` (db, analyzer, pack, settings, tts).

## 위험 신호 / Risk Signals

1. `commands.rs` 는 입력 유효성 검증 없음. 길이/형식 제한이 `core` 측 SQL 바인딩에 의존(rusqlite 가 prepared statement 이라 SQLi는 닫힘).
2. `lib.rs::setup` 의 `db_path.unwrap_or_else(|_| std::env::temp_dir().join(...))` 는 권한 거부 시 임시 디렉토리로 강등 — 사용자가 데이터 손실을 인지하기 어려움.
