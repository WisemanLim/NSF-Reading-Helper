import { useEffect } from "react";
import { FormulaEditor } from "../components/FormulaEditor";
import { RenderView } from "../components/RenderView";
import { SpeechControls } from "../components/SpeechControls";
import { SymbolPanel } from "../components/SymbolPanel";
import { TokenList } from "../components/TokenList";
import { ipc } from "../ipc/client";
import { latexToSpeech } from "../lib/sre";
import { useAppStore } from "../store/useAppStore";
import type { AnalyzeResult, InputFormat } from "../types";

export function AnalyzePage() {
  const input = useAppStore((s) => s.currentInput);
  const setInput = useAppStore((s) => s.setInput);
  const setResult = useAppStore((s) => s.setResult);
  const result = useAppStore((s) => s.currentResult);
  const selected = useAppStore((s) => s.selectedToken);
  const selectedSymbol = useAppStore((s) => s.selectedSymbol);
  const selectToken = useAppStore((s) => s.selectToken);
  const analyzing = useAppStore((s) => s.analyzing);
  const setAnalyzing = useAppStore((s) => s.setAnalyzing);
  const reloadHistory = useAppStore((s) => s.reloadHistory);

  useEffect(() => {
    if (!input && !result) setInput("\\int_0^1 x^2 \\, dx");
  }, [input, result, setInput]);

  const handleAnalyze = async (format: InputFormat) => {
    if (!input.trim()) return;
    setAnalyzing(true);
    try {
      let r: AnalyzeResult;
      try {
        r = await ipc.analyze({ rawInput: input, format, locale: "ko" });
      } catch (e) {
        console.warn("backend analyze unavailable, falling back to client", e);
        const speech = await latexToSpeech(input, "ko");
        r = {
          normalizedLatex: input,
          speech: { ko: speech },
          tokens: [],
        };
      }
      if (!r.speech?.ko || r.speech.ko === input) {
        try {
          r = { ...r, speech: { ...r.speech, ko: await latexToSpeech(input, "ko") } };
        } catch {
          /* keep server speech */
        }
      }
      setResult(r);
      reloadHistory();
    } catch (e) {
      console.error("analyze failed", e);
      setResult({
        normalizedLatex: input,
        speech: { ko: `오류: ${e instanceof Error ? e.message : String(e)}` },
        tokens: [],
      });
    } finally {
      setAnalyzing(false);
    }
  };

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <header className="flex h-14 shrink-0 items-center border-b border-line px-6">
        <h1 className="text-sm font-semibold text-fg-muted">분석</h1>
      </header>
      <div className="grid min-h-0 flex-1 grid-cols-1 gap-4 overflow-hidden p-4 lg:grid-cols-[minmax(0,1fr)_380px]">
        <section className="flex min-h-0 flex-col gap-4 overflow-y-auto pr-1">
          <FormulaEditor
            value={input}
            onChange={setInput}
            onAnalyze={handleAnalyze}
            analyzing={analyzing}
          />
          <RenderView latex={result?.normalizedLatex || input} />
          <SpeechControls text={result?.speech.ko || ""} />
          <div>
            <div className="mb-1 text-xs font-medium text-fg-muted">토큰</div>
            <TokenList
              tokens={result?.tokens ?? []}
              selected={selected}
              onSelect={(t) => selectToken(t)}
            />
          </div>
        </section>
        <aside className="min-h-0 overflow-y-auto">
          <SymbolPanel symbol={selectedSymbol} />
        </aside>
      </div>
    </div>
  );
}
