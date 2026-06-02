# src/components/SymbolPanel.tsx — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 단일 기호의 상세 정보 표시 (이름, 별칭, 도메인별 의미, 예시, 유사 기호). 즐겨찾기 추가 버튼 포함.
- LOC: 139
- 주요 export: `SymbolPanel`
- 의존 모듈: `../ipc/client`, `../store/useAppStore`, `../types`, `./icons`

---

```tsx path=src/components/SymbolPanel.tsx lines=1-9
import { ipc } from "../ipc/client";
import { useAppStore } from "../store/useAppStore";
import type { SymbolDetail, SymbolHit } from "../types";
import { IconStar } from "./icons";

interface Props {
  symbol: SymbolDetail | null;
}
```

**분석 / Analysis**

- prop 단일 — 부모(`AnalyzePage`/`SearchPage`)가 detail 객체 또는 null 전달.

```tsx path=src/components/SymbolPanel.tsx lines=10-22
export function SymbolPanel({ symbol }: Props) {
  if (!symbol) {
    return (
      <div role="status" aria-live="polite"
        className="rounded-xl border border-dashed border-line bg-bg-elevated p-4 text-sm text-fg-muted">
        토큰을 클릭하면 기호 설명이 표시됩니다.
      </div>
    );
  }
```

**분석 / Analysis**

- null 상태에 placeholder. `aria-live="polite"` — 스크린리더가 detail 도착 시 자동 announce.

```tsx path=src/components/SymbolPanel.tsx lines=23-54
  return (
    <div role="region" aria-label="기호 설명" aria-live="polite"
      className="flex flex-col gap-3 rounded-xl border border-line bg-bg-elevated p-4">
      <div className="flex items-start justify-between gap-2">
        <div>
          <div className="font-mono text-2xl text-fg">
            {symbol.unicodeChar || symbol.latexCommand}
          </div>
          <div className="text-sm font-medium text-fg">
            {symbol.canonicalNameKo}
            {symbol.canonicalNameEn && (
              <span className="ml-2 text-fg-muted">· {symbol.canonicalNameEn}</span>
            )}
          </div>
          {symbol.readingKo && (
            <div className="text-xs text-brand">읽기: {symbol.readingKo}</div>
          )}
        </div>
        <button className="btn-outline !py-1" title="즐겨찾기 추가"
          onClick={() => ipc.favoritesAdd(symbol.id).catch(console.warn)}>
          <IconStar />
        </button>
      </div>
```

**분석 / Analysis**

- 헤더: 큰 기호(`unicodeChar` 우선, 없으면 `latexCommand`), 이름(KO 필수, EN 옵셔널), 읽기.
- 즐겨찾기 버튼 — `fire-and-forget`. *짚어볼 점*: 토글이 아니라 add 전용 → 중복 클릭은 백엔드의 unique 제약(SQLite `unique(symbol_id)`) 이 흡수. 사용자 피드백 없음. 좋은 UX 는 별 아이콘이 채워지거나 토스트 표시.

```tsx path=src/components/SymbolPanel.tsx lines=56-63
      <p className="text-sm text-fg">{symbol.shortMeaningKo}</p>

      {symbol.aliases.length > 0 && (
        <div className="text-xs text-fg-muted">
          <span className="font-medium text-fg">별칭:</span>{" "}
          {symbol.aliases.join(" · ")}
        </div>
      )}
```

**분석 / Analysis**

- 별칭 평탄 join. 패치 시 모두 한 줄에 — 너무 많으면 가독성 저하 가능.

```tsx path=src/components/SymbolPanel.tsx lines=65-83
      {symbol.domains.length > 0 && (
        <div>
          <div className="mb-1 text-xs font-medium text-fg">분야별 의미</div>
          <ul className="flex flex-col gap-1">
            {symbol.domains.map((d) => (
              <li key={d.code} className="rounded-md bg-bg px-2 py-1 text-xs">
                <span className="font-medium text-brand">{d.nameKo}</span>
                {d.meaningKo && (
                  <span className="ml-2 text-fg-muted">{d.meaningKo}</span>
                )}
              </li>
            ))}
          </ul>
        </div>
      )}

      {symbol.examples.length > 0 && (
        <div>
          <div className="mb-1 text-xs font-medium text-fg">예시</div>
          <ul className="flex flex-col gap-1">
            {symbol.examples.map((e) => (
              <li key={e.id} className="rounded-md bg-bg px-2 py-1">
                <code className="font-mono text-xs text-brand">{e.exampleLatex}</code>
                {e.exampleExplanationKo && (
                  <div className="mt-0.5 text-[11px] text-fg-muted">{e.exampleExplanationKo}</div>
                )}
              </li>
            ))}
          </ul>
        </div>
      )}
```

**분석 / Analysis**

- 도메인별/예시 두 섹션 모두 빈 배열이면 미렌더 — 작은 동적 카드 효과.
- key 가 `d.code`(unique), `e.id`(DB primary key) — 안전.
- 예시의 LaTeX 는 raw `<code>` — `mathjax` 렌더는 하지 않음. 단순 표시 의도.

```tsx path=src/components/SymbolPanel.tsx lines=104-139
      {symbol.similar.length > 0 && (
        <SimilarSymbols similar={symbol.similar} />
      )}
    </div>
  );
}

function SimilarSymbols({ similar }: { similar: SymbolHit[] }) {
  const selectToken = useAppStore((s) => s.selectToken);
  return (
    <div>
      <div className="mb-1 text-xs font-medium text-fg">유사 기호</div>
      <div className="flex flex-wrap gap-1">
        {similar.map((s) => (
          <button key={s.id} className="btn-outline !py-1 font-mono text-xs"
            title={s.shortMeaningKo}
            onClick={() =>
              selectToken({
                index: -1,
                tokenText: s.unicodeChar || s.latexCommand || "",
                symbolId: s.id,
                nameKo: s.canonicalNameKo,
                readingKo: s.readingKo ?? null,
                meaningKo: s.shortMeaningKo,
              })
            }>
            {s.unicodeChar || s.latexCommand} {s.canonicalNameKo}
          </button>
        ))}
      </div>
    </div>
  );
}
```

**분석 / Analysis**

- 별도 함수 컴포넌트 `SimilarSymbols` — `useAppStore` 호출이 조건부 렌더 아래에 있으므로 hook 규칙(상위 호출 위치 일관) 유지를 위해 분리. **중요한 패턴**: `if (symbol.similar.length > 0) { const sel = useAppStore(...) }` 는 hook 위반.
- 클릭 시 synthetic token 생성 (`index: -1`) 으로 `selectToken` 호출. *왜 -1*: 실제 토큰 인덱스가 없음(검색에서 들어온 항목). store 의 race 가드는 `symbolId` 기준이라 영향 없음.
- 결과: 사용자가 유사 기호 클릭 → store 가 새 detail fetch → SymbolPanel re-render.

**짚어볼 점 / Watch-outs**
- 즐겨찾기 추가 후 시각 피드백 부재.
- `aliases.join(" · ")` 이 매우 길어지면 패널 폭 초과.
- 별칭 카드 안에 `alias_type` 정보 없음 — 모든 종류가 한 곳에 섞여 표시.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 조건부 섹션 깔끔, hook 규칙 준수, a11y `aria-live`.
- 위험: 즐겨찾기 피드백 부재, alias 종류 미구분.
- 개선 권고:
  1. 즐겨찾기 토글 + 별 아이콘 fill 상태.
  2. 별칭을 `<Pill>` 컴포넌트로 분리, 종류별 색상.
  3. 예시 LaTeX 를 MathJax 로 렌더(`RenderView` 재사용).
- 재읽기 권고 레벨: 2.
