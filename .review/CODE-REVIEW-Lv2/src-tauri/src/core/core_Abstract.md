# src-tauri/src/core/ — 디렉토리 개요 / Directory Abstract

> [← INDEX.md](../../../INDEX.md)

## 파일 목록 / File List

| 파일 / File | LOC | 역할 / Role | 리뷰 |
|---|---|---|---|
| mod.rs | 8 | 모듈 인덱스 | [mod.rs.md](mod.rs.md) |
| db.rs | **773** | SQLite 스키마 + 모든 쿼리 + UDF (block-mode) | [db.rs.md](db.rs.md) |
| models.rs | 178 | Serde DTO | [models.rs.md](models.rs.md) |
| analyzer.rs | 83 | LaTeX/유니코드 정규식 토크나이저 + 사전 조회 | [analyzer.rs.md](analyzer.rs.md) |
| pack.rs | 402 | `.nsfpack` zip 입·출력 | [pack.rs.md](pack.rs.md) |
| seed.rs | 160 | 초기 시드 설치자 (include_str!) | [seed.rs.md](seed.rs.md) |
| error.rs | 44 | `AppError`/`AppResult` + Serialize | [error.rs.md](error.rs.md) |
| settings.rs | 17 | 사용자 설정 KV 스토어 | [settings.rs.md](settings.rs.md) |
| tts.rs | 62 | OS-level TTS 스폰 | [tts.rs.md](tts.rs.md) |

## 디렉토리 구조 / Structure Summary

`core` 는 도메인 단일 출처. 데스크톱(`src-tauri/src/commands.rs`)과 웹(`web-server/src/core.rs` 의 `#[path]` 별칭) 모두 이 디렉토리를 직접 컴파일합니다. 모듈 책임은 명확히 갈립니다:
- **저장소** — `db` (스키마 v1+v2_fts, 쿼리, levenshtein UDF), `seed` (1회 적재), `settings` (key=`user_settings_v1` JSON), `pack` (외부 zip 패키지 import/export).
- **변환** — `analyzer` (입력→tokens).
- **인프라** — `models` (Serde 타입), `error` (`AppError`), `tts` (OS 명령).

## 주요 의존 관계 / Key Dependencies

- `analyzer`, `pack`, `seed`, `settings` 모두 `db` 에 의존.
- `models` 는 Serde DTO만 정의 — 다른 core 모듈과 commands.rs 가 함께 사용.
- `error` 는 모든 core 모듈에서 `?` 컨버전 대상.

## 위험 신호 / Risk Signals

1. `db.rs::symbol_search` 가 4단계 (exact / LIKE / FTS5 / Levenshtein) 검색을 in-process 머지 — 큰 사전에선 비효율, 한 Mutex 가드 안에서 4번 prepare.
2. `tts.rs` 의 macOS/Linux/Windows 브랜치는 각각 다른 escape 정책. Windows 분기는 따옴표만 처리.
3. `pack.rs::import_pack` 가 `alias_type` 으로 사용자 제공 문자열을 그대로 적재(`db.insert_alias(id, alias, ty, "ko")`) — DB 스키마 자체엔 unique 제약뿐이라 동작은 하나, alias_type 카테고리 오염 가능.
