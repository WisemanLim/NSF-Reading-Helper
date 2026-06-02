# src/lib/sre.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: Speech Rule Engine(SRE) 의 lazy loader + LaTeX/MathML → 음성 텍스트 변환 래퍼.
- LOC: 45
- 주요 export: `latexToSpeech`, `mathmlToSpeech`
- 외부 패키지: `speech-rule-engine` (dynamic import)
- 부수효과: 모듈 평가 시 한 번 초기화(Promise 캐시), 이후 호출 시 internal state set.

---

```ts path=src/lib/sre.ts lines=1-21
// Speech Rule Engine (SRE) wrapper. Loads lazily so the initial bundle stays small.
// SRE itself ships korean readings; we apply our own post-processor for symbols
// that don't have a canonical korean rule yet.

let _ready: Promise<any> | null = null;

async function load() {
  if (_ready) return _ready;
  _ready = (async () => {
    const sre: any = await import("speech-rule-engine");
    await sre.setupEngine({
      domain: "clearspeak",
      style: "default",
      locale: "ko",
      speech: "shallow",
      modality: "speech",
    });
    return sre;
  })();
  return _ready;
}
```

**분석 / Analysis**

- `_ready: Promise<any> | null` — Lazy singleton 패턴. **첫 호출에서만 import + 초기화**, 이후 호출은 같은 Promise를 재사용. *왜*: speech-rule-engine 은 큰 모듈이고 init 비용이 큼.
- `await import(...)` — Vite 가 별도 청크로 분리(코드 스플리팅). 초기 bundle 가벼움.
- `sre.setupEngine({...})` 호출 인자:
  - `domain: "clearspeak"` — 자연스러운 음성 스타일.
  - `style: "default"` — clearspeak 의 기본 변형.
  - `locale: "ko"` — 한국어 음성 데이터.
  - `speech: "shallow"` — 얕은 음성(중첩 구조 단순화).
  - `modality: "speech"` — speech(braille 아님) 출력.
- *흔한 실수*: `_ready` 가 한 번 set 되면 그 안의 `setupEngine` 호출이 실패해도 다시 시도 불가. 실패 시 fallback 으로 자연어 변환 실패만 알 수 있음.
- `: any` 캐스팅 — speech-rule-engine 의 타입 정의가 불완전해 캐스팅으로 우회. 그래도 *외부 API* 표면이 좁아 큰 위험 없음.

```ts path=src/lib/sre.ts lines=23-36
export async function latexToSpeech(
  latex: string,
  locale: "ko" | "en" = "ko",
): Promise<string> {
  const sre = await load();
  try {
    if (locale !== "ko") await sre.setupEngine({ locale });
    const out: string = sre.toSpeech(`<math>${sre.toMathml(latex)}</math>`);
    return out.trim();
  } catch (e) {
    console.warn("SRE failure, fallback", e);
    return latex;
  }
}
```

**분석 / Analysis**

- 두 단계: `toMathml(latex)` → `toSpeech(mathml)`. 내부에서 두 번 변환 비용 발생.
- `sre.setupEngine({ locale })` — locale 만 바꿔 재구성. 전역 상태 변경이라 **동시에 여러 locale 호출이 race** 할 가능성 존재. 본 앱은 ko 기본이라 위험 낮음.
- `<math>${sre.toMathml(latex)}</math>` — `toMathml` 의 출력이 `<math>` 루트를 포함하지 않는 fragment 일 때 보완. SRE 의 `toSpeech` 는 valid MathML 문서를 기대.
- catch 시 `latex` 원문 반환 — fallback 이지만 사용자에겐 raw LaTeX 가 noise. 호출자(`koSpeak`)에서 다시 한번 더 fallback 처리.

```ts path=src/lib/sre.ts lines=38-46
export async function mathmlToSpeech(
  mathml: string,
  locale: "ko" | "en" = "ko",
): Promise<string> {
  const sre = await load();
  if (locale !== "ko") await sre.setupEngine({ locale });
  return sre.toSpeech(mathml).trim();
}
```

**분석 / Analysis**

- 단순 wrapper. try-catch 없음 — caller 책임.

**짚어볼 점 / Watch-outs**
- 전역 상태 변경(`setupEngine`)이 race-prone. 단일 locale 시나리오에선 안전하지만 영어 분석 동시 시 결과 흔들림 가능.
- SRE 초기화 실패 시 영구 fail — `_ready` 재시도 로직 없음.
- `: any` 캐스팅으로 인한 타입 검증 부재.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: lazy 로딩 효과적, fallback 안전.
- 위험: locale race, init 영구 실패.
- 개선 권고:
  1. 초기화 실패 시 `_ready = null` 로 reset(다음 호출 재시도).
  2. locale 별로 별도 Promise 캐시(`_readyKo`, `_readyEn`).
  3. 타입 정의 보강(`@types/speech-rule-engine` 없음 → 자체 d.ts).
- 재읽기 권고 레벨: 2.
