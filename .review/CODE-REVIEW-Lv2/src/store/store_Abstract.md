# src/store/ — 디렉토리 개요 / Directory Abstract

> [← INDEX.md](../../INDEX.md)

## 파일 목록 / File List

| 파일 / File | LOC | 역할 / Role | 리뷰 링크 |
|---|---|---|---|
| useAppStore.ts | 94 | Zustand 글로벌 스토어 | [useAppStore.ts.md](useAppStore.ts.md) |

## 디렉토리 구조 / Structure Summary

단일 파일. Zustand `create<AppState>()(...)` 로 정의되며, `loaded` 가 `true` 가 되기 전엔 `App.tsx` 가 라우트를 그리지 않는 부팅 게이트 역할도 합니다. 상태 카테고리:
- 세션 / Session: `loaded`, `analyzing`.
- 설정 / Settings: `settings` (백엔드 영속).
- 사용자 흐름 / User flow: `currentInput`, `currentResult`, `selectedToken`, `selectedSymbol`.
- 메타데이터 / Metadata: `history`.

액션은 모두 `ipc` 를 통해 백엔드와 동기화하며, `selectToken` 에는 stale-response 보호(`if (get().selectedToken?.symbolId === ...)`).

## 주요 의존 관계 / Key Dependencies

- `zustand` (vanilla create)
- `ipc/client` 의 모든 read/write 메서드.
- 소비자 / Consumers: `App.tsx`, 거의 모든 `pages/*`, `components/SymbolPanel.tsx`.

## 위험 신호 / Risk Signals

1. `bootstrap` 실패 시 `setLoaded(true)` 를 호출해 라우트를 그대로 띄움 — 사용자는 데이터가 비어 있음을 알아채야 함.
2. `saveSettings` 가 부분 patch 후 `saved ?? next` 로 두 값 중 하나 선택; 서버가 `null` 반환할 가능성은 webClient 쪽 throw 의 영향으로 거의 없지만 명세 부족.
