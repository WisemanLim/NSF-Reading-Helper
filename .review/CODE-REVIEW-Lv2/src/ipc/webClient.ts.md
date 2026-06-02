# src/ipc/webClient.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 웹 브라우저용 IPC 어댑터. `fetch` 로 `${apiBaseUrl()}/api/*` 호출.
- 진입점에서의 거리: 1-2
- LOC: 91
- 주요 export: `webIpc`
- 의존 모듈: `../types`, `../lib/env`, `./tauriClient` (타입만)
- 외부 패키지: 없음 (Web Fetch API)
- 부수효과: 네트워크 I/O
- 동시성: Promise
- 테스트: 없음

---

```ts path=src/ipc/webClient.ts lines=1-14
import type {
  AnalyzeRequest, AnalyzeResult, HistoryItem, Note, Settings,
  SymbolDetail, SymbolExample, SymbolHit,
} from "../types";
import { apiBaseUrl } from "../lib/env";
import type { SearchOptions, TtsOptions } from "./tauriClient";

const base = () => apiBaseUrl();
```

**분석 / Analysis**

- `base()` 는 함수 — 런타임 시점에 env 평가. 초기 import 시점에 잘못된 값이 캐시되는 문제 회피.
- `SearchOptions`/`TtsOptions` 는 `tauriClient` 의 정의 재사용 — 일방향 의존.

```ts path=src/ipc/webClient.ts lines=16-36
async function jget<T>(path: string): Promise<T> {
  const r = await fetch(`${base()}/api${path}`);
  if (!r.ok) throw new Error(await r.text().catch(() => r.statusText));
  return (await r.json()) as T;
}
async function jsend<T>(
  method: string, path: string, body?: unknown,
): Promise<T | void> {
  const r = await fetch(`${base()}/api${path}`, {
    method,
    headers: body !== undefined ? { "content-type": "application/json" } : {},
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });
  if (!r.ok) throw new Error(await r.text().catch(() => r.statusText));
  if (r.status === 204) return;
  const ct = r.headers.get("content-type") ?? "";
  if (!ct.includes("application/json")) return;
  return (await r.json()) as T;
}
```

**분석 / Analysis**

- `jget` 는 GET 전용; `jsend` 는 POST/PUT/DELETE 공통.
- *왜 두 함수*: GET 은 body 가 없으므로 별도 시그니처가 명료. 사실상 `jsend("GET", ...)` 로 통일 가능.
- `if (!r.ok) throw new Error(await r.text().catch(() => r.statusText))` — 서버가 에러 본문을 텍스트로 줄 수도, statusText 만 있을 수도 있음. 둘 다 다룸. *흔한 실수*: `r.text()` 가 throw 할 수 있으므로 `.catch` 가 필요.
- 204 No Content 분기 — Axum 의 favorites_add/remove 는 `StatusCode::NO_CONTENT` 를 돌려줌. 정확히 매칭.
- content-type 검증 — JSON 이 아닌 본문은 안전하게 무시. 단, 비-JSON 응답이 정상으로 간주되어 `T | void` 가 됨.
- *짚어볼 점*: 캐스팅 `as T` 는 런타임 검증 없음. zod/io-ts 도입 여지.

```ts path=src/ipc/webClient.ts lines=38-46
function qs(opts?: SearchOptions): string {
  if (!opts) return "";
  const p = new URLSearchParams();
  if (opts.domain) p.set("domain", opts.domain);
  if (opts.symbolType) p.set("symbolType", opts.symbolType);
  if (opts.limit) p.set("limit", String(opts.limit));
  const s = p.toString();
  return s ? `&${s}` : "";
}
```

**분석 / Analysis**

- `?` prefix 가 아닌 `&` prefix — 즉 caller 가 이미 `?` 를 포함했다고 가정. 아래 `symbolSearch` 가 `?q=...${qs(opts)}` 패턴이라 OK.
- `limit === 0` 인 경우 falsy 라 send 되지 않음 — *잠재 버그*: 사용자가 의도적으로 0 을 보내려 해도 동작 안 함. 현실에선 limit=0 의미 없음.

```ts path=src/ipc/webClient.ts lines=48-91
export const webIpc = {
  analyze: (input: AnalyzeRequest) =>
    jsend<AnalyzeResult>("POST", "/expressions/analyze", input) as Promise<AnalyzeResult>,

  symbolSearch: (q: string, opts?: SearchOptions) =>
    jget<SymbolHit[]>(`/symbols/search?q=${encodeURIComponent(q)}${qs(opts)}`),
  symbolGet: (id: number) => jget<SymbolDetail>(`/symbols/${id}`),
  symbolExamples: (id: number) => jget<SymbolExample[]>(`/symbols/${id}/examples`),

  historyList: () => jget<HistoryItem[]>("/history"),
  favoritesAdd: async (symbolId: number): Promise<void> => {
    await jsend("POST", "/favorites", { symbolId });
  },
  favoritesRemove: async (symbolId: number): Promise<void> => {
    await jsend("DELETE", `/favorites/${symbolId}`);
  },
  favoritesList: () => jget<SymbolHit[]>("/favorites"),
  notesUpsert: (n: Note) =>
    jsend<Note>("POST", "/notes", n) as Promise<Note>,
  notesList: () => jget<Note[]>("/notes"),

  settingsGet: () => jget<Settings>("/settings"),
  settingsSet: (s: Settings) =>
    jsend<Settings>("PUT", "/settings", s) as Promise<Settings>,

  ttsSpeak: async (_text: string, _opts: TtsOptions): Promise<void> => {
    /* browser uses speechSynthesis directly in lib/tts.ts */
  },
  ttsStop: async (): Promise<void> => {
    /* same as above */
  },

  packImport: async (_path: string): Promise<never> => {
    throw new Error(".nsfpack import is desktop-only");
  },
  packExport: async (_path: string, _name?: string): Promise<never> => {
    throw new Error(".nsfpack export is desktop-only");
  },
  ftsRebuild: async (): Promise<void> => {
    await jsend("POST", "/admin/fts-rebuild").catch(() => {
      /* optional endpoint */
    });
  },
};
```

**분석 / Analysis**

- `analyze`, `notesUpsert`, `settingsSet` 의 `as Promise<T>` — `jsend` 의 union(`T | void`) 좁히기. *왜 필요*: 204 가 안 와도 항상 JSON 이 옴을 caller 가 안다는 가정. 잘못된 백엔드 응답 시 caller 가 undefined 를 받음.
- `favoritesAdd`/`favoritesRemove` — 명시적 `async ... { await jsend(...); }` 로 void 보장. 좋은 패턴.
- *URL 인코딩*: `encodeURIComponent(q)` — q에 `&`, `=`, 한글 등 들어가도 안전.
- **`packImport/packExport`** — 의도적으로 throw. SettingsPage 가 `isTauri` 가드에서 호출하므로 실제 호출은 없음. 안전.
- **`ftsRebuild`** — `/admin/fts-rebuild` 라우트는 현재 web-server 에 없음 → `.catch(() => {})` 가 silently 흡수. **데드코드 또는 향후 endpoint 자리**. 명시 주석 `/* optional endpoint */` 있어 의도 명확.
- `ttsSpeak/ttsStop` 가 no-op — 브라우저는 `lib/tts.ts` 에서 `window.speechSynthesis` 를 직접 호출. *왜 분기 두 곳*: `lib/tts.ts` 가 `isTauri` 일 때만 `ipc.ttsSpeak` 호출 → 웹 경로에선 이 메서드가 호출되지 않음. no-op 는 호출 방어용.

**짚어볼 점 / Watch-outs**
- *Trust boundary*: `jget`/`jsend` 가 응답을 `as T` 로 단언. 백엔드 변경 시 런타임 오류는 사용자에게 모호한 메시지로 전달.
- `qs.limit === 0` 분기 버그 잠재.
- `ftsRebuild` 의 `/admin/fts-rebuild` 부재 — 라우트 등록을 잊었거나 의도 보류. 코드 검토 시 명확화 필요.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 작은 헬퍼 두 개로 일관, throw-on-not-ok 표준.
- 위험: 응답 검증 부재, deadpath 가능성.
- 개선 권고:
  1. zod/io-ts 도입해 응답 런타임 검증.
  2. `/admin/fts-rebuild` 라우트 등록 or webClient.ftsRebuild 제거.
  3. `limit ?? undefined` 사용해 0 처리 일관.
- 재읽기 권고 레벨: 2.
