import { useEffect, useRef } from "react";
import { renderLatex } from "../lib/mathjax";

interface Props {
  latex: string;
  display?: boolean;
  /** Korean speech text used for the accessible label so screen readers
   *  announce the formula's meaning rather than the raw LaTeX source. */
  ariaSpeech?: string;
}

export function RenderView({ latex, display = true, ariaSpeech }: Props) {
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
      aria-label={ariaSpeech || latex || "수식"}
      tabIndex={0}
      className="min-h-[80px] overflow-x-auto rounded-xl border border-line bg-bg-elevated p-6 text-center text-fg focus:outline-none focus-visible:ring-2 focus-visible:ring-brand"
    />
  );
}
