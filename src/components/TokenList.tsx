import { cn } from "../lib/cn";
import type { AnalyzeToken } from "../types";

interface Props {
  tokens: AnalyzeToken[];
  selected: AnalyzeToken | null;
  onSelect: (t: AnalyzeToken) => void;
}

export function TokenList({ tokens, selected, onSelect }: Props) {
  if (tokens.length === 0) {
    return (
      <div className="rounded-lg border border-dashed border-line bg-bg-elevated px-3 py-4 text-xs text-fg-muted">
        분석된 토큰이 없습니다.
      </div>
    );
  }
  return (
    <div className="flex flex-wrap gap-1">
      {tokens.map((t) => {
        const active = selected?.index === t.index;
        return (
          <button
            key={`${t.index}-${t.tokenText}`}
            onClick={() => onSelect(t)}
            className={cn(
              "rounded-md border px-2 py-1 text-xs font-mono transition-colors",
              active
                ? "border-brand bg-brand-soft text-brand"
                : "border-line bg-bg-elevated text-fg hover:border-brand/60 hover:text-brand",
            )}
            title={t.nameKo ?? t.tokenText}
          >
            {t.tokenText}
            {t.nameKo && (
              <span className="ml-1 text-[10px] text-fg-muted">{t.nameKo}</span>
            )}
          </button>
        );
      })}
    </div>
  );
}
