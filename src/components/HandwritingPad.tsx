// Detexify-style handwriting input. User draws a symbol on the canvas;
// strokes are captured as [[{x,y}, ...], ...] and sent to the classifier,
// which returns LaTeX command suggestions. We then resolve those against the
// local symbol dictionary via the existing search IPC.

import { useCallback, useEffect, useRef, useState } from "react";
import { classifyStrokes, type ClassifyHit } from "../lib/detexify";

interface Props {
  onPickSymbol: (latex: string) => void;
}

interface Point {
  x: number;
  y: number;
}

export function HandwritingPad({ onPickSymbol }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const strokesRef = useRef<Point[][]>([]);
  const currentRef = useRef<Point[] | null>(null);
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const [hits, setHits] = useState<ClassifyHit[]>([]);

  // Initialize canvas + DPI-scaled drawing context.
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const dpr = window.devicePixelRatio || 1;
    const cssW = canvas.clientWidth;
    const cssH = canvas.clientHeight;
    canvas.width = cssW * dpr;
    canvas.height = cssH * dpr;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.scale(dpr, dpr);
    ctx.strokeStyle = "#e7eaef";
    ctx.lineWidth = 3;
    ctx.lineCap = "round";
    ctx.lineJoin = "round";
  }, []);

  const eventToPoint = useCallback(
    (e: React.PointerEvent<HTMLCanvasElement>): Point => {
      const rect = e.currentTarget.getBoundingClientRect();
      return {
        x: (e.clientX - rect.left) / rect.width,
        y: (e.clientY - rect.top) / rect.height,
      };
    },
    [],
  );

  const onPointerDown = (e: React.PointerEvent<HTMLCanvasElement>) => {
    e.currentTarget.setPointerCapture(e.pointerId);
    const p = eventToPoint(e);
    currentRef.current = [p];
    drawDot(p);
  };

  const onPointerMove = (e: React.PointerEvent<HTMLCanvasElement>) => {
    if (!currentRef.current) return;
    const p = eventToPoint(e);
    const prev = currentRef.current[currentRef.current.length - 1];
    currentRef.current.push(p);
    drawSegment(prev, p);
  };

  const onPointerUp = (e: React.PointerEvent<HTMLCanvasElement>) => {
    if (!currentRef.current) return;
    e.currentTarget.releasePointerCapture(e.pointerId);
    strokesRef.current.push(currentRef.current);
    currentRef.current = null;
  };

  function drawDot(p: Point) {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.beginPath();
    ctx.arc(p.x * canvas.clientWidth, p.y * canvas.clientHeight, 1.5, 0, 2 * Math.PI);
    ctx.fillStyle = "#e7eaef";
    ctx.fill();
  }

  function drawSegment(a: Point, b: Point) {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.beginPath();
    ctx.moveTo(a.x * canvas.clientWidth, a.y * canvas.clientHeight);
    ctx.lineTo(b.x * canvas.clientWidth, b.y * canvas.clientHeight);
    ctx.stroke();
  }

  const clear = () => {
    strokesRef.current = [];
    setHits([]);
    setErr(null);
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.clearRect(0, 0, canvas.width, canvas.height);
  };

  const recognize = async () => {
    if (strokesRef.current.length === 0) return;
    setBusy(true);
    setErr(null);
    setHits([]);
    try {
      const results = await classifyStrokes(strokesRef.current);
      setHits(results.slice(0, 8));
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="flex flex-col gap-2 rounded-xl border border-line bg-bg-elevated p-3">
      <div className="flex items-center justify-between">
        <span className="text-xs font-medium text-fg">손그림으로 기호 찾기</span>
        <div className="flex gap-1">
          <button className="btn-outline !py-1 text-xs" onClick={clear}>
            지우기
          </button>
          <button
            className="btn-primary !py-1 text-xs"
            onClick={recognize}
            disabled={busy}
          >
            {busy ? "인식 중..." : "인식"}
          </button>
        </div>
      </div>
      <canvas
        ref={canvasRef}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerCancel={onPointerUp}
        className="h-44 w-full touch-none cursor-crosshair rounded-lg border border-dashed border-line bg-bg"
        role="img"
        aria-label="수식 기호 손그림 입력 캔버스"
      />
      {err && <div className="text-xs text-danger">오류: {err}</div>}
      {hits.length > 0 && (
        <div>
          <div className="mb-1 text-xs text-fg-muted">상위 후보 (Detexify)</div>
          <div className="flex flex-wrap gap-1">
            {hits.map((h, i) => (
              <button
                key={`${h.command}-${i}`}
                className="btn-outline !py-1 text-xs"
                title={`${h.package ?? "—"} · score ${h.score.toFixed(3)}`}
                onClick={() => onPickSymbol(h.command)}
              >
                <span className="font-mono">{h.command}</span>
                <span className="ml-1 text-[10px] text-fg-subtle">
                  {(h.score * 100).toFixed(0)}%
                </span>
              </button>
            ))}
          </div>
          <p className="mt-1 text-[10px] text-fg-subtle">
            후보를 클릭하면 기호 사전에서 검색합니다.
          </p>
        </div>
      )}
    </div>
  );
}
