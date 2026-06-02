# src/types.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 프런트엔드 전체에서 사용하는 DTO 타입 정의. 백엔드 `src-tauri/src/core/models.rs` 의 `#[serde(rename_all="camelCase")]` 직렬화와 1:1 매칭.
- 진입점에서의 거리: 다수(2-3홉) — 거의 모든 페이지/스토어가 import.
- LOC: 129
- 주요 export: `InputFormat`, `SymbolType`, `Domain`, `SymbolRow`, `SymbolHit`, `SymbolExample`, `SymbolDetail`, `AnalyzeRequest`, `AnalyzeToken`, `AnalyzeResult`, `HistoryItem`, `Note`, `Settings`, `SymbolDomainMeaning`
- 의존 모듈: 없음 (type-only)
- 외부 패키지: 없음
- 부수효과: 없음
- 동시성: N/A
- 테스트: 없음

---

```ts path=src/types.ts lines=1-16
export type InputFormat = "latex" | "mathml" | "asciimath" | "unicode";
export type SymbolType =
  | "operator"
  | "relation"
  | "delimiter"
  | "greek-letter"
  | "function"
  | "arrow"
  | "set-symbol"
  | "logic-symbol"
  | "chemistry-notation"
  | "unit-symbol"
  | "biology-notation"
  | "constant"
  | "other";
```

**분석 / Analysis**

- 두 개의 string-literal union — TypeScript에서 가장 가벼운 폐쇄형 enum 대용. *왜 enum 이 아닌가?*: `Settings` 등이 string 으로 직렬화되므로 enum 의 reverse-map/숫자값이 불필요. union 이 직렬화 일관성 면에서 유리.
- *짚어볼 점*: 백엔드 Rust 측 `SymbolRow.symbol_type` 은 `String` (자유 텍스트). 즉 union 의 엄격성은 *컴파일 타임만* 의미가 있고, 백엔드가 새 타입을 보내면 TS는 그것을 `"other"` 분기에서 받지 못함 → 단순 `string` 비교 누락 가능.
- *흔한 실수*: 신규 종류 추가 시 두 곳(TS union + Rust 시드 데이터) 모두 갱신 필요. 매뉴얼 동기화.

```ts path=src/types.ts lines=17-23
export interface Domain {
  id: number;
  code: string;
  nameKo: string;
  nameEn?: string | null;
  description?: string | null;
}
```

**분석 / Analysis**

- `id: number` — JS 의 `number` 는 64-bit float. 백엔드 `i64` 와 안전한 범위(2^53)는 일반적인 사용에선 문제 없음. *주의*: 시드가 매우 커지면 정밀도 손실 이론적 가능(현실엔 거의 없음).
- 필드 명명: backend `name_ko` → frontend `nameKo` — `#[serde(rename_all="camelCase")]` 로 자동 변환.
- 옵셔널이 `T | null | undefined` 의 효과를 가지도록 `?: T | null` 패턴 — JSON `null` 과 omitted 모두 허용.

```ts path=src/types.ts lines=25-30
export interface SymbolDomainMeaning {
  code: string;
  nameKo: string;
  meaningKo?: string | null;
  exampleUsage?: string | null;
}
```

**분석 / Analysis**

- 백엔드 `SymbolDomainMeaning { code, name_ko, meaning_ko, example_usage }` 와 매칭.

```ts path=src/types.ts lines=32-52
export interface SymbolRow {
  id: number;
  unicodeChar?: string | null;
  latexCommand?: string | null;
  mathmlTag?: string | null;
  canonicalNameKo: string;
  canonicalNameEn?: string | null;
  readingKo?: string | null;
  readingEn?: string | null;
  shortMeaningKo: string;
  shortMeaningEn?: string | null;
  symbolType: SymbolType;
  difficultyLevel: number;
  isOperator: boolean;
  isRelation: boolean;
  isDelimiter: boolean;
  isGreek: boolean;
  isConstant: boolean;
  isFunction: boolean;
  notes?: string | null;
}
```

**분석 / Analysis**

- `SymbolRow` 는 백엔드 DB row 의 평탄 표현. flag 들이 `is_*` 패턴으로 분리 — 백엔드 SQLite 컬럼 그대로.
- *설계 결정*: enum 한 종류 대신 boolean 다섯 개 — `is_operator` 와 `is_relation` 이 동시에 true 가능(예: `=` 은 operator+relation). 즉 다중 라벨링 모델.
- `symbolType: SymbolType` — 단일 union. boolean 들과 의미적 중첩이 있어 정합 유지가 시드 데이터에 의존.

```ts path=src/types.ts lines=54-63
export interface SymbolHit {
  id: number;
  unicodeChar?: string | null;
  latexCommand?: string | null;
  canonicalNameKo: string;
  readingKo?: string | null;
  shortMeaningKo: string;
  symbolType: SymbolType;
  matchedAlias?: string | null;
}
```

**분석 / Analysis**

- 검색 결과용 경량 DTO. `SymbolRow` 의 부분집합 + `matchedAlias` 추가. 백엔드의 `row_to_hit` 와 매칭.

```ts path=src/types.ts lines=65-73
export interface SymbolExample {
  id: number;
  symbolId: number;
  domainCode?: string | null;
  exampleLatex: string;
  exampleReadingKo?: string | null;
  exampleExplanationKo?: string | null;
  difficultyLevel: number;
}
```

**분석 / Analysis**

- 백엔드 `SymbolExample` 미러.

```ts path=src/types.ts lines=75-80
export interface SymbolDetail extends SymbolRow {
  aliases: string[];
  domains: SymbolDomainMeaning[];
  examples: SymbolExample[];
  similar: SymbolHit[];
}
```

**분석 / Analysis**

- 백엔드 `SymbolDetail` 가 `#[serde(flatten)] pub row: SymbolRow` 로 표현되어 JSON 평탄화 → TS 측에선 `extends SymbolRow` 로 정확히 모델링. 좋은 매핑.

```ts path=src/types.ts lines=82-104
export interface AnalyzeRequest {
  rawInput: string;
  format: InputFormat;
  locale?: "ko" | "en";
}

export interface AnalyzeToken {
  index: number;
  tokenText: string;
  role?: string | null;
  symbolId?: number | null;
  nameKo?: string | null;
  readingKo?: string | null;
  meaningKo?: string | null;
}

export interface AnalyzeResult {
  expressionId?: number | null;
  normalizedLatex?: string | null;
  normalizedMathml?: string | null;
  speech: { ko: string; en?: string };
  tokens: AnalyzeToken[];
}
```

**분석 / Analysis**

- `AnalyzeRequest` 는 Rust 의 `AnalyzeRequest { raw_input, format: String, locale }` 와 같음.
- 백엔드의 `format: String` 은 자유 텍스트, TS의 `InputFormat` 은 union — 백엔드가 다른 값을 보낼 일은 없으나 *명세상* TS가 더 엄격.
- `speech: { ko; en? }` — 익명 객체 인라인. *왜*: 백엔드 `AnalyzeSpeech` 가 단순 두 필드라 별도 export interface 의 필요성 낮음. 트레이드오프: 재사용성↓, 가독성↑.

```ts path=src/types.ts lines=106-129
export interface HistoryItem { ... }
export interface Note { ... }
export interface Settings {
  speechRule: "clearspeak" | "mathspeak";
  speechLocale: "ko" | "en";
  ttsRate: number;
  ttsVoice?: string | null;
  uiLocale: "ko" | "en";
  domainsEnabled: string[];
}
```

**분석 / Analysis**

- `Settings.speechRule` 은 union. 백엔드 `Settings.speech_rule: String` 은 string. TS-side 엄격성은 `SettingsPage` 의 `<select>` 가 두 값만 노출하므로 실용적 안전.
- `domainsEnabled: string[]` — 비활성/활성 도메인 토글. 현재 UI에는 노출되지 않음(`SettingsPage.tsx` 에 해당 컨트롤 없음) → *deferred feature*.

**짚어볼 점 / Watch-outs**
- 백엔드 enum/string 미러링은 수작업 — `openapi-typescript` 또는 zod/ts-rs 같은 자동 동기화 없음. 신규 필드 추가 시 두 곳 갱신 누락 위험.
- `id`, `expressionId`, `symbolId` 가 모두 `number` — i64 → JS number 변환. 일반 사용엔 안전.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 백엔드 DTO와 형상이 깔끔히 미러, optional 패턴 일관.
- 위험: 자동 동기화 도구 부재, union 의 엄격성은 명목적.
- 개선 권고:
  1. ts-rs 또는 `serde-typescript` 도입해 Rust → TS 자동 생성.
  2. `unifiedSettings` 등으로 union 들을 한 곳에 모아 SettingsPage 의 `<option>` 과 동기.
  3. `SymbolType` 도 백엔드에서 enum 으로 좁히고 직렬화는 kebab-case 유지.
- 재읽기 권고 레벨: 1 — 데이터 모델 한눈에 보기 좋음.
