# src/components/Layout.tsx — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 앱 셸 — 데스크톱 사이드바, 모바일 드로어, 글로벌 단축키, skip-to-content.
- LOC: 109
- 주요 export: `Layout`
- 의존 모듈: `./Sidebar`, `react-router-dom`
- 외부 패키지: `react`, `react-router-dom`
- 부수효과: `window keydown` listener
- 동시성: 단일

---

```tsx path=src/components/Layout.tsx lines=1-14
import { useEffect, useState } from "react";
import { Outlet, useLocation, useNavigate } from "react-router-dom";
import { Sidebar } from "./Sidebar";

export function Layout() {
  const navigate = useNavigate();
  const location = useLocation();
  const [drawerOpen, setDrawerOpen] = useState(false);

  // Close mobile drawer on route change.
  useEffect(() => {
    setDrawerOpen(false);
  }, [location.pathname]);
```

**분석 / Analysis**

- 라우트 바뀔 때마다 드로어 자동 닫힘. *왜*: 사용자가 메뉴 항목 선택 시 즉시 콘텐츠로 이동.

```tsx path=src/components/Layout.tsx lines=15-38
  useEffect(() => {
    function handler(e: KeyboardEvent) {
      const mod = e.metaKey || e.ctrlKey;
      if (mod && e.key.toLowerCase() === "k") {
        e.preventDefault();
        navigate("/search");
        setTimeout(() => {
          const input = document.querySelector<HTMLInputElement>(
            "input[type='text'], input[placeholder*='기호']",
          );
          input?.focus();
        }, 50);
      }
      if (mod && e.key === "/") {
        e.preventDefault();
        document.getElementById("main-content")?.focus();
      }
      if (e.key === "Escape") {
        setDrawerOpen(false);
      }
    }
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [navigate]);
```

**분석 / Analysis**

- 세 단축키:
  - **Cmd/Ctrl+K**: `/search` 로 이동 후 50ms 뒤 검색 input 포커스. *왜 setTimeout*: navigate 가 새 렌더 큐에 들어가 input 이 DOM 에 마운트되기까지 대기. **취약점**: 50ms 가짜고 DOM 마운트 보장 안 됨. 마운트 늦으면 미포커스. 더 견고한 방식: `SearchPage` 에 `autoFocus` (이미 있음) + route 가 동일하면 input 이 이미 있어 focus(). 두 경우를 한 곳에서 handle 하느라 selector 사용.
  - **DOM selector** `"input[type='text'], input[placeholder*='기호']"` — `type='text'` 가 fallback, placeholder 한글 문자열로 매칭. *문제점*: 한국어 placeholder 변경 시 매칭 깨짐. UI 변경에 결합.
  - **Cmd/Ctrl+/**: main content 영역 포커스. `<main id="main-content" tabIndex={-1}>` 와 매칭.
  - **Escape**: 드로어 닫기.
- `[navigate]` deps — `navigate` 는 react-router 가 stable ref 를 보장하므로 effect 가 재실행되지 않음.
- 글로벌 리스너 — 모든 페이지에 영향. 페이지 내부 input 에서 Ctrl+K 가 텍스트 입력으로 사용되면 가로채짐(현재 기능 없음).

```tsx path=src/components/Layout.tsx lines=40-69
  return (
    <div className="flex h-[100dvh] w-screen overflow-hidden bg-bg text-fg">
      <a href="#main-content"
        className="absolute left-2 top-2 z-50 -translate-y-12 rounded-md bg-brand px-3 py-1.5 text-sm font-medium text-bg transition-transform focus:translate-y-0 focus:outline-none focus:ring-2 focus:ring-brand">
        본문으로 건너뛰기
      </a>

      {/* Desktop sidebar */}
      <div className="hidden md:block">
        <Sidebar />
      </div>

      {/* Mobile drawer */}
      <div id="mobile-drawer"
        className={`fixed inset-0 z-40 md:hidden ${drawerOpen ? "" : "pointer-events-none"}`}
        aria-hidden={!drawerOpen}>
        <div
          className={`absolute inset-0 bg-black/60 transition-opacity ${drawerOpen ? "opacity-100" : "opacity-0"}`}
          onClick={() => setDrawerOpen(false)} />
        <div className={`absolute left-0 top-0 h-full w-64 transform transition-transform ${drawerOpen ? "translate-x-0" : "-translate-x-full"}`}>
          <Sidebar />
        </div>
      </div>
```

**분석 / Analysis**

- **Skip link** — a11y 모범. 키보드 포커스 시 화면에 나타남(`focus:translate-y-0`).
- `h-[100dvh]` — `dvh` 는 dynamic viewport height(모바일 주소창 변동 흡수). 최신 단위.
- 데스크톱/모바일 분기 / Tailwind: `hidden md:block` vs `md:hidden`.
- 드로어:
  - `aria-hidden={!drawerOpen}` — 비활성 상태에서 스크린리더 차단.
  - `pointer-events-none` — 닫힌 상태에서 클릭 무시(시각 보이지 않아도 인터랙티브 차단).
  - Backdrop 클릭 시 닫힘.
  - 두 자식: backdrop(어두운 배경) + drawer 본체(좌측 슬라이드).

```tsx path=src/components/Layout.tsx lines=71-109
      <main id="main-content" tabIndex={-1}
        className="flex min-w-0 flex-1 flex-col overflow-hidden focus:outline-none"
        aria-label="주 콘텐츠 영역">
        {/* Mobile top bar with hamburger */}
        <div className="flex items-center gap-2 border-b border-line bg-bg-elevated px-3 py-2 md:hidden">
          <button type="button"
            aria-label={drawerOpen ? "메뉴 닫기" : "메뉴 열기"}
            aria-expanded={drawerOpen}
            aria-controls="mobile-drawer"
            onClick={() => setDrawerOpen((v) => !v)}
            className="rounded-md border border-line p-1.5 focus:outline-none focus-visible:ring-2 focus-visible:ring-brand">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none"
              stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
              <line x1="3" y1="6" x2="21" y2="6" />
              <line x1="3" y1="12" x2="21" y2="12" />
              <line x1="3" y1="18" x2="21" y2="18" />
            </svg>
          </button>
          <span className="text-sm font-semibold">NSF Reading Helper</span>
        </div>
        <Outlet />
      </main>
    </div>
  );
}
```

**분석 / Analysis**

- `<main tabIndex={-1}>` — Ctrl+/ 단축키 포커스 받을 수 있도록 tabindex 설정.
- 모바일 햄버거 버튼 — `aria-expanded`, `aria-controls`, dynamic `aria-label` 모두 a11y 만점.
- `setDrawerOpen((v) => !v)` — functional updater. 빠른 연속 클릭에도 정확.
- `<Outlet />` — 자식 라우트 렌더 위치.

**짚어볼 점 / Watch-outs**
- 단축키 핸들러가 모든 입력에서 동작 — `<input>` 안에서 Ctrl+K 가 가로채짐. 일부 사용자는 OS 단축키 충돌 가능.
- DOM `querySelector` 가 한국어 placeholder 에 결합 — i18n 시 깨짐.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: a11y 우수(skip link, aria, role).
- 위험: placeholder 매칭에 의존, setTimeout 50ms 취약.
- 개선 권고:
  1. SearchPage 가 `autoFocus` 와 `useRef` 로 자체 포커스 보장 — Layout 의 selector 제거.
  2. Cmd+K 안에서 활성 input 검사(`document.activeElement instanceof HTMLInputElement`) 후 동작.
- 재읽기 권고 레벨: 2.
