# src/lib/detectFormat.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 사용자 입력 문자열의 포맷(LaTeX/MathML/AsciiMath/Unicode) 휴리스틱 추정.
- LOC: 13
- 주요 export: `detectInputFormat(input)`
- 외부 패키지: 없음
- 부수효과: 없음 (순수 함수)

---

```ts path=src/lib/detectFormat.ts lines=1-13
import type { InputFormat } from "../types";

const UNICODE_MATH = /[∑∫∮∂∇∞≈≠≤≥⊂⊆∈∉∀∃→⇌±×÷√Σπμσ°αβγδεθλ]/;

export function detectInputFormat(input: string): InputFormat {
  const t = input.trim();
  if (!t) return "latex";
  if (t.startsWith("<math") || t.includes("</math>")) return "mathml";
  if (t.includes("\\")) return "latex";
  if (UNICODE_MATH.test(t) && !/[a-zA-Z]{3,}/.test(t)) return "unicode";
  if (/sum_|int_|sqrt|frac|\^|_\(/.test(t)) return "asciimath";
  return "latex";
}
```

**분석 / Analysis**

- 우선순위 순서 / Order:
  1. 빈 입력 → `latex` 기본.
  2. `<math` 시작 또는 `</math>` 포함 → `mathml`.
  3. 백슬래시 포함 → `latex` (`\frac`, `\int` 등 LaTeX 명령의 강한 신호).
  4. 유니코드 수학 기호 + **연속 알파벳 ≥3** 없음 → `unicode` (예: `∫` 단독은 unicode, `int x dx` 는 asciimath 후보로 넘어감).
  5. asciimath 키워드(`sum_`, `int_`, `sqrt`, `frac`, `^`, `_(`) 매칭 → `asciimath`.
  6. 그 외 → `latex` (fallback).
- *흔한 실수*: 캐럿 `^` 와 언더스코어 `_` 는 LaTeX 에서도 흔함. 4단계 통과(LaTeX 백슬래시 없음)한 뒤에만 `asciimath` 판정 → 안전.
- *왜 `[a-zA-Z]{3,}` 부정 검사*: "alpha", "sin" 등 영문 토큰이 들어가면 LaTeX 또는 asciimath 가능성이 높음. 단순 unicode 기호 입력만 unicode 로 분류.
- *주의*: 정규식이 모듈 평가 시 한 번만 컴파일(`const`) — 좋음.

**짚어볼 점 / Watch-outs**
- 이 휴리스틱은 호출 시점이 `FormulaEditor` 같은 곳 — 명시적 사용 grep 필요. **실제 호출 위치를 grep 으로 찾아 보지 않으면 이 함수가 진짜 쓰이는지 불확실**. (분석 시점에 본 디렉토리에선 직접 import 가 적음.)
- AsciiMath 의 진짜 키워드(예: `prod_`, `oint_`, `lim_`)는 누락 — 사용자 시나리오에 따라 fallthrough.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 짧고 명료, 순서가 논리적.
- 위험: 휴리스틱 정밀도 한계, asciimath 키워드 부족.
- 개선 권고:
  1. AsciiMath 추가 키워드 (`prod_`, `oint_`, `lim_`, `to`, `=>`) 보강.
  2. 단위 테스트 추가 (한국어 단순 텍스트가 `latex` 로 떨어지는지 등).
- 재읽기 권고 레벨: 1.
