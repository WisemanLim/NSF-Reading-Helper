# src/pages/SearchPage.tsx — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 기호 사전 검색 — 텍스트 입력(200ms 디바운스) + 손그림 패드.
- LOC: 103
- 주요 export: `SearchPage`
- 의존 모듈: `../ipc/client`, `../types`, `../components/SymbolPanel`, `../components/HandwritingPad`

---

```tsx path=src/pages/SearchPage.tsx lines=1-12
import { useEffect, useState } from "react";
import { ipc } from "../ipc/client";
import type { SymbolHit, SymbolDetail } from "../types";
import { SymbolPanel } from "../components/SymbolPanel";
import { HandwritingPad } from "../components/HandwritingPad";

export function SearchPage() {
  const [q, setQ] = useState("");
  const [hits, setHits] = useState<SymbolHit[]>([]);
  const [detail, setDetail] = useState<SymbolDetail | null>(null);
  const [loading, setLoading] = useState(false);
```

**분석 / Analysis**

- 로컬 상태만 사용(글로벌 store 미사용) — *왜*: 검색 결과는 페이지 떠나면 잊혀도 OK.

```tsx path=src/pages/SearchPage.tsx lines=13-34
  useEffect(() => {
    let cancelled = false;
    if (!q.trim()) {
      setHits([]);
      return;
    }
    setLoading(true);
    const t = setTimeout(async () => {
      try {
        const r = await ipc.symbolSearch(q, { limit: 30 });
        if (!cancelled) setHits(r);
      } catch (e) {
        console.warn("search failed", e);
      } finally {
        if (!cancelled) setLoading(false);
      }
    }, 200);
    return () => {
      cancelled = true;
      clearTimeout(t);
    };
  }, [q]);
```

**분석 / Analysis**

- 200ms 디바운스 + cancellation flag. **정석적 검색 패턴**.
- `let cancelled = false` + cleanup `cancelled = true` — effect cleanup 으로 in-flight 결과 폐기.
- `clearTimeout(t)` — 미실행된 timer 도 정리.
- *흔한 실수*: `setLoading(true)` 는 timer 시작 시점에 즉시 → 입력 변경 즉시 로딩 인디케이터. 200ms 동안만 표시되는 micro-spinner 가 깜빡일 수 있음. 의도라면 OK.
- 빈 입력 분기에서 `setLoading(false)` 누락 — `q` 가 비면 `loading` 이 이전 값 유지. 다음 입력에 의해 토글되긴 함.

```tsx path=src/pages/SearchPage.tsx lines=36-43
  const openDetail = async (id: number) => {
    try {
      const d = await ipc.symbolGet(id);
      setDetail(d);
    } catch (e) {
      console.warn(e);
    }
  };
```

**분석 / Analysis**

- 빠르게 두 번 클릭 시 race — 두 번째 클릭의 detail 이 먼저 도착하면 첫 번째가 덮어씀. `useAppStore.selectToken` 처럼 가드가 없음.

```tsx path=src/pages/SearchPage.tsx lines=45-103
  return (
    <div className="flex h-full flex-col overflow-hidden">
      <header className="flex h-14 shrink-0 items-center gap-3 border-b border-line px-6">
        <h1 className="text-sm font-semibold text-fg-muted">검색</h1>
        <input
          autoFocus
          className="input-base ml-auto max-w-md"
          placeholder="기호명·LaTeX·Unicode·별칭 (예: 적분, \int, ∫)"
          value={q}
          onChange={(e) => setQ(e.target.value)}
        />
      </header>
      <div className="grid min-h-0 flex-1 grid-cols-1 gap-4 overflow-hidden p-4 lg:grid-cols-[minmax(0,1fr)_380px]">
        <section className="min-h-0 overflow-y-auto">
          <div className="mb-3">
            <HandwritingPad onPickSymbol={(latex) => setQ(latex)} />
          </div>
          {loading && <div className="p-3 text-xs text-fg-muted">검색 중...</div>}
          {!loading && hits.length === 0 && q && (
            <div className="rounded-lg border border-dashed border-line bg-bg-elevated p-4 text-sm text-fg-muted">
              결과가 없습니다.
            </div>
          )}
          <ul className="flex flex-col gap-1">
            {hits.map((h) => (
              <li key={h.id}>
                <button onClick={() => openDetail(h.id)}
                  className="flex w-full items-center gap-3 rounded-lg border border-line bg-bg-elevated px-3 py-2 text-left text-sm hover:border-brand/60">
                  <span className="w-12 font-mono text-lg">{h.unicodeChar || h.latexCommand}</span>
                  <span className="min-w-0 flex-1">
                    <span className="font-medium">{h.canonicalNameKo}</span>
                    {h.readingKo && <span className="ml-2 text-xs text-brand">{h.readingKo}</span>}
                    <span className="block truncate text-xs text-fg-muted">{h.shortMeaningKo}</span>
                  </span>
                  <span className="rounded bg-bg px-1.5 py-0.5 text-[10px] uppercase text-fg-muted">{h.symbolType}</span>
                </button>
              </li>
            ))}
          </ul>
        </section>
        <aside className="min-h-0 overflow-y-auto">
          <SymbolPanel symbol={detail} />
        </aside>
      </div>
    </div>
  );
}
```

**분석 / Analysis**

- `<input autoFocus>` — 페이지 진입 시 자동 포커스. 라우트 전환마다 새 마운트라 정상 작동.
- `<HandwritingPad onPickSymbol={(latex) => setQ(latex)}>` — 손그림 결과를 검색 쿼리로. 디바운스 effect 가 자동으로 search 트리거.
- 리스트 키 `h.id` — backend i64 → number, 안전.
- 빈 결과 분기: `!loading && hits.length === 0 && q` — 모두 만족해야 표시.
- `<button>` 내부에 multiple `<span>` — 시맨틱 보존, 키보드 접근.

**짚어볼 점 / Watch-outs**
- detail race 가드 부재.
- 모바일 키보드 사용 시 200ms 디바운스가 빠른 타이핑 시 미세하게 끊겨 보일 수 있음(취향 차원).

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 디바운스 패턴 정석, 손그림 연동 자연스러움.
- 위험: detail race, 빈 입력 시 loading 토글 불완전.
- 개선 권고:
  1. `openDetail` 에 `requestId`/`cancelled` 가드 추가.
  2. 빈 입력 분기에서도 `setLoading(false)` 명시.
  3. 결과 0개일 때 마지막 쿼리를 표시(예: ‘"abc" 결과 없음’).
- 재읽기 권고 레벨: 2.
