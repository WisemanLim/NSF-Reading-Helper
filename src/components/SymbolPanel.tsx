import { ipc } from "../ipc/client";
import { useAppStore } from "../store/useAppStore";
import type { SymbolDetail, SymbolHit } from "../types";
import { IconStar } from "./icons";

interface Props {
  symbol: SymbolDetail | null;
}

export function SymbolPanel({ symbol }: Props) {
  if (!symbol) {
    return (
      <div
        role="status"
        aria-live="polite"
        className="rounded-xl border border-dashed border-line bg-bg-elevated p-4 text-sm text-fg-muted"
      >
        토큰을 클릭하면 기호 설명이 표시됩니다.
      </div>
    );
  }

  return (
    <div
      role="region"
      aria-label="기호 설명"
      aria-live="polite"
      className="flex flex-col gap-3 rounded-xl border border-line bg-bg-elevated p-4"
    >
      <div className="flex items-start justify-between gap-2">
        <div>
          <div className="font-mono text-2xl text-fg">
            {symbol.unicodeChar || symbol.latexCommand}
          </div>
          <div className="text-sm font-medium text-fg">
            {symbol.canonicalNameKo}
            {symbol.canonicalNameEn && (
              <span className="ml-2 text-fg-muted">
                · {symbol.canonicalNameEn}
              </span>
            )}
          </div>
          {symbol.readingKo && (
            <div className="text-xs text-brand">읽기: {symbol.readingKo}</div>
          )}
        </div>
        <button
          className="btn-outline !py-1"
          title="즐겨찾기 추가"
          onClick={() => ipc.favoritesAdd(symbol.id).catch(console.warn)}
        >
          <IconStar />
        </button>
      </div>

      <p className="text-sm text-fg">{symbol.shortMeaningKo}</p>

      {symbol.aliases.length > 0 && (
        <div className="text-xs text-fg-muted">
          <span className="font-medium text-fg">별칭:</span>{" "}
          {symbol.aliases.join(" · ")}
        </div>
      )}

      {symbol.domains.length > 0 && (
        <div>
          <div className="mb-1 text-xs font-medium text-fg">분야별 의미</div>
          <ul className="flex flex-col gap-1">
            {symbol.domains.map((d) => (
              <li
                key={d.code}
                className="rounded-md bg-bg px-2 py-1 text-xs"
              >
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
                <code className="font-mono text-xs text-brand">
                  {e.exampleLatex}
                </code>
                {e.exampleExplanationKo && (
                  <div className="mt-0.5 text-[11px] text-fg-muted">
                    {e.exampleExplanationKo}
                  </div>
                )}
              </li>
            ))}
          </ul>
        </div>
      )}

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
          <button
            key={s.id}
            className="btn-outline !py-1 font-mono text-xs"
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
            }
          >
            {s.unicodeChar || s.latexCommand} {s.canonicalNameKo}
          </button>
        ))}
      </div>
    </div>
  );
}
