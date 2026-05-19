import { useState } from "react";
import { cn } from "../lib/cn";
import { detectInputFormat } from "../lib/detectFormat";
import type { InputFormat } from "../types";

const SAMPLES: { label: string; latex: string }[] = [
  { label: "적분", latex: "\\int_0^1 x^2 \\, dx" },
  { label: "합", latex: "\\sum_{i=1}^{n} i^2" },
  { label: "가우스", latex: "\\oint \\mathbf{E} \\cdot d\\mathbf{A} = \\frac{Q}{\\varepsilon_0}" },
  { label: "이차방정식", latex: "x = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}" },
  { label: "오일러", latex: "e^{i\\pi} + 1 = 0" },
  { label: "화학식", latex: "\\ce{H2 + O2 -> H2O}" },
  { label: "표준편차", latex: "\\sigma = \\sqrt{\\frac{1}{N}\\sum_{i=1}^N (x_i - \\mu)^2}" },
];

interface Props {
  value: string;
  onChange: (v: string) => void;
  onAnalyze: (format: InputFormat) => void;
  analyzing: boolean;
}

export function FormulaEditor({ value, onChange, onAnalyze, analyzing }: Props) {
  const [format, setFormat] = useState<InputFormat>("latex");

  const handleAnalyze = () => {
    const detected = detectInputFormat(value);
    setFormat(detected);
    onAnalyze(detected);
  };

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center justify-between">
        <label className="text-xs font-medium text-fg-muted">수식 입력</label>
        <div className="flex items-center gap-1 text-[11px] text-fg-subtle">
          감지 포맷: <span className="font-mono text-brand">{format}</span>
        </div>
      </div>
      <textarea
        className={cn("input-base min-h-[120px] resize-y font-mono")}
        value={value}
        onChange={(e) => {
          onChange(e.target.value);
          setFormat(detectInputFormat(e.target.value));
        }}
        placeholder={"예: \\int_0^1 x^2 \\, dx"}
        spellCheck={false}
      />
      <div className="flex flex-wrap gap-1">
        {SAMPLES.map((s) => (
          <button
            key={s.label}
            className="btn-outline !py-1 text-xs"
            onClick={() => {
              onChange(s.latex);
              setFormat("latex");
            }}
          >
            {s.label}
          </button>
        ))}
        <button
          className="btn-primary ml-auto"
          onClick={handleAnalyze}
          disabled={!value.trim() || analyzing}
        >
          {analyzing ? "분석 중..." : "분석 / 읽기"}
        </button>
      </div>
    </div>
  );
}
