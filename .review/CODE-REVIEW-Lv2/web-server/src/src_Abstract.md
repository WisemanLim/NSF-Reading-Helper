# web-server/src/ — 디렉토리 개요 / Directory Abstract

> [← INDEX.md](../../INDEX.md)

## 파일 목록 / File List

| 파일 / File | LOC | 역할 / Role | 리뷰 |
|---|---|---|---|
| main.rs | 256 | Axum Router + 12 handlers + ApiError | [main.rs.md](main.rs.md) |
| backend.rs | 340 | `DbBackend` enum 디스패치 + async analyzer | [backend.rs.md](backend.rs.md) |
| db_pg.rs | **615** | Postgres backend (block-mode review) | [db_pg.rs.md](db_pg.rs.md) |
| seed_pg.rs | 132 | Postgres seed 설치자 | [seed_pg.rs.md](seed_pg.rs.md) |
| core.rs | 20 | `#[path]` 별칭 모듈 | (미선택 — alias-only) |

## 디렉토리 구조 / Structure Summary

라우터(`main.rs`)→ 백엔드 추상(`backend.rs`) → 구현(`db_pg.rs` / `core::db` via alias). `backend.rs` 의 `DbBackend` enum 이 sqlite/postgres 분기를 정확히 한 곳에 모읍니다. sqlite 측은 동기 API라 `tokio::task::spawn_blocking` 으로 비동기화.

## 주요 의존 관계 / Key Dependencies

- `main.rs` → `backend`, `core::{error, models}`.
- `backend.rs` → `core::db::Db` (sqlite), `db_pg::PgDb`, `core::models`.
- `db_pg.rs` ↔ `seed_pg.rs` (서로 호출).
- `core.rs` ↔ 데스크톱 `src-tauri/src/core/*` (path-alias).

## 위험 신호 / Risk Signals

1. `db_pg.rs::symbol_search` 의 단일 거대 CTE — alias 검색 부분이 LIKE만 사용하고 `or similarity(...) > 0.3` 은 `canonical_name_ko` 에만 적용. 영어 컬럼에는 트라이그램 적용 안 됨(데스크톱 검색과 미세하게 다른 결과).
2. `backend.rs::analyze` 가 LaTeX/Unicode regex 를 자체 보유 — `core::analyzer` 와 코드 중복. 패턴이 한쪽만 갱신될 위험.
3. `main.rs::handwriting_classify` 가 응답을 모두 `Json<Value>` 로 파싱 — 단순 프록시인 만큼 raw bytes pass-through 가 더 안전하고 빠를 수 있음.
