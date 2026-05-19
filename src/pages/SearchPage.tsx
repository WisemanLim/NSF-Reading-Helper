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

  const openDetail = async (id: number) => {
    try {
      const d = await ipc.symbolGet(id);
      setDetail(d);
    } catch (e) {
      console.warn(e);
    }
  };

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
                <button
                  onClick={() => openDetail(h.id)}
                  className="flex w-full items-center gap-3 rounded-lg border border-line bg-bg-elevated px-3 py-2 text-left text-sm hover:border-brand/60"
                >
                  <span className="w-12 font-mono text-lg">
                    {h.unicodeChar || h.latexCommand}
                  </span>
                  <span className="min-w-0 flex-1">
                    <span className="font-medium">{h.canonicalNameKo}</span>
                    {h.readingKo && (
                      <span className="ml-2 text-xs text-brand">
                        {h.readingKo}
                      </span>
                    )}
                    <span className="block truncate text-xs text-fg-muted">
                      {h.shortMeaningKo}
                    </span>
                  </span>
                  <span className="rounded bg-bg px-1.5 py-0.5 text-[10px] uppercase text-fg-muted">
                    {h.symbolType}
                  </span>
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
