# src/ipc/ — 디렉토리 개요 / Directory Abstract

> [← INDEX.md](../../INDEX.md)

## 파일 목록 / File List

| 파일 / File | LOC | 역할 / Role | 리뷰 링크 |
|---|---|---|---|
| client.ts | 6 | `isTauri` 기반 정적 디스패처 | [client.ts.md](client.ts.md) |
| tauriClient.ts | 73 | `@tauri-apps/api/core` `invoke` 래퍼 | [tauriClient.ts.md](tauriClient.ts.md) |
| webClient.ts | 91 | `fetch` 기반 HTTP 클라이언트 | [webClient.ts.md](webClient.ts.md) |

## 디렉토리 구조 / Structure Summary

`ipc/` 는 "백엔드 두 종류"를 단일 호출 표면으로 통일합니다. `client.ts` 는 빌드/런타임 `isTauri` 플래그로 분기해 한 번에 한 백엔드만 활성화하고, 양쪽 모듈은 동일한 메서드 시그니처(`analyze`, `symbolSearch`, …) 를 노출합니다. 두 모듈에서 차이가 있는 메서드는:
- 데스크톱 전용: `packImport`, `packExport`, `ftsRebuild`, `ttsSpeak`, `ttsStop`(웹은 no-op).
- 웹은 packImport/Export에서 의도적으로 throw.

## 주요 의존 관계 / Key Dependencies

- `client.ts` → `lib/env.isTauri`, `tauriClient`, `webClient`.
- `tauriClient.ts` → `@tauri-apps/api/core` (`invoke`).
- `webClient.ts` → `lib/env.apiBaseUrl()`, `fetch`.

## 위험 신호 / Risk Signals

1. `tauriClient.symbolSearch` 의 두 번째 인자가 `opts ?? null` 인데 Rust 측 `SearchOptions` 는 `Option<...>` 으로 받습니다. JSON `null` 도 정상이지만 Tauri serde 가 `serde_json::Value::Null` 을 `None` 으로 받는 동작에 의존. (현재 작동하지만, 신규 필드 추가 시 함정.)
2. `webClient.ttsSpeak/ttsStop` 는 빈 함수 — 호출자가 결과 검사 없이 `await` 해도 무방하나 타입은 `Promise<void>` 라서 분기 누락에 잘 노출.
3. `webClient.ftsRebuild` 가 `/admin/fts-rebuild` 를 POST 하지만 서버에 해당 라우트가 존재하지 않음(현재 `web-server/src/main.rs` 에 등록 없음). 의도적 옵션이라면 `.catch(() => {})` 가 무음 처리 — 데드코드에 가까움.
