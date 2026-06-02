# src/ — 디렉토리 개요 / Directory Abstract

> [← INDEX.md](../INDEX.md)

## 파일 목록 / File List

| 파일 / File | LOC | 역할 / Role | 리뷰 링크 / Review |
|---|---|---|---|
| main.tsx | 29 | Vite/React 부팅 + PWA SW 등록 | [main.tsx.md](main.tsx.md) |
| App.tsx | 52 | 라우트 트리 + bootstrap 게이트 | [App.tsx.md](App.tsx.md) |
| types.ts | 129 | 공유 DTO (백엔드 모델 미러) | [types.ts.md](types.ts.md) |
| index.css | — | Tailwind base + 변수 | (미선택 / deferred) |
| vite-env.d.ts | 3 | 타입 선언만 | (미선택) |

서브디렉토리 / Subdirectories: [`components/`](components/components_Abstract.md), [`ipc/`](ipc/ipc_Abstract.md), [`lib/`](lib/lib_Abstract.md), [`pages/`](pages/pages_Abstract.md), [`store/`](store/store_Abstract.md).

## 디렉토리 구조 / Structure Summary

`src/` 는 React 19 SPA의 루트로, `main.tsx` 가 Brower-Router 컨텍스트 안에서 `App` 을 마운트합니다. `App.tsx` 는 Zustand `useAppStore` 의 `bootstrap()` 으로 백엔드 핸드셰이크를 끝낸 뒤에야 라우트를 그립니다. 라우트 셀은 `components/Layout` 이 사이드바·모바일 드로어·키보드 단축키를 책임지고, `pages/*` 가 실제 화면을 그립니다. `types.ts` 는 백엔드 `core/models.rs` 의 카멜케이스 직렬화와 1:1 대응됩니다.

## 주요 의존 관계 / Key Dependencies

- 들어오는 호출 / Inbound: `index.html` 이 `src/main.tsx` 를 모듈 스크립트로 로드.
- 나가는 호출 / Outbound: `store/useAppStore` → `ipc/client` → Tauri or Web; `pages/*` 도 같은 경로 사용.
- 데이터 모델은 `types.ts` 가 단일 진실. 변경 시 backend `core/models.rs` 와 동기 필요.

## 위험 신호 / Risk Signals

1. `App.tsx` 의 bootstrap 실패 분기가 화면 전체 차단(라우트 무효)이지만 재시도 버튼이 없음.
2. `types.ts` 와 백엔드 enum 사이의 정합은 컴파일 타임 검증이 아니라 수작업.
