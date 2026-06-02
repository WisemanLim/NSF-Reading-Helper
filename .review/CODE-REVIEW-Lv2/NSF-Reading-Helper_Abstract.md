# NSF-Reading-Helper/ — 프로젝트 개요 / Project Abstract

> [← INDEX.md](INDEX.md)

## 한 줄 정의 / One-line

자연과학 수식(LaTeX/MathML/AsciiMath/Unicode)을 분석해 한국어로 읽어주고 기호 사전을 제공하는 멀티-타깃 앱: Tauri 데스크톱 + React PWA + Rust HTTP 서버 + Python OCR 사이드카.
Multi-target app that parses scientific formulas and reads them aloud in Korean while exposing a symbol dictionary: Tauri desktop + React PWA + Rust HTTP server + Python OCR sidecar.

## 디렉토리 구조 / Structure Summary

- `src/` — Vite + React 19 + Tailwind 프런트엔드. PWA로 동작하고 동일 코드가 Tauri WebView에서도 호스팅됩니다 / Vite + React 19 + Tailwind frontend. Acts as a PWA and also hosts inside the Tauri WebView.
- `src-tauri/` — 데스크톱 셸 + 핵심 도메인 로직 (Rust). `core::db` 가 SQLite 단일 소스 / Desktop shell + core domain logic (Rust). `core::db` is the SQLite source of truth.
- `web-server/` — Axum 기반 HTTP API. 동일한 `core` 코드를 `#[path]` 별칭으로 재사용하고, Postgres 백엔드도 지원 / Axum HTTP API; reuses the same `core` modules via `#[path]` aliases and adds a Postgres backend.
- `ocr/` — pix2tex 래핑 FastAPI. 데스크톱에서는 호출하지 않고 웹 스택에서만 활용 / FastAPI wrapping pix2tex; reachable from the web stack, not from desktop.
- `deploy/` — nginx 리버스 프록시 설정 / nginx reverse proxy.
- `public/`, `index.html` — PWA 자산 + Vite 엔트리 HTML / PWA assets + Vite HTML.
- `.doc/` — PRD, 외부 자료 / PRD + reference docs.

## 주요 의존 관계 / Key Cross-Tree Dependencies

- `web-server/src/core.rs` → 직접 `#[path]`로 `src-tauri/src/core/*.rs` 를 인라인. 두 Rust 크레이트가 단일 도메인 코드를 공유합니다.
- `web-server/src/seed_pg.rs` → `include_str!("../../src-tauri/data/seed.json")` 동일 시드 데이터 단일 출처화.
- `src/` (TypeScript) → `@tauri-apps/api/core` 의 `invoke` 로 데스크톱 IPC, 또는 `fetch(${apiBaseUrl()}/api/...)` 로 웹 백엔드 호출. 분기는 `src/ipc/client.ts` 에서 정적 결정.

## 위험 신호 / Risk Signals

1. `core::tts::speak (windows)` 의 PowerShell 인라인 명령은 작은따옴표만 이스케이프 → 셸 명령 주입 여지 / inline PowerShell command escapes only the single-quote; broader injection vectors remain.
2. `web-server` 가 항상 SQLite 모듈을 같이 컴파일(원치 않아도 `rusqlite/bundled` 빌드) — 사용 안 할 때도 비용 발생.
3. 테스트 자산 부재 — 데스크톱 `core/db` 검색 랭킹, `.nsfpack` 라운드트립, 한국어 음성 템플릿 등 미커버.

## 빌드/실행 입구 / Build & Run Entry Points

| 명령 / Command | 효과 / Effect |
|---|---|
| `pnpm dev` | Vite 개발 서버 |
| `pnpm tauri:dev` | Tauri 데스크톱 dev |
| `pnpm tauri:build:mac:arm` 등 | OS별 데스크톱 번들 |
| `cargo run -p nsf-reading-helper-web` | Axum 서버 (`NSF_BIND=0.0.0.0:8080`) |
| `uvicorn ocr.main:app --host 0.0.0.0 --port 8000` | OCR 사이드카 |
| `docker compose up` | nginx + web + ocr 통합 스택 |
