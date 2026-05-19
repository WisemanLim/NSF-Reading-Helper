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
                <li
                  key={h.expressionId}
                  className="rounded-lg border border-line bg-bg-elevated p-3"
                >
                  <code className="block break-all font-mono text-xs text-brand">
                    {h.rawInput}
                  </code>
                  {h.speechKo && (
                    <div className="mt-1 text-xs text-fg-muted">{h.speechKo}</div>
                  )}
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
                <li
                  key={s.id}
                  className="flex items-center gap-3 rounded-lg border border-line bg-bg-elevated p-3"
                >
                  <span className="w-10 font-mono text-lg">
                    {s.unicodeChar || s.latexCommand}
                  </span>
                  <span className="min-w-0 flex-1">
                    <span className="font-medium">{s.canonicalNameKo}</span>
                    <span className="block truncate text-xs text-fg-muted">
                      {s.shortMeaningKo}
                    </span>
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
