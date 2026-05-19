import { useRef } from "react";
import { cn } from "../lib/cn";
import type { AnalyzeToken } from "../types";

interface Props {
  tokens: AnalyzeToken[];
  selected: AnalyzeToken | null;
  onSelect: (t: AnalyzeToken) => void;
}

export function TokenList({ tokens, selected, onSelect }: Props) {
  const refs = useRef<(HTMLButtonElement | null)[]>([]);

  if (tokens.length === 0) {
    return (
      <div className="rounded-lg border border-dashed border-line bg-bg-elevated px-3 py-4 text-xs text-fg-muted">
        분석된 토큰이 없습니다.
      </div>
    );
  }

  const onKeyDown = (e: React.KeyboardEvent, idx: number) => {
    let next = idx;
    switch (e.key) {
      case "ArrowRight":
      case "ArrowDown":
        next = (idx + 1) % tokens.length;
        break;
      case "ArrowLeft":
      case "ArrowUp":
        next = (idx - 1 + tokens.length) % tokens.length;
        break;
      case "Home":
        next = 0;
        break;
      case "End":
        next = tokens.length - 1;
        break;
      case "Enter":
      case " ":
        e.preventDefault();
        onSelect(tokens[idx]);
        return;
      default:
        return;
    }
    e.preventDefault();
    refs.current[next]?.focus();
    onSelect(tokens[next]);
  };

  return (
    <div
      role="listbox"
      aria-label="수식 토큰"
      className="flex flex-wrap gap-1"
    >
      {tokens.map((t, i) => {
        const active = selected?.index === t.index;
        return (
          <button
            key={`${t.index}-${t.tokenText}`}
            ref={(el) => {
              refs.current[i] = el;
            }}
            role="option"
            aria-selected={active}
            aria-label={`${t.tokenText}${t.nameKo ? `: ${t.nameKo}` : ""}${t.meaningKo ? `, ${t.meaningKo}` : ""}`}
            tabIndex={active || (!selected && i === 0) ? 0 : -1}
            onClick={() => onSelect(t)}
            onKeyDown={(e) => onKeyDown(e, i)}
            className={cn(
              "rounded-md border px-2 py-1 text-xs font-mono transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-brand",
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
