# web-server/ — 디렉토리 개요 / Directory Abstract

> [← INDEX.md](../INDEX.md)

## 파일 목록 / File List

| 파일 / File | 역할 / Role | 리뷰 |
|---|---|---|
| Cargo.toml | 크레이트 매니페스트 | (구조 정보) |
| migrations/postgres.sql | Postgres 스키마 (include_str!) | (데이터) |
| src/ | 서버 소스 | [src/src_Abstract.md](src/src_Abstract.md) |

## 디렉토리 구조 / Structure Summary

Axum 기반 HTTP API. 두 가지 백엔드를 런타임에 선택합니다:
- `NSF_DATABASE_URL=postgres://...` → `db_pg::PgDb` (native async, `pg_trgm`/`fuzzystrmatch`).
- 그 외 (empty/sqlite scheme) → `src-tauri/src/core/db::Db` (sync rusqlite, `spawn_blocking` 으로 비동기화).

`src/core.rs` 는 `#[path]` 어트리뷰트로 데스크톱 core 모듈을 그대로 컴파일합니다 — 한 줄도 복제하지 않고 동일 로직 사용.

## 주요 의존 관계 / Key Dependencies

- 외부 / External: axum, tokio-postgres + deadpool, reqwest (Detexify 프록시), rusqlite (bundled — 데스크톱 parity 이유).
- 입력 / Inputs: HTTP `/api/*`, env `NSF_*`.
- 출력 / Outputs: JSON, optional outbound HTTP to `detexify.kirelabs.org`.

## 위험 신호 / Risk Signals

1. **SSRF 위험 — `handwriting_classify`**: `NSF_DETEXIFY_URL` 이 운영자에 의해서만 설정되도록 신뢰. 만일 다른 경로로 사용자가 이 env 를 설정할 수 있는 환경이라면(쿠버네티스 ConfigMap, 멀티테넌트 등) 임의 HTTP 호출 가능. 타임아웃 10초는 설정되어 있음.
2. CORS `allow_origin=Any`, `allow_methods=Any`, `allow_headers=Any` — credential 없이 통신하므로 위험은 제한적이나 운영 정책 검토 권장.
3. `expect()` 다수 — 실패 시 panic으로 프로세스 종료. 개발 편의용으로는 OK, 운영 환경에선 graceful 종료 손실.
