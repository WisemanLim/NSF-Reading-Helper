# src-tauri/ — 디렉토리 개요 / Directory Abstract

> [← INDEX.md](../INDEX.md)

## 파일 목록 / File List

| 파일 / File | 역할 / Role | 리뷰 / Review |
|---|---|---|
| Cargo.toml | Tauri 크레이트 매니페스트 | (구조 정보) |
| Cargo.lock | 잠금 | (제외) |
| build.rs | `tauri-build` 호출 | (제외) |
| tauri.conf.json | Tauri 셸 설정(번들, 권한, 윈도우) | (제외) |
| capabilities/default.json | Tauri 권한 정책 | (제외) |
| data/seed.json | 시드 사전 (컴파일타임 include) | (데이터) |
| icons/, gen/schemas/ | 아이콘, 자동 생성 스키마 | (제외) |
| src/ | Rust 소스 | [src/src_Abstract.md](src/src_Abstract.md) |

## 디렉토리 구조 / Structure Summary

`src-tauri/` 는 데스크톱 빌드의 루트입니다. 셸은 Tauri 2 이며, 비즈니스 로직은 `src/core/*` 에 모여 있습니다. 시드 사전 `data/seed.json` 은 `core::seed` 에서 `include_str!` 로 정적 임베드되어 첫 부팅에 단 한 번 SQLite로 적재됩니다. 이 디렉토리의 `core/*.rs` 파일들은 `web-server/src/core.rs` 가 `#[path]` 별칭으로 그대로 끌어다 씁니다 — 즉 데스크톱 코어는 사실상 **공유 라이브러리** 입니다.

## 주요 의존 관계 / Key Dependencies

- 의존받는 측 / Depended by: `web-server/src/core.rs` (path-alias 재사용).
- 외부 / External: Tauri 2 + plugins (`opener`/`dialog`/`fs`/`updater`), rusqlite (bundled), zip 0.6, regex, tracing.
- WebView 프런트는 `pnpm tauri:build` 결과의 `dist/` 를 자동 임베드.

## 위험 신호 / Risk Signals

1. `Cargo.toml` 에 선언된 `dirs`, `parking_lot`, `unicode-normalization` 가 직접 `use` 되지 않음 → 데드 의존.
2. `core::tts::speak (windows)` 가 PowerShell 인라인 명령에 사용자 텍스트를 끼워 넣음 — 따옴표 외 제어문자 미차단.
