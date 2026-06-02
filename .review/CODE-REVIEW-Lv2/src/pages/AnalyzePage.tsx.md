# src/pages/AnalyzePage.tsx — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 메인 분석 화면. 입력 → 백엔드 analyze → koSpeak 폴리시 → 토큰 렌더 → 음성 컨트롤.
- LOC: 103
- 주요 export: `AnalyzePage`
- 의존 모듈: `components/*`, `ipc/client`, `lib/koSpeak`, `store/useAppStore`, `../types`
- 외부 패키지: `react`
- 부수효과: IPC 호출, 글로벌 store 변경
- 동시성: 단일 in-flight 가드(`analyzing`)는 표시용이고 실제 race 차단은 없음

---

```tsx path=src/pages/AnalyzePage.tsx lines=1-12
import { useEffect } from "react";
import { FormulaEditor } from "../components/FormulaEditor";
import { OcrDropzone } from "../components/OcrDropzone";
import { RenderView } from "../components/RenderView";
import { SpeechControls } from "../components/SpeechControls";
import { SymbolPanel } from "../components/SymbolPanel";
import { TokenList } from "../components/TokenList";
import { ipc } from "../ipc/client";
import { generateKoreanSpeech } from "../lib/koSpeak";
import { useAppStore } from "../store/useAppStore";
import type { AnalyzeResult, InputFormat } from "../types";
```

**분석 / Analysis**

- 7개 컴포넌트 + ipc + koSpeak + store + 타입 — 가장 의존이 두꺼운 페이지.

```tsx path=src/pages/AnalyzePage.tsx lines=13-23
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
```

**분석 / Analysis**

- 11개의 store 선택자. *왜 셀렉터 분리*: zustand 의 default 비교는 `===`. 함수/원시값을 각각 꺼내면 다른 키의 변경에 리렌더되지 않음.
- *흔한 실수*: 한 번에 객체로 묶으면 매번 새 참조 → 무한 리렌더. 11개 선택자는 약간 verbose 하지만 안전.

```tsx path=src/pages/AnalyzePage.tsx lines=25-27
  useEffect(() => {
    if (!input && !result) setInput("\\int_0^1 x^2 \\, dx");
  }, [input, result, setInput]);
```

**분석 / Analysis**

- 첫 진입 시 데모 LaTeX 자동 채움. *왜 if 가드*: 사용자가 다른 페이지에 다녀와도 자신의 입력을 보존.
- StrictMode 에선 effect 가 2번 실행 — 두 번째 실행에서 `input` 이 이미 세팅되어 가드 통과 안 됨. 안전.

```tsx path=src/pages/AnalyzePage.tsx lines=29-67
  const handleAnalyze = async (format: InputFormat) => {
    if (!input.trim()) return;
    setAnalyzing(true);
    try {
      let r: AnalyzeResult;
      try {
        r = await ipc.analyze({ rawInput: input, format, locale: "ko" });
      } catch (e) {
        console.warn("backend analyze unavailable, falling back to client", e);
        r = {
          normalizedLatex: input,
          speech: { ko: "" },
          tokens: [],
        };
      }
      // Always layer the Korean speech polisher on top — even when the backend
      // produced a token-list, our template+SRE pipeline gives better Korean.
      try {
        const ko = await generateKoreanSpeech({
          latex: r.normalizedLatex ?? input,
          tokens: r.tokens ?? [],
        });
        r = { ...r, speech: { ...r.speech, ko } };
      } catch (e) {
        console.debug("koSpeak failed", e);
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
```

**분석 / Analysis**

- 중첩 try-catch 의 의도:
  - 내부 try: 백엔드 analyze 가 실패해도 빈 결과로 진행 → koSpeak 가 작동하면 사용자 경험 보전.
  - 두 번째 try: koSpeak 실패는 무시(콘솔 debug).
  - 외부 try: 위 두 단계 외 예상치 못한 에러(예: `setResult` 등) — 메시지로 표시.
- *짚어볼 점*: 백엔드 실패 시 사용자에게 "오류" 가 표시되지 않음(메시지 없이 빈 결과). koSpeak 가 어떤 음성을 만들면 사용자는 "정상" 처럼 느낌. 디버깅 어려움.
- `reloadHistory()` — async 함수를 await 없이 호출(fire-and-forget). 약간 늦은 history 갱신 허용.
- *흔한 실수*: 두 번 연속 분석 시 첫 응답이 나중에 도착해 두 번째 결과를 덮어쓸 race 가 있음. `analyzing` 플래그가 사용자 버튼 disable 에 쓰일 뿐 race 차단 안 함.

```tsx path=src/pages/AnalyzePage.tsx lines=69-103
  return (
    <div className="flex h-full flex-col overflow-hidden">
      <header className="flex h-14 shrink-0 items-center border-b border-line px-6">
        <h1 className="text-sm font-semibold text-fg-muted">분석</h1>
      </header>
      <div className="grid min-h-0 flex-1 grid-cols-1 gap-4 overflow-hidden p-4 lg:grid-cols-[minmax(0,1fr)_380px]">
        <section className="flex min-h-0 flex-col gap-4 overflow-y-auto pr-1">
          <FormulaEditor value={input} onChange={setInput} onAnalyze={handleAnalyze} analyzing={analyzing} />
          <OcrDropzone onResult={(latex) => setInput(latex)} />
          <RenderView latex={result?.normalizedLatex || input} ariaSpeech={result?.speech.ko} />
          <SpeechControls text={result?.speech.ko || ""} />
          <div>
            <div className="mb-1 text-xs font-medium text-fg-muted">토큰</div>
            <TokenList tokens={result?.tokens ?? []} selected={selected} onSelect={(t) => selectToken(t)} />
          </div>
        </section>
        <aside className="min-h-0 overflow-y-auto">
          <SymbolPanel symbol={selectedSymbol} />
        </aside>
      </div>
    </div>
  );
}
```

**분석 / Analysis**

- `lg:grid-cols-[minmax(0,1fr)_380px]` — 데스크탑은 좌측 가변 + 우측 고정 380px. `minmax(0, 1fr)` 는 grid 가 자식 콘텐츠 크기로 인해 트랙이 늘어나는 현상 방지(중요 idiom).
- `<RenderView latex={result?.normalizedLatex || input} />` — 분석 결과 있으면 normalized, 없으면 입력 원문. `??` 가 아닌 `||` 사용 — `""` 도 fallthrough 의도. 합리적.
- `ariaSpeech={result?.speech.ko}` — 시각장애인 대상 a11y. RenderView 가 `aria-label` 로 사용.

**짚어볼 점 / Watch-outs**
- 분석 race 차단 없음 — 빠른 두 번 클릭 결과 비결정적.
- 백엔드 실패가 사용자에게 거의 보이지 않음(빈 결과 + 콘솔 warn).

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 폴백 전략 적절, 컴포넌트 합성 명료.
- 위험: race, 사용자 피드백 부족.
- 개선 권고:
  1. `handleAnalyze` 안에 `requestId` + 이펙트 단조 가드.
  2. 백엔드 실패 시 토스트 또는 상태 메시지 표시.
- 재읽기 권고 레벨: 2.
