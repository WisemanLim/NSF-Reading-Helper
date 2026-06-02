# src/ipc/tauriClient.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: Tauri 의 `invoke()` 로 데스크톱 Rust 명령(`#[tauri::command]`)을 호출하는 얇은 래퍼.
- 진입점에서의 거리: 1-2
- LOC: 73
- 주요 export: `tauriIpc`, `TtsOptions`, `SearchOptions`, `PackManifest`, `PackImportResult`
- 의존 모듈: `../types`
- 외부 패키지: `@tauri-apps/api`
- 부수효과: `invoke()` 매번 호출 시 Tauri 시스템과 통신
- 동시성: Promise 기반
- 테스트: 없음

---

```ts path=src/ipc/tauriClient.ts lines=1-11
import { invoke } from "@tauri-apps/api/core";
import type {
  AnalyzeRequest,
  AnalyzeResult,
  HistoryItem,
  Note,
  Settings,
  SymbolDetail,
  SymbolExample,
  SymbolHit,
} from "../types";
```

**분석 / Analysis**

- `@tauri-apps/api/core` 는 v2의 IPC 헬퍼 진입점.
- 모든 타입은 `../types` 에서 가져옴 → 백엔드 contract 와 매핑.

```ts path=src/ipc/tauriClient.ts lines=13-21
export interface TtsOptions {
  rate: number;
  lang: string;
}
export interface SearchOptions {
  domain?: string;
  symbolType?: string;
  limit?: number;
}
```

**분석 / Analysis**

- 로컬에서 정의. *왜 types.ts 가 아닌가*: 다른 모듈(`webClient`, `store`)이 직접 사용하지 않고 IPC 레이어 내부의 보조 타입이라 가까이 둠. 다만 `webClient` 도 `SearchOptions`/`TtsOptions` 를 사용하려고 `import type from "./tauriClient"` 를 함 → 결합도가 살짝 비대칭.

```ts path=src/ipc/tauriClient.ts lines=23-55
export const tauriIpc = {
  analyze: (input: AnalyzeRequest) =>
    invoke<AnalyzeResult>("analyze", { input }),

  symbolSearch: (q: string, opts?: SearchOptions) =>
    invoke<SymbolHit[]>("symbol_search", { q, opts: opts ?? null }),
  symbolGet: (id: number) => invoke<SymbolDetail>("symbol_get", { id }),
  symbolExamples: (id: number) =>
    invoke<SymbolExample[]>("symbol_examples", { id }),

  historyList: () => invoke<HistoryItem[]>("history_list"),
  favoritesAdd: (symbolId: number) =>
    invoke<void>("favorites_add", { symbolId }),
  favoritesRemove: (symbolId: number) =>
    invoke<void>("favorites_remove", { symbolId }),
  favoritesList: () => invoke<SymbolHit[]>("favorites_list"),
  notesUpsert: (n: Note) => invoke<Note>("notes_upsert", { note: n }),
  notesList: () => invoke<Note[]>("notes_list"),

  settingsGet: () => invoke<Settings>("settings_get"),
  settingsSet: (s: Settings) =>
    invoke<Settings>("settings_set", { settings: s }),

  ttsSpeak: (text: string, opts: TtsOptions) =>
    invoke<void>("tts_speak", { text, opts }),
  ttsStop: () => invoke<void>("tts_stop"),

  packImport: (path: string) =>
    invoke<PackImportResult>("pack_import", { path }),
  packExport: (path: string, name?: string) =>
    invoke<PackManifest>("pack_export", { path, name: name ?? null }),
  ftsRebuild: () => invoke<void>("fts_rebuild"),
};
```

**분석 / Analysis**

- **명령 이름 컨벤션**: Rust 측 `commands::analyze`, `commands::symbol_search` 등이 snake_case 이고 Tauri 의 invoke 가 이를 그대로 노출. JS-side 메서드만 camelCase.
- **인자 객체 키 명명**: invoke 의 2번째 인자 키(`symbolId`, `note`, `settings`)는 Tauri 의 자동 변환 규칙(camelCase argument → Rust snake_case parameter) 에 의해 Rust 측 `#[tauri::command] pub async fn favorites_add(state, symbol_id: i64)` 에 매핑. *흔한 실수*: Rust 인자명을 camelCase 로 쓰면 매핑 실패.
- **`opts: opts ?? null`** — Rust `Option<SearchOptions>` 가 `null` 을 `None` 으로 받음(serde 동작). undefined 도 무시되지만 명시적 null 이 안전.
- **`packExport` 의 `name: name ?? null`** — 동일 패턴.
- **타입 안정성**: invoke<T> 의 T 는 클라이언트가 단언; 실제 값은 직렬화된 JSON 이라 잘못된 T 라도 컴파일은 통과 → 런타임에 키 누락 시 undefined.
- **`historyList`, `favoritesList`, `notesList`, `settingsGet`, `ttsStop` 등은 두 번째 인자 없음** — 인자 없는 명령은 invoke 의 두 번째 인자 생략 가능.

```ts path=src/ipc/tauriClient.ts lines=57-74
export interface PackManifest {
  name: string;
  version: string;
  author?: string | null;
  license?: string | null;
  createdAt?: number | null;
  description?: string | null;
  symbolCount: number;
  domainCount: number;
}

export interface PackImportResult {
  manifest: PackManifest;
  insertedSymbols: number;
  skippedSymbols: number;
  insertedDomains: number;
}
```

**분석 / Analysis**

- 백엔드 `core/pack::PackManifest` 와 `PackImportResult` 의 미러. 카멜케이스 동기.
- *흔한 실수*: 이 두 타입이 `types.ts` 가 아닌 `tauriClient.ts` 에 있다는 점. 데스크톱 전용 기능이라 위치는 합리적이지만 webClient 가 throw 메서드에서 이 타입을 import 하지는 않음.

**짚어볼 점 / Watch-outs**
- 명령 이름이 string 리터럴이라 컴파일 타임 검증 없음. Rust 측 `tauri::generate_handler![...]` 와 manual 동기화.
- `invoke<void>` 의 실제 응답이 unit `()` 인 Rust 함수 — 정상.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 표면이 작고 일관, 인자 객체 컨벤션 명확.
- 위험: 명령 이름 typo 가 빌드 통과(런타임 발견).
- 개선 권고:
  1. `commands::*` 이름을 const 객체로 모아 typo 위험 감소.
  2. `IpcClient` 공통 인터페이스로 webClient 와 형상 동기화.
- 재읽기 권고 레벨: 1.
