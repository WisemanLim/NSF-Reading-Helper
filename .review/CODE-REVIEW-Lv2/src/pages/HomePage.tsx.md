# src/pages/HomePage.tsx — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 랜딩 화면 — 분석/검색으로 가는 2개 카드 링크.
- LOC: 51
- 주요 export: `HomePage`
- 의존 모듈: `../components/icons`, `react-router-dom`
- 외부 패키지: `react-router-dom`
- 부수효과: 없음

---

```tsx path=src/pages/HomePage.tsx lines=1-51
import { Link } from "react-router-dom";
import { IconFunction, IconSearch, IconSparkles } from "../components/icons";

export function HomePage() {
  return (
    <div className="flex h-full flex-col overflow-y-auto">
      <header className="flex h-14 shrink-0 items-center border-b border-line px-6">
        <h1 className="text-sm font-semibold text-fg-muted">홈</h1>
      </header>
      <div className="mx-auto flex w-full max-w-3xl flex-1 flex-col justify-center gap-8 px-6 py-10">
        <div className="space-y-2 text-center">
          <div className="mx-auto mb-3 flex h-12 w-12 items-center justify-center rounded-2xl bg-brand-soft text-brand">
            <IconSparkles />
          </div>
          <h2 className="text-2xl font-semibold tracking-tight">
            자연과학 수식, 읽고 이해해 볼까요?
          </h2>
          <p className="text-sm text-fg-muted">
            수식을 입력하면 한국어로 읽어주고, 기호 하나하나의 뜻을 풀어드려요.
          </p>
        </div>
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
          <Link to="/analyze" className="flex items-start gap-3 rounded-2xl border border-line bg-bg-elevated p-5 hover:border-brand/60">
            <IconFunction />
            <div>
              <div className="text-sm font-semibold">수식 분석</div>
              <div className="text-xs text-fg-muted">LaTeX · MathML · AsciiMath · Unicode 입력</div>
            </div>
          </Link>
          <Link to="/search" className="flex items-start gap-3 rounded-2xl border border-line bg-bg-elevated p-5 hover:border-brand/60">
            <IconSearch />
            <div>
              <div className="text-sm font-semibold">기호 사전 검색</div>
              <div className="text-xs text-fg-muted">한글·영어·LaTeX·Unicode 통합 검색</div>
            </div>
          </Link>
        </div>
      </div>
    </div>
  );
}
```

**분석 / Analysis**

- 정적 마크업. 상태/effects 없음 — 가장 단순한 페이지.
- `<Link>` v6 컴포넌트 — `<a>` 가 아니라 클라이언트 사이드 라우팅.
- Tailwind 디자인 토큰: `bg-bg`, `bg-bg-elevated`, `border-line`, `text-fg-muted`, `text-brand`, `bg-brand-soft` — `tailwind.config.js`/`index.css` 에 변수로 정의된 색.
- 접근성 / a11y:
  - `<header>` + `<h1>`, `<h2>` 위계 명확.
  - `<Link>` 의 텍스트 콘텐츠가 의미적이라 스크린리더 OK.
  - *흔한 실수*: 카드 자체에 `aria-label` 이 없어 IconSparkles 와 본문이 모두 읽혀 다소 장황. 개선 여지.

**짚어볼 점 / Watch-outs**
- 다국어 미지원 (UI 가 한국어 하드코딩). `Settings.uiLocale` 가 영문 모드라도 본 화면은 한국어 그대로.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 깔끔, 디자인 시스템 일관.
- 위험: i18n 부재.
- 개선 권고: `useAppStore(s => s.settings.uiLocale)` 으로 분기, 또는 `react-i18next` 도입.
- 재읽기 권고 레벨: 0.
