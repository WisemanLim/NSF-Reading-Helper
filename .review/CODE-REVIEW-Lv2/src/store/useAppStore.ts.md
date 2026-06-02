# src/store/useAppStore.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: Zustand 글로벌 스토어. 부팅, 설정, 입력, 결과, 토큰/심볼 선택, 분석 진행 플래그를 관리.
- 진입점에서의 거리: 1
- LOC: 94
- 주요 export: `useAppStore` (hook)
- 의존 모듈 / Depends on: `../ipc/client`, `../types`
- 외부 패키지: `zustand`
- 부수효과: `ipc.*` 호출(IPC/HTTP)
- 동시성: 비동기 액션 + race-condition 보호 한 곳
- 테스트: 없음

---

```ts path=src/store/useAppStore.ts lines=1-9
import { create } from "zustand";
import { ipc } from "../ipc/client";
import type {
  AnalyzeResult,
  AnalyzeToken,
  HistoryItem,
  Settings,
  SymbolDetail,
} from "../types";
```

**분석 / Analysis**

- `zustand` 의 vanilla `create` 만 사용. middleware (persist/devtools/immer) 없음 — 상태가 작아 가벼움 우선.
- `ipc` 가 Tauri/Web 분기를 이미 한 추상이라 스토어는 백엔드 종류를 인지하지 않음. 좋은 계층화.

```ts path=src/store/useAppStore.ts lines=11-18
const DEFAULT_SETTINGS: Settings = {
  speechRule: "clearspeak",
  speechLocale: "ko",
  ttsRate: 1.0,
  ttsVoice: null,
  uiLocale: "ko",
  domainsEnabled: [],
};
```

**분석 / Analysis**

- 백엔드 부재 시 사용할 안전 기본값. *왜*: `bootstrap` 의 catch 분기에서 빈 settings 로도 라우트는 그려져야 하므로 client-side default 가 필수.
- *주의*: 백엔드 `Settings::default()` (Rust) 와 값이 일치해야 일관성 보장. 현재 값이 정확히 같음(`speech_rule: "clearspeak"`, `speech_locale: "ko"`, `tts_rate: 1.0`, …) — 좋은 동기화.

```ts path=src/store/useAppStore.ts lines=20-37
interface AppState {
  loaded: boolean;
  settings: Settings;
  history: HistoryItem[];
  currentInput: string;
  currentResult: AnalyzeResult | null;
  selectedToken: AnalyzeToken | null;
  selectedSymbol: SymbolDetail | null;
  analyzing: boolean;

  bootstrap: () => Promise<void>;
  saveSettings: (s: Partial<Settings>) => Promise<void>;
  setInput: (v: string) => void;
  setResult: (r: AnalyzeResult | null) => void;
  selectToken: (t: AnalyzeToken | null) => Promise<void>;
  setAnalyzing: (b: boolean) => void;
  reloadHistory: () => Promise<void>;
}
```

**분석 / Analysis**

- 상태와 액션을 하나의 인터페이스로 통합. Zustand 의 정형 패턴.
- `saveSettings` 가 `Partial<Settings>` 를 받아 부분 패치 — UI 가 한 필드씩 바꿔도 됨.

```ts path=src/store/useAppStore.ts lines=39-48
export const useAppStore = create<AppState>()((set, get) => ({
  loaded: false,
  settings: DEFAULT_SETTINGS,
  history: [],
  currentInput: "",
  currentResult: null,
  selectedToken: null,
  selectedSymbol: null,
  analyzing: false,
```

**분석 / Analysis**

- `create<AppState>()(...)` — 두 번째 호출은 zustand v4+ 의 typed-curry 패턴. middleware 없을 땐 의미 없는 듯하지만 타입 추론 유지를 위해 표준.
- 초기 상태: 모두 안전한 기본값. `loaded:false` 로 시작해 App 이 게이트.

```ts path=src/store/useAppStore.ts lines=49-58
  async bootstrap() {
    try {
      const settings = await ipc.settingsGet();
      const history = await ipc.historyList().catch(() => []);
      set({ settings: { ...DEFAULT_SETTINGS, ...settings }, history, loaded: true });
    } catch (e) {
      console.warn("bootstrap fell back to defaults", e);
      set({ loaded: true });
    }
  },
```

**분석 / Analysis**

- 순서 / *왜*: `settingsGet` 은 필수(설정 없으면 UI 일부 비정상), `historyList` 는 선택적(빈 배열로 충분).
- `historyList().catch(() => [])` — 첫 부팅(빈 DB) 시 빈 배열이 유효, 명시적 폴백.
- 외부 catch 가 `settingsGet` 실패까지 흡수. 그 경우 `settings` 는 그대로 `DEFAULT_SETTINGS`.
- `{ ...DEFAULT_SETTINGS, ...settings }` — 백엔드가 새 필드를 누락해도 클라이언트 기본값으로 채움. 좋은 forward-compat.
- *흔한 실수*: 외부 try 가 throw 하면 위 catch 의 `set({ loaded: true })` 가 호출되어 사용자는 빈 사전을 봄. `bootError` 는 `App.tsx` 의 자체 promise rejection 가 잡기 때문에 — 잠깐, 본 함수는 throw 하지 않음 → `App.tsx` 의 `.catch` 가 작동하지 않음. **즉 부팅 실패 시 `bootError` 화면이 표시되지 않을 수도 있음**. 정확히는 `ipc.settingsGet` 이 외부 try 가 잡아 자체 처리 → `App.tsx` 의 catch 는 트리거되지 않음. 의도된 동작이라면 OK, `App.tsx` 의 에러 분기는 사실상 dead path 가능성.

```ts path=src/store/useAppStore.ts lines=60-64
  async saveSettings(patch) {
    const next = { ...get().settings, ...patch };
    const saved = await ipc.settingsSet(next);
    set({ settings: saved ?? next });
  },
```

**분석 / Analysis**

- `get().settings` 로 최신 값 스냅샷. 두 번의 빠른 호출이 race 할 가능성이 있지만 동일 키 토글이면 마지막 쓰기 승리.
- `saved ?? next` — 백엔드가 반환을 주면 그것을 사용, null/undefined면 클라이언트 계산값. *왜 둘 다 처리*: Tauri command 는 반드시 `Settings` 를 돌려주지만 webClient 는 미응답 가능성.

```ts path=src/store/useAppStore.ts lines=66-71
  setInput(v) {
    set({ currentInput: v });
  },
  setResult(r) {
    set({ currentResult: r, selectedToken: null, selectedSymbol: null });
  },
```

**분석 / Analysis**

- `setResult` 가 결과를 갱신할 때 selection 도 초기화 — 이전 결과의 선택이 잘못 표시되지 않도록 보호.

```ts path=src/store/useAppStore.ts lines=73-84
  async selectToken(t) {
    set({ selectedToken: t, selectedSymbol: null });
    if (!t?.symbolId) return;
    try {
      const detail = await ipc.symbolGet(t.symbolId);
      if (get().selectedToken?.symbolId === t.symbolId) {
        set({ selectedSymbol: detail });
      }
    } catch (e) {
      console.warn("symbol lookup failed", e);
    }
  },
```

**분석 / Analysis**

- *핵심 패턴*: `if (get().selectedToken?.symbolId === t.symbolId) { ... }` — race-condition 가드. 사용자가 토큰 A 클릭 → IPC 보내고, 응답 전 토큰 B 클릭 → 토큰 A 응답이 늦게 와도 detail 을 덮어쓰지 않음.
- *흔한 실수*: 비교를 `selectedToken === t` (참조) 가 아니라 `symbolId` (값) 으로 한 이유 — 동일 id 라면 새 클릭이라도 결과를 받아도 OK. 합리적.
- `set({ selectedToken: t, selectedSymbol: null })` 을 먼저 호출 → 사용자에게 즉시 반응(스피너 등은 SymbolPanel 에서 selectedSymbol 가 null 일 때 placeholder 표시).

```ts path=src/store/useAppStore.ts lines=86-94
  setAnalyzing(b) {
    set({ analyzing: b });
  },

  async reloadHistory() {
    const history = await ipc.historyList().catch(() => []);
    set({ history });
  },
}));
```

**분석 / Analysis**

- 분석 완료 후 `AnalyzePage` 가 `reloadHistory()` 호출 → 좌측 라이브러리에 즉시 반영.
- 동일하게 빈 배열 폴백.

**짚어볼 점 / Watch-outs**
- `bootstrap` 의 외부 try-catch 가 throw 흐름을 흡수하여 `App.tsx` 의 `bootError` 화면이 표시될 일이 거의 없음. 의도라면 `App.tsx`의 분기를 단순화하거나, 실패를 store state(`bootError`) 로 두는 게 일관적.
- `analyzing` 상태가 동시 분석 요청을 막지는 않음 — 두 번 빠른 클릭 시 두 결과 중 마지막만 사용(setResult 후 덮어씀). race 가드 없음.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 가벼운 zustand 사용, race 가드 1개, 백엔드 부재 시 빈-안전.
- 위험: bootstrap 실패가 사용자에 잘 보이지 않음, 분석 동시성 보호 부재.
- 개선 권고:
  1. `bootError` 를 스토어로 옮기고 `App.tsx` 가 그것을 읽도록 통일.
  2. `analyzing` 가 true 일 때 추가 호출을 차단하거나 latest-only 패턴 추가.
  3. `selectedSymbolLoading` 같은 보조 플래그로 detail 로딩 시각 피드백.
- 재읽기 권고 레벨: 2 — 스토어 동작이 거의 모든 페이지에 영향.
