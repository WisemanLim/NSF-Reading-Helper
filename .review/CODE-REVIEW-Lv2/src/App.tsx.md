# src/App.tsx — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 부트스트랩 게이트 + 라우트 트리 정의. Zustand `useAppStore.bootstrap()` 결과에 따라 로딩/에러/실제 화면 중 하나 렌더.
- 진입점에서의 거리: **1** (main.tsx 가 import)
- LOC: 52
- 주요 export: default `App` 컴포넌트
- 의존 모듈 / Depends on: `./components/Layout`, `./components/ErrorBoundary`, `./pages/*`, `./store/useAppStore`
- 외부 패키지 / External: `react`, `react-router-dom`
- 부수효과: `useEffect` 안에서 `bootstrap()` 1회 호출
- 동시성: React 함수형 컴포넌트, 단일 effect
- 테스트 커버리지: 없음 / none

---

```tsx path=src/App.tsx lines=1-10
import { useEffect, useState } from "react";
import { Route, Routes } from "react-router-dom";
import { Layout } from "./components/Layout";
import { HomePage } from "./pages/HomePage";
import { AnalyzePage } from "./pages/AnalyzePage";
import { SearchPage } from "./pages/SearchPage";
import { LibraryPage } from "./pages/LibraryPage";
import { SettingsPage } from "./pages/SettingsPage";
import { useAppStore } from "./store/useAppStore";
import { ErrorBoundary } from "./components/ErrorBoundary";
```

**분석 / Analysis**

- 모든 페이지를 정적 import — code splitting 없음. 페이지 수가 5개로 적고 합산 LOC 가 작아 합리적 선택이지만, 향후 `lazy()` + `<Suspense>` 도입 여지.
- `ErrorBoundary` 는 라우트 트리만 감싸는 위치(아래) — 부트스트랩 단계 자체는 보호하지 않음.

```tsx path=src/App.tsx lines=12-19
export default function App() {
  const bootstrap = useAppStore((s) => s.bootstrap);
  const loaded = useAppStore((s) => s.loaded);
  const [bootError, setBootError] = useState<string | null>(null);

  useEffect(() => {
    bootstrap().catch((e) => setBootError(e instanceof Error ? e.message : String(e)));
  }, [bootstrap]);
```

**분석 / Analysis**

- Zustand 선택자 패턴 — 함수 단위 구독으로 리렌더 최소화. `bootstrap` 은 함수 참조라 정체성이 안정적이고 `useEffect` deps 에 안전합니다. *흔한 실수*: zustand에서 `useAppStore((s) => ({ a, b }))` 처럼 객체를 리턴하면 매 렌더 새 ref → 무한 리렌더. 여기서는 함수만 꺼내 안전.
- `useState<string | null>` — 부트 실패 메시지만 별도 관리. 글로벌 스토어에 두지 않은 이유는 한 번만 살아 있는 일회성 정보이기 때문.
- `useEffect(..., [bootstrap])` — StrictMode 에서 dev 시 2회 실행 가능 → `bootstrap` 이 idempotent 해야 함. `useAppStore.bootstrap` 은 결과를 항상 `loaded=true` 로 세팅하고 두 번째 호출에선 IPC 가 중복 발생하므로 미세한 중복 비용(허용 가능).

```tsx path=src/App.tsx lines=21-30
  if (bootError) {
    return (
      <div className="flex h-screen w-screen flex-col items-center justify-center gap-3 bg-bg p-8 text-center">
        <div className="text-base font-semibold text-danger">초기화에 실패했습니다</div>
        <pre className="max-w-2xl whitespace-pre-wrap rounded-lg bg-bg-elevated p-4 text-left font-mono text-xs text-fg-muted">
          {bootError}
        </pre>
      </div>
    );
  }
```

**분석 / Analysis**

- 에러 화면이 자기 자체 UI 만 그리고 라우트는 그리지 않음 — 사용자가 다른 페이지로 회피할 수단이 없음.
- 메시지를 `<pre>` 에 그대로 노출 — 디버깅엔 좋지만 운영 환경에서 IPC 내부 상태 누설 가능성(Tauri commands 에 비밀이 없으니 위험 낮음).
- *설계 결정*: 재시도 버튼 없음. `bootError` 발생 = 백엔드 자체 부재 가능성이 높아 단순 reload 가 안전한 대안.

```tsx path=src/App.tsx lines=31-37
  if (!loaded) {
    return (
      <div className="flex h-screen w-screen items-center justify-center bg-bg text-sm text-fg-muted">
        초기화 중...
      </div>
    );
  }
```

**분석 / Analysis**

- `bootstrap` 의 try 분기는 실패해도 finally 가 없이 `loaded:true` 를 세팅(스토어 코드 참조). 즉 사실상 영구 로딩 상태는 발생하지 않음.
- 짧은 깜빡임 방지용 스피너/지연 노출은 없음 — 부트가 빠르면 한 프레임 깜빡임.

```tsx path=src/App.tsx lines=39-52
  return (
    <ErrorBoundary>
      <Routes>
        <Route element={<Layout />}>
          <Route index element={<HomePage />} />
          <Route path="analyze" element={<AnalyzePage />} />
          <Route path="search" element={<SearchPage />} />
          <Route path="library" element={<LibraryPage />} />
          <Route path="settings" element={<SettingsPage />} />
        </Route>
      </Routes>
    </ErrorBoundary>
  );
}
```

**분석 / Analysis**

- `<Route element={<Layout/>}>` 는 react-router v6+ 의 *layout route* 패턴 — `Layout` 안 `<Outlet/>` 위치에 자식 라우트가 들어갑니다.
- `index` 는 부모 경로의 기본 자식 — 본 앱에선 `/` 가 `HomePage`.
- *짚어볼 점*: 404 처리(`path="*"`) 가 없음 — 잘못된 URL 입력 시 빈 `Layout` 만 그려짐(콘텐츠 영역이 비어 보임).
- *흔한 실수*: ErrorBoundary 가 라우터 안이라 React 렌더 중 발생한 예외는 잡지만, 라우터 navigation 에러는 별도 처리 필요. 현재 모든 navigate 는 hash-link 수준이라 위험 낮음.

**짚어볼 점 / Watch-outs**
- 404 라우트 부재.
- 부트 실패 시 사용자 회복 경로 부재(재시도 버튼/리다이렉트).
- StrictMode dev 환경에서 `bootstrap` 가 두 번 호출됨 — 백엔드 history insert 등 부수효과 시 dev 한정 중복 가능(현재 bootstrap 은 read-only).

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 부트 단계 명확히 분리, ErrorBoundary 위치 적절, Zustand 선택자 패턴 안전.
- 위험: 404 부재, 재시도 UX 부재.
- 개선 권고:
  1. `<Route path="*" element={<NotFound/>} />` 추가.
  2. 부트 실패 화면에 "다시 시도" 버튼(상태 reset + `bootstrap()` 재호출).
  3. 페이지 lazy import 도입(장기적 빌드 사이즈 관리).
- 재읽기 권고 레벨: 2 — 라우팅·부트 흐름 이해 필수.
