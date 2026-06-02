# src/components/HandwritingPad.tsx — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: Canvas 기반 손그림 입력 → `classifyStrokes` 호출 → 상위 8개 후보를 버튼으로 표시.
- LOC: 178
- 주요 export: `HandwritingPad`
- 의존 모듈: `../lib/detexify`
- 외부 패키지: `react`
- 부수효과: Canvas 2D 컨텍스트, PointerEvents 캡처, 네트워크 호출

---

```tsx path=src/components/HandwritingPad.tsx lines=1-17
// Detexify-style handwriting input. ...
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
```

**분석 / Analysis**

- 세 ref:
  - `canvasRef`: DOM 노드.
  - `strokesRef`: 누적된 모든 stroke(저장본). *왜 ref*: 매 stroke 종료마다 setState 하면 캔버스 리렌더 비용 발생.
  - `currentRef`: 진행 중 stroke(pointerdown ~ pointerup 사이).

```tsx path=src/components/HandwritingPad.tsx lines=18-24
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const [hits, setHits] = useState<ClassifyHit[]>([]);

  // Initialize canvas + DPI-scaled drawing context.
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
```

```tsx path=src/components/HandwritingPad.tsx lines=25-42
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
```

**분석 / Analysis**

- **HiDPI 스케일링** / *왜*: 캔버스 width/height(픽셀) 와 CSS width/height(레이아웃) 가 다르면 흐릿. `dpr` 배수로 backing store 늘리고 `ctx.scale(dpr, dpr)` 로 그리기 좌표는 CSS 단위 유지.
- **마운트 시 한 번만** 호출 — *흔한 실수*: 윈도우 리사이즈/dpr 변경(모니터 이동) 시 캔버스가 흐려짐. resize listener 부재.
- `strokeStyle` 색상 하드코딩 — 테마 변경 시 어색.

```tsx path=src/components/HandwritingPad.tsx lines=44-54
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
```

**분석 / Analysis**

- **정규화 좌표** (0~1). Detexify 가 정규화 좌표를 받아도 동작 — 백엔드/공개 API 양쪽 호환.
- `useCallback(..., [])` — handler 안정 ref. 매 렌더 새 함수 방지(필수는 아니지만 안전).

```tsx path=src/components/HandwritingPad.tsx lines=55-75
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
```

**분석 / Analysis**

- `setPointerCapture` — 포인터가 캔버스 밖으로 나가도 이벤트가 캔버스에 계속 전달. *왜*: stroke 도중 마우스가 빠르게 움직여 캔버스 영역 벗어나면 이벤트 손실 방지.
- `pointerdown` 에서 첫 점 푸시 + `drawDot`(점 형태로 시각화) — 사용자가 짧게 탭해도 흔적 남김.
- `pointermove` 에서 마지막 점 → 새 점 segment 그리기.
- `pointerup` 에서 현재 stroke 를 strokesRef 에 append, currentRef 비움.
- *흔한 실수*: `pointercancel` 도 동일 처리 — 본 컴포넌트는 `onPointerCancel={onPointerUp}` 으로 jsx 에서 연결 (아래 145행 부근).

```tsx path=src/components/HandwritingPad.tsx lines=77-97
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
```

**분석 / Analysis**

- 좌표 변환: 정규화 → CSS 픽셀 (`canvas.clientWidth` 곱). 위 `ctx.scale(dpr,dpr)` 로 DPR 처리되어 있어 CSS 단위 그대로 그려도 됨.
- *흔한 실수*: `getContext("2d")` 를 매번 호출 — 같은 컨텍스트 객체 반환이라 성능 영향 미미. ref 로 보관해 절감 가능.

```tsx path=src/components/HandwritingPad.tsx lines=99-123
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
```

**분석 / Analysis**

- `clear` — strokes + hits + err 모두 초기화 후 픽셀 지우기.
- `recognize` — 빈 strokes 가드, busy 토글, slice(0,8) 로 후보 상한.
- **race**: 빠르게 recognize 두 번 누르면 두 응답 중 마지막이 표시. 결과는 무관해도 OK(strokes 동일).

```tsx path=src/components/HandwritingPad.tsx lines=125-178
  return (
    <div className="flex flex-col gap-2 rounded-xl border border-line bg-bg-elevated p-3">
      <div className="flex items-center justify-between">
        <span className="text-xs font-medium text-fg">손그림으로 기호 찾기</span>
        <div className="flex gap-1">
          <button className="btn-outline !py-1 text-xs" onClick={clear}>지우기</button>
          <button className="btn-primary !py-1 text-xs" onClick={recognize} disabled={busy}>
            {busy ? "인식 중..." : "인식"}
          </button>
        </div>
      </div>
      <canvas ref={canvasRef}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerCancel={onPointerUp}
        className="h-44 w-full touch-none cursor-crosshair rounded-lg border border-dashed border-line bg-bg"
        role="img"
        aria-label="수식 기호 손그림 입력 캔버스" />
      {err && <div className="text-xs text-danger">오류: {err}</div>}
      {hits.length > 0 && (
        <div>
          <div className="mb-1 text-xs text-fg-muted">상위 후보 (Detexify)</div>
          <div className="flex flex-wrap gap-1">
            {hits.map((h, i) => (
              <button key={`${h.command}-${i}`}
                className="btn-outline !py-1 text-xs"
                title={`${h.package ?? "—"} · score ${h.score.toFixed(3)}`}
                onClick={() => onPickSymbol(h.command)}>
                <span className="font-mono">{h.command}</span>
                <span className="ml-1 text-[10px] text-fg-subtle">
                  {(h.score * 100).toFixed(0)}%
                </span>
              </button>
            ))}
          </div>
          <p className="mt-1 text-[10px] text-fg-subtle">후보를 클릭하면 기호 사전에서 검색합니다.</p>
        </div>
      )}
    </div>
  );
}
```

**분석 / Analysis**

- `touch-none` — Tailwind 가 `touch-action: none` 설정. 모바일에서 스크롤 대신 그리기 가능.
- `role="img" aria-label` — 캔버스 자체에 시맨틱 라벨. (정확히 image 는 아니지만 a11y 도구가 캔버스를 다룰 단서.)
- 후보 버튼 key 가 `${h.command}-${i}` — 중복 command 가능성에 대비해 index 결합.
- `title` 툴팁에 package + score(0~1) — 디버그 친화.

**짚어볼 점 / Watch-outs**
- 리사이즈 미대응. 사용자가 창 크기 바꾸면 캔버스가 흐려지거나 좌표 어긋남.
- DPR 변경 시 동일.
- 외부 호스트로 손그림 자동 전송(`classifyStrokes` 의 fallback) — `lib/detexify.ts` 참조.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: HiDPI 처리, PointerEvents 사용, a11y 라벨.
- 위험: 리사이즈 미대응, 외부 호스트 전송 가시성 부족.
- 개선 권고:
  1. `ResizeObserver` 로 캔버스 백킹 스토어 재설정.
  2. context ref 보관.
  3. Detexify fallback 정책을 UI 에 명시(예: "외부 서비스에 전송됨").
- 재읽기 권고 레벨: 2.
