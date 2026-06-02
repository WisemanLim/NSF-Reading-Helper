# src/ipc/client.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: `isTauri` 플래그 기반으로 Tauri 또는 Web 클라이언트를 정적 선택. 단일 진입 표면 제공.
- 진입점에서의 거리: 2-3 (store/pages 가 다수 import)
- LOC: 6
- 주요 export: `ipc`, `SearchOptions` (re-export), `TtsOptions` (re-export)
- 의존 모듈: `../lib/env`, `./tauriClient`, `./webClient`
- 외부 패키지: 없음
- 부수효과: 없음 (정적 선택)
- 동시성: N/A
- 테스트: 없음

---

```ts path=src/ipc/client.ts lines=1-6
import { isTauri } from "../lib/env";
import { tauriIpc } from "./tauriClient";
import { webIpc } from "./webClient";

export const ipc = isTauri ? tauriIpc : webIpc;
export type { SearchOptions, TtsOptions } from "./tauriClient";
```

**분석 / Analysis**

- 모듈 평가 시 `isTauri` 가 `window` 객체를 읽음 → SSR 환경에선 평가 실패 가능. 본 프로젝트는 SSR 미사용이라 안전.
- *설계 결정*: 동적 import 가 아닌 정적 선택. 양 모듈을 항상 번들에 포함 — Vite 가 tree-shake 으로 Tauri-only 빌드에서 webClient 의 미사용 코드를 일부 제거할 수 있지만 plugin 패키지의 부수효과는 남음. 만일 번들 크기가 큰 이슈가 되면 `if (isTauri) import('./tauriClient') ...` 방식이 더 작아짐.
- `export type` re-export — 타입만 빼서 다른 모듈에서 `import type { SearchOptions } from "../ipc/client"` 가능. 좋은 facade.
- *흔한 실수*: 양 클라이언트의 메서드 시그니처가 다르면 호출자가 빌드 타임에 알 수 없는 차이를 만남. 현재는 두 모듈이 동일한 키 집합과 시그니처를 가짐 — 단, `ttsSpeak/ttsStop`, `packImport/packExport`, `ftsRebuild` 의 동작이 다름(web 은 throw or no-op).

**짚어볼 점 / Watch-outs**
- 두 모듈의 키 집합 동기화는 manual. 한쪽에 메서드를 추가하고 다른 쪽을 잊으면 TypeScript 가 `ipc.xxx` 호출 위치에서 union narrow 실패로 빌드 에러를 일으킬 수는 있으나(공통 형태가 깨지면), 동일 시그니처라면 silently 한 쪽만 호출됨.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 짧고, 의도 명확.
- 위험: 두 모듈 시그니처 정합은 사람 책임.
- 개선 권고:
  1. 명시적 `interface IpcClient { ... }` 도입해 `tauriIpc/webIpc` 가 그 인터페이스를 만족하도록 컴파일 타임 강제.
  2. (선택) 환경 분기를 dynamic import 로 전환해 번들 크기 절감.
- 재읽기 권고 레벨: 1 — 6줄, 한눈에 이해 가능.
