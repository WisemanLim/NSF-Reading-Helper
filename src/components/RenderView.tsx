import { useEffect, useRef } from "react";
import { renderLatex } from "../lib/mathjax";

interface Props {
  latex: string;
  display?: boolean;
}

export function RenderView({ latex, display = true }: Props) {
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!ref.current) return;
    if (!latex.trim()) {
      ref.current.textContent = "";
      return;
    }
    renderLatex(ref.current, latex, display).catch((e) => {
      if (ref.current) {
        ref.current.textContent = `렌더링 오류: ${
          e instanceof Error ? e.message : String(e)
        }`;
      }
    });
  }, [latex, display]);

  return (
    <div
      ref={ref}
      role="math"
      aria-label={latex}
      className="min-h-[80px] overflow-x-auto rounded-xl border border-line bg-bg-elevated p-6 text-center text-fg"
    />
  );
}
