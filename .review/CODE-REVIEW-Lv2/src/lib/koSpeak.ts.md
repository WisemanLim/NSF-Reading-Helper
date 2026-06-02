# src/lib/koSpeak.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 한국어 음성 생성 파이프라인. 1) 템플릿 매칭 → 2) SRE 호출 + 후처리 → 3) 토큰 폴백.
- LOC: 115
- 주요 export: `generateKoreanSpeech`, `generateKoreanSpeechSync`, `SpeechInput`
- 의존 모듈: `./sre`, `./ko-clearspeak.json`, `../types`
- 외부 패키지: 없음
- 부수효과: 정규식 컴파일 매번
- 동시성: 비동기

---

```ts path=src/lib/koSpeak.ts lines=1-27
// Korean speech polish layer.
//
// Pipeline:
//   1. Try template matching (frac, sum, int, lim, …) for natural phrasing.
//   2. If no template matches, fall back to SRE for Korean speech, then run a
//      post-processor (regex replacements + lexicon swaps).
//   3. Optionally inject canonical Korean readings from the backend's token
//      table so domain-specific symbols get the curated pronunciation.

import rules from "./ko-clearspeak.json";
import { latexToSpeech } from "./sre";
import type { AnalyzeToken } from "../types";

interface TemplateRule {
  pattern: string;
  speech: string;
}
interface Replace {
  from: string;
  to: string;
}
interface RulePack {
  templates: TemplateRule[];
  replacements: Replace[];
  sre_post: Replace[];
  lexicon: Record<string, string>;
}

const PACK = rules as unknown as RulePack;
```

**분석 / Analysis**

- `import rules from "./ko-clearspeak.json"` — Vite 가 JSON 모듈을 import 가능. 빌드 시 정적 포함.
- 4종 데이터: `templates`(패턴→음성), `replacements`(인라인 토큰 변환), `sre_post`(SRE 결과 정리), `lexicon`(그리스 문자→한글).
- `as unknown as RulePack` — JSON 의 런타임 타입은 검증되지 않으나 강제 단언. *왜 unknown 경유*: TS 가 `as Direct` 를 거부할 수 있는 경우 우회. 안전성은 JSON 작성자에 의존.

```ts path=src/lib/koSpeak.ts lines=31-46
/** Try one of the LaTeX templates. Returns the formatted speech or null. */
function tryTemplate(latex: string): string | null {
  const trimmed = latex.trim();
  for (const rule of PACK.templates) {
    const re = new RegExp(rule.pattern);
    const m = trimmed.match(re);
    if (m) {
      let out = rule.speech;
      for (let i = 1; i < m.length; i++) {
        out = out.replace(new RegExp(`\\$${i}`, "g"), polishInline(m[i] ?? ""));
      }
      return out.trim().replace(/\s+/g, " ");
    }
  }
  return null;
}
```

**분석 / Analysis**

- 각 템플릿마다 `new RegExp(rule.pattern)` — **매 호출마다 모든 패턴 재컴파일**. 성능 핫스팟. 캐시 가능: 모듈 평가 시 `const compiled = PACK.templates.map(t => ({ ...t, re: new RegExp(t.pattern) }))`.
- `m.length` 가 1 이면 캡처가 없는 패턴 — for 루프가 돌지 않아 `out` 가 그대로.
- `out.replace(new RegExp(\`\\$${i}\`, "g"), ...)` — 음성 템플릿 안의 `$1`, `$2` 를 캡처 그룹 텍스트로 치환. **여기서도 매번 정규식 새로 생성**.
- `polishInline(m[i] ?? "")` — 캡처가 undefined 면 빈 문자열.
- `trim().replace(/\s+/g, " ")` — 연속 공백 정리.
- *흔한 실수*: 사용자 입력이 패턴 정규식을 통과시키지만 매우 큰 문자열일 때 `match` 가 backtracking 으로 폭주(catastrophic backtracking) 가능. JSON 안의 패턴 품질에 의존.

```ts path=src/lib/koSpeak.ts lines=48-61
/** Apply replacements + lexicon swaps to an inline LaTeX fragment so template
 *  captures like `$1` come out reading-friendly. */
function polishInline(s: string): string {
  let r = s;
  for (const rep of PACK.replacements) {
    r = r.replace(new RegExp(rep.from, "g"), rep.to);
  }
  // Greek lexicon — match \alpha / \\alpha / alpha
  r = r.replace(/\\([A-Za-z]+)/g, (_, name: string) => {
    const k = name.toLowerCase();
    return PACK.lexicon[k] ?? name;
  });
  return r.trim();
}
```

**분석 / Analysis**

- 두 단계: 1) replacements 일반 치환, 2) 그리스 문자 lexicon 치환.
- 그리스 lexicon: `\alpha` → 한글 "알파" 식.
- `replacements` 의 `from` 도 정규식 — JSON 작성자가 escape 책임. 잘못된 패턴은 throw.

```ts path=src/lib/koSpeak.ts lines=63-72
/** Polish an SRE-generated Korean string further. SRE's `ko` locale is partial,
 *  so the output often contains untranslated English fragments. */
function polishSreOutput(raw: string): string {
  let r = raw;
  for (const rep of PACK.sre_post) {
    r = r.replace(new RegExp(rep.from, "g"), rep.to);
  }
  return r.replace(/\s+/g, " ").trim();
}
```

**분석 / Analysis**

- SRE 의 한국어 locale 이 부분 번역만 한다는 점을 보완. 운영 데이터로 잘 만들면 자연스럽게 들림.

```ts path=src/lib/koSpeak.ts lines=74-80
/** Build a token-driven fallback when SRE and templates both fail. */
function fromTokens(tokens: AnalyzeToken[]): string {
  if (tokens.length === 0) return "";
  return tokens
    .map((t) => t.readingKo || t.nameKo || t.tokenText)
    .join(" ");
}
```

**분석 / Analysis**

- 최후 폴백 — 백엔드가 토큰별로 보낸 `readingKo`, `nameKo`, 원문 텍스트 우선순.

```ts path=src/lib/koSpeak.ts lines=82-107
export interface SpeechInput {
  latex: string;
  tokens?: AnalyzeToken[];
  preferTemplate?: boolean;
}

export async function generateKoreanSpeech(
  input: SpeechInput,
): Promise<string> {
  const { latex, tokens = [], preferTemplate = true } = input;

  if (preferTemplate) {
    const tpl = tryTemplate(latex);
    if (tpl) return tpl;
  }

  try {
    const raw = await latexToSpeech(latex, "ko");
    const polished = polishSreOutput(raw);
    if (polished && polished !== latex) return polished;
  } catch (e) {
    console.debug("SRE failed, falling back to token linearization", e);
  }

  const fallback = fromTokens(tokens);
  return fallback || latex;
}
```

**분석 / Analysis**

- 우선순위 / Order:
  1. 템플릿 매치 (사용자가 `preferTemplate=false` 명시하지 않은 한).
  2. SRE + post-process. 결과가 비었거나 latex 와 같으면 거부.
  3. 토큰 폴백.
  4. 결국 모두 실패면 원문 latex 반환.
- `polished !== latex` — SRE 가 변환에 실패해 원문을 그대로 반환했을 때 그 결과를 "성공" 으로 보지 않음.
- `console.debug` — production 에선 보이지 않음(브라우저 기본 레벨).

```ts path=src/lib/koSpeak.ts lines=109-115
/** Synchronous version for cases where SRE isn't needed (templates only). */
export function generateKoreanSpeechSync(
  latex: string,
  tokens: AnalyzeToken[] = [],
): string {
  return tryTemplate(latex) ?? fromTokens(tokens) ?? latex;
}
```

**분석 / Analysis**

- `?? fromTokens(tokens)` 가 항상 문자열을 반환하므로 마지막 `?? latex` 는 unreachable(빈 문자열은 falsy 가 아님). 실제로는 `fromTokens` 의 `""` 반환 시 `?? latex` 가 *작동하지 않음*(`""` 가 truthy → 그대로). 안전하게 `||` 가 의도.
- *잠재 버그*: `fromTokens([])` = `""` → 그대로 반환되어 `latex` 폴백 미실행.

**짚어볼 점 / Watch-outs**
- **성능**: 모든 패턴/치환 정규식이 호출마다 재컴파일. 모듈 평가 시 사전 컴파일하면 무시할 수 없는 개선.
- JSON 데이터의 잘못된 정규식 패턴은 첫 호출에서 런타임 throw.
- `??` 와 `||` 의 의미 차이를 동기 함수에서 혼합 — 의도 모호.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 명확한 3단 폴백, 자연어 우선의 사용자 지향 설계.
- 위험: 정규식 재컴파일, 빈 토큰 시 라텍스 폴백 누락.
- 개선 권고:
  1. 모듈 로드 시 패턴/치환 정규식 사전 컴파일.
  2. `generateKoreanSpeechSync` 에서 `||` 사용.
  3. `polished` 가 latex 와 "동일" 검사 — 공백/대소문자 정규화 후 비교 권장.
- 재읽기 권고 레벨: 2.
