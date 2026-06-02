# src/lib/env.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 런타임 환경 판정 (Tauri vs Web) + Vite env 에서 API base URL 읽기.
- LOC: 13
- 주요 export: `isTauri`, `apiBaseUrl()`
- 외부 패키지: 없음
- 부수효과: 모듈 평가 시 `window` 접근

---

```ts path=src/lib/env.ts lines=1-13
declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

export const isTauri =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export function apiBaseUrl(): string {
  const env = (import.meta as unknown as { env?: Record<string, string> }).env;
  return env?.VITE_NSF_API_BASE ?? "";
}
```

**분석 / Analysis**

- `declare global { interface Window { ... } }` — TypeScript 의 *ambient module augmentation*. `window` 객체에 비표준 프로퍼티를 안전하게 추가. Tauri 가 내부적으로 주입.
- `typeof window !== "undefined"` 가드 — SSR 안전, Web Worker 안전.
- `"__TAURI_INTERNALS__" in window` — Tauri 2 의 신호. v1 의 `__TAURI__` 와 다름. 버전 업그레이드 시 변경 위험.
- `apiBaseUrl()`:
  - `(import.meta as unknown as { env?: ... }).env` — `import.meta.env` 는 Vite 가 빌드 시 주입하는 객체. TypeScript 의 모듈 augmentation 으로도 처리 가능하지만 여기서는 캐스팅으로 우회.
  - 빈 문자열 폴백 → caller 가 `${base}/api/...` 처럼 사용해도 상대 경로(`/api/...`)가 되어 동일-출처 호출. *왜*: dev 시 같은 도메인에 nginx 가 프록시할 때 유용.

**짚어볼 점 / Watch-outs**
- Tauri v3+ 가 `__TAURI_INTERNALS__` 이름을 바꾸면 환경 감지 실패. 버전 체크 후 갱신 필요.
- `import.meta.env` 캐스팅 — `vite/client` 의 `<reference>` 를 `vite-env.d.ts` 가 이미 추가하므로 캐스팅은 사실 불필요. 코드 단순화 여지.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 짧고, SSR 안전.
- 위험: Tauri 내부 이름 의존.
- 개선 권고: `vite-env.d.ts` 를 활용해 캐스팅 제거 → `import.meta.env.VITE_NSF_API_BASE ?? ""` 한 줄.
- 재읽기 권고 레벨: 0 — 거의 자명.
