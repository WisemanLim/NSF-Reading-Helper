# src/main.tsx — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: React 19 SPA 부팅 (StrictMode + BrowserRouter + App). 웹 환경에서만 PWA 서비스 워커 등록.
- 진입점에서의 거리 / Distance from entrypoint: **0** (Vite/Tauri 엔트리)
- 라인 수 / LOC: 29
- 주요 export / Exports: 없음 (top-level side-effect)
- 의존 모듈 / Depends on (internal): `./App`, `./lib/env`, `./index.css`
- 사용된 외부 패키지 / External: `react`, `react-dom/client`, `react-router-dom`
- 부수효과 / Side effects: `document.getElementById('root')` DOM 마운트, SW 등록(브라우저+PROD only)
- 동시성 모델 / Concurrency: 단일 스레드 + `load` 이벤트 핸들러
- 테스트 커버리지 / Test coverage signal: 없음 / none

---

```tsx path=src/main.tsx lines=1-7
import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import App from "./App";
import "./index.css";
import { isTauri } from "./lib/env";
```

**분석 / Analysis (Level 2)**

- `React` 자체는 JSX 트랜스폼이 자동(`react/jsx-runtime`)이라 명시적 import 가 필수는 아니지만, `<React.StrictMode>` 를 명시 사용하므로 필요합니다.
- `ReactDOM` 은 React 18+ 의 `createRoot` API 진입점.
- `BrowserRouter` 는 HTML5 History API 기반 라우터. Tauri WebView 도 일반 브라우저와 같은 History API 를 제공하므로 동일 코드 동작.
- `App` 이 라우트 트리.
- `./index.css` 는 Tailwind layer + 커스텀 변수.
- `isTauri` 는 `window.__TAURI_INTERNALS__` 존재로 판정. 아래 SW 분기에 사용.

```tsx path=src/main.tsx lines=8-14
ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <BrowserRouter>
      <App />
    </BrowserRouter>
  </React.StrictMode>,
);
```

**분석 / Analysis**

- `document.getElementById("root")!` — non-null assertion. `index.html` 의 `<div id="root">` 이 존재한다는 정적 보장에 의존합니다. *흔한 실수*: HTML 셸을 변경했을 때 id 가 바뀌면 런타임 NPE.
- `createRoot(...)` 는 동시성 모드 지원 루트.
- `React.StrictMode` 는 dev 빌드에서 사이드이펙트(`useEffect`) 를 2회 실행해 누수 감지. 프로덕션에선 단발. *주의*: 개발 모드에서 SW 등록은 `import.meta.env.PROD` 가드를 통과하지 않아 비활성 — 분기 정확.
- `BrowserRouter` 가 `App` 을 감싸고 있어 모든 자식이 `useNavigate`/`useLocation` 사용 가능.
- *설계 결정*: HashRouter 대신 BrowserRouter 사용 → Tauri 의 단일 페이지 셸과 nginx 의 `try_files` (SPA fallback) 양쪽에서 모두 동작해야 함. `deploy/nginx.app.conf` 의 fallback 설정에 의존.

```tsx path=src/main.tsx lines=16-29
// Service worker registration — web only (Tauri has its own asset protocol),
// production only (dev SW caching obscures live reloads).
if (
  !isTauri &&
  typeof window !== "undefined" &&
  "serviceWorker" in navigator &&
  import.meta.env.PROD
) {
  window.addEventListener("load", () => {
    navigator.serviceWorker
      .register("/sw.js", { scope: "/" })
      .catch((e) => console.warn("sw registration failed", e));
  });
}
```

**분석 / Analysis**

- 4개의 조건 AND — *왜*:
  1. `!isTauri`: Tauri 내부에선 자체 asset 프로토콜(`tauri://`)을 사용해 SW가 충돌/오작동 가능.
  2. `typeof window !== "undefined"`: SSR-안전 가드. 본 프로젝트는 SSR 미사용이지만 방어적 코드.
  3. `"serviceWorker" in navigator`: 구형 브라우저/Webview 호환.
  4. `import.meta.env.PROD`: 개발 시 SW 캐시가 HMR 충돌 — 핵심 트레이드오프 명시.
- `window.addEventListener("load", ...)`: `load` 이벤트는 모든 sub-resource 로드 후 발생. 등록 자체가 백그라운드 작업이므로 메인 렌더 차단 없음 — 좋은 패턴.
- `.register("/sw.js", { scope: "/" })`: 절대 경로. nginx 가 `/sw.js` 를 정적 자산으로 서빙해야 함(`public/sw.js`).
- `.catch(console.warn)`: 실패는 비치명적 — 오프라인 캐시가 빠질 뿐 앱은 동작.

**짚어볼 점 / Watch-outs**
- SW 업데이트 정책이 없음. `sw.js` 변경 시 사용자가 새로고침해도 새 SW 가 활성화되지 않음(controlled by user agent). 향후 `skipWaiting` + `clients.claim` 또는 강제 reload 전략 필요.
- `!` non-null assertion 은 HTML 셸을 신뢰 — `App` 마운트 실패 시 흰 화면.

---

**파일 수준 평가 / File-Level Verdict**

- 강점 / Strengths: 분기 조건이 명료, dev/prod 분리 합리적, 모든 외부 부수효과(SW 등록)가 try-catch 포함.
- 위험 / Risks: SW 갱신 전략 부재, `getElementById!` 의 정적 보장 의존, dev 빌드에서 SW 가드를 우회하면 캐시 디버깅 어려움.
- 개선 권고 / Recommendations:
  1. SW 등록 후 `registration.waiting` 처리 정책 추가(skipWaiting / 사용자 토스트).
  2. `import.meta.env.PROD` 대신 환경별 환경변수로 SW 토글 가능하게.
- 재읽기 권고 레벨 / Suggested re-read level: 1 — 초보자도 빠르게 이해 가능, 핵심 분기만 새겨두면 충분.
