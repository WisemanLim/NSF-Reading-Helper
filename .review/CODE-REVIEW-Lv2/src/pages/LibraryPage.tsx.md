# src/pages/LibraryPage.tsx — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 최근 분석 + 즐겨찾기 표시. 마운트 시 한 번 fetch.
- LOC: 74
- 주요 export: `LibraryPage`
- 의존 모듈: `../ipc/client`, `../types`

---

```tsx path=src/pages/LibraryPage.tsx lines=1-12
import { useEffect, useState } from "react";
import { ipc } from "../ipc/client";
import type { HistoryItem, SymbolHit } from "../types";

export function LibraryPage() {
  const [history, setHistory] = useState<HistoryItem[]>([]);
  const [favorites, setFavorites] = useState<SymbolHit[]>([]);

  useEffect(() => {
    ipc.historyList().then(setHistory).catch(console.warn);
    ipc.favoritesList().then(setFavorites).catch(console.warn);
  }, []);
```

**분석 / Analysis**

- 두 fetch 가 병렬. 둘 다 실패해도 빈 배열 유지(초기값).
- `useEffect(..., [])` — 마운트 한 번. 페이지 떠났다가 돌아오면 다시 fetch.
- *흔한 실수*: store 의 `history` 와 별도로 두 번 fetch — 동일 데이터 두 곳 보관. `useAppStore(s => s.history)` 를 직접 사용했더라면 데이터 일관성↑. 의도적으로 별도라면 favorites 가 store 에 없는 이유와 정합.
- 사용자가 `AnalyzePage` 에서 분석 후 `Library` 로 이동하면 새 fetch — 최근 데이터 보임. 그러나 즐겨찾기 추가 후 다시 Library 안 갔다 오면 갱신 안 됨.

```tsx path=src/pages/LibraryPage.tsx lines=14-74
  return (
    <div className="flex h-full flex-col overflow-hidden">
      <header className="flex h-14 shrink-0 items-center border-b border-line px-6">
        <h1 className="text-sm font-semibold text-fg-muted">라이브러리</h1>
      </header>
      <div className="grid min-h-0 flex-1 grid-cols-1 gap-4 overflow-y-auto p-4 lg:grid-cols-2">
        <section>
          <h2 className="mb-2 text-sm font-medium">최근 분석</h2>
          {history.length === 0 ? (
            <div className="rounded-lg border border-dashed border-line bg-bg-elevated p-4 text-xs text-fg-muted">
              아직 분석한 수식이 없습니다.
            </div>
          ) : (
            <ul className="flex flex-col gap-1">
              {history.map((h) => (
                <li key={h.expressionId} className="rounded-lg border border-line bg-bg-elevated p-3">
                  <code className="block break-all font-mono text-xs text-brand">{h.rawInput}</code>
                  {h.speechKo && <div className="mt-1 text-xs text-fg-muted">{h.speechKo}</div>}
                </li>
              ))}
            </ul>
          )}
        </section>
        <section>
          <h2 className="mb-2 text-sm font-medium">즐겨찾기</h2>
          {favorites.length === 0 ? (
            <div className="rounded-lg border border-dashed border-line bg-bg-elevated p-4 text-xs text-fg-muted">
              아직 즐겨찾기한 기호가 없습니다.
            </div>
          ) : (
            <ul className="flex flex-col gap-1">
              {favorites.map((s) => (
                <li key={s.id} className="flex items-center gap-3 rounded-lg border border-line bg-bg-elevated p-3">
                  <span className="w-10 font-mono text-lg">{s.unicodeChar || s.latexCommand}</span>
                  <span className="min-w-0 flex-1">
                    <span className="font-medium">{s.canonicalNameKo}</span>
                    <span className="block truncate text-xs text-fg-muted">{s.shortMeaningKo}</span>
                  </span>
                </li>
              ))}
            </ul>
          )}
        </section>
      </div>
    </div>
  );
}
```

**분석 / Analysis**

- `lg:grid-cols-2` — 두 섹션 좌우 배치(데스크탑).
- `<code className="block break-all">` — 긴 LaTeX 자동 줄바꿈.
- 즐겨찾기 항목 클릭 시 navigate 처리 없음 — 단순 표시만 가능. 향후 SymbolPanel 열기/AnalyzePage 채우기 등 인터랙션 추가 여지.
- 즐겨찾기 제거 버튼 없음.

**짚어볼 점 / Watch-outs**
- 데이터 동기화: store 의 history 와 로컬 history 가 분리되어 두 곳 갱신 책임 분산.
- 즐겨찾기 제거 인터랙션 부재.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 단순, 빈 상태 UI 좋음.
- 위험: 인터랙션 누락(클릭/제거), store 와 미동기.
- 개선 권고:
  1. `useAppStore(s => s.history)` 로 통일.
  2. 즐겨찾기 제거 버튼 + `ipc.favoritesRemove` 호출.
  3. 항목 클릭 시 라우트 이동(`/analyze?input=...` 또는 `/search?q=...`).
- 재읽기 권고 레벨: 1.
