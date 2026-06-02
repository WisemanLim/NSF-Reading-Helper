# src-tauri/src/core/models.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 백엔드 Serde DTO. JS-side(`src/types.ts`)와 1:1 매핑.
- LOC: 178
- 주요 export: `Domain`, `SymbolDomainMeaning`, `SymbolRow`, `SymbolHit`, `SymbolExample`, `SymbolDetail`, `AnalyzeRequest`, `AnalyzeToken`, `AnalyzeSpeech`, `AnalyzeResult`, `HistoryItem`, `Note`, `SearchOptions`, `Settings`, `TtsOptions`
- 외부 패키지: `serde`

---

```rust path=src-tauri/src/core/models.rs lines=1-12
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Domain {
    pub id: i64,
    pub code: String,
    pub name_ko: String,
    pub name_en: Option<String>,
    pub description: Option<String>,
}
```

**분석 / Analysis**

- `#[serde(rename_all = "camelCase")]` — JSON 직렬화 시 `name_ko` → `nameKo`. JS 가 자연스럽게 사용.
- 모든 구조체에 `Debug + Clone` — 로깅·복사 가능.
- `Option<String>` 은 JSON null 또는 omit 양쪽 허용.

```rust path=src-tauri/src/core/models.rs lines=13-44
#[derive(...)] pub struct SymbolDomainMeaning { ... }

#[derive(...)] pub struct SymbolRow {
    pub id: i64,
    pub unicode_char: Option<String>,
    pub latex_command: Option<String>,
    pub mathml_tag: Option<String>,
    pub canonical_name_ko: String,
    pub canonical_name_en: Option<String>,
    pub reading_ko: Option<String>,
    pub reading_en: Option<String>,
    pub short_meaning_ko: String,
    pub short_meaning_en: Option<String>,
    pub symbol_type: String,
    pub difficulty_level: i64,
    pub is_operator: bool,
    pub is_relation: bool,
    pub is_delimiter: bool,
    pub is_greek: bool,
    pub is_constant: bool,
    pub is_function: bool,
    pub notes: Option<String>,
}
```

**분석 / Analysis**

- `symbol_type: String` — TS-side 가 `SymbolType` union 으로 좁혔지만 Rust 측은 자유 문자열. *왜*: SQLite TEXT 컬럼 그대로 매핑. 시드 데이터의 잘못된 값을 받을 가능성을 받아들임.
- 6개 boolean — 다중 라벨링 가능(예: `is_operator + is_relation` 동시 true).
- `difficulty_level: i64` — TS side `number`. JSON 직렬화 정확.

```rust path=src-tauri/src/core/models.rs lines=46-58
#[derive(...)]
pub struct SymbolHit {
    pub id: i64,
    pub unicode_char: Option<String>,
    pub latex_command: Option<String>,
    pub canonical_name_ko: String,
    pub reading_ko: Option<String>,
    pub short_meaning_ko: String,
    pub symbol_type: String,
    pub matched_alias: Option<String>,
}
```

**분석 / Analysis**

- 검색용 경량 DTO. `matched_alias` 는 어떤 alias 가 매치했는지 표시할 자리이지만 현재 `Db::symbol_search` 가 `null` 만 보냄 (`row_to_hit` 의 마지막 컬럼이 항상 `null`). *deferred enhancement*.

```rust path=src-tauri/src/core/models.rs lines=60-80
#[derive(...)] pub struct SymbolExample { ... }
#[derive(...)] pub struct SymbolDetail {
    #[serde(flatten)] pub row: SymbolRow,
    pub aliases: Vec<String>,
    pub domains: Vec<SymbolDomainMeaning>,
    pub examples: Vec<SymbolExample>,
    pub similar: Vec<SymbolHit>,
}
```

**분석 / Analysis**

- `#[serde(flatten)] pub row: SymbolRow` — `row` 의 필드를 부모 객체로 평탄화. JS 측 `SymbolDetail extends SymbolRow` 와 완벽히 매핑. **좋은 idiom**.

```rust path=src-tauri/src/core/models.rs lines=82-118
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeRequest {
    pub raw_input: String,
    pub format: String,
    pub locale: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeToken { ... }

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeSpeech {
    pub ko: String,
    pub en: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeResult {
    pub expression_id: Option<i64>,
    pub normalized_latex: Option<String>,
    pub normalized_mathml: Option<String>,
    pub speech: AnalyzeSpeech,
    pub tokens: Vec<AnalyzeToken>,
}
```

**분석 / Analysis**

- `AnalyzeRequest` 는 `Deserialize` 만, `AnalyzeResult`/`Token`/`Speech` 는 `Serialize` 만 — 단방향 사용에 맞춰 trait 최소화. **의도적 최소화**.
- `format: String` — TS-side union 의 4값만 의미가 있지만 강제 미적용.

```rust path=src-tauri/src/core/models.rs lines=119-128
#[derive(...)] pub struct HistoryItem {
    pub expression_id: i64,
    pub raw_input: String,
    pub format: String,
    pub speech_ko: Option<String>,
    pub created_at: i64,
}
```

**분석 / Analysis**

- `created_at: i64` — ms epoch. JSON number 로 직렬화되며 JS 가 Date 변환 책임.

```rust path=src-tauri/src/core/models.rs lines=129-138
#[derive(...)] pub struct Note {
    pub id: Option<i64>,
    pub expression_id: Option<i64>,
    pub symbol_id: Option<i64>,
    pub body_md: String,
    #[serde(default)]
    pub tags: Vec<String>,
}
```

**분석 / Analysis**

- `#[serde(default)]` on `tags` — JSON 에서 누락이면 빈 vec. 좋은 forward-compat.
- `id: Option<i64>` — insert 시 None, update 시 Some(id).

```rust path=src-tauri/src/core/models.rs lines=140-179
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchOptions { ... }

#[derive(...)] pub struct Settings {
    pub speech_rule: String,
    pub speech_locale: String,
    pub tts_rate: f64,
    pub tts_voice: Option<String>,
    pub ui_locale: String,
    #[serde(default)]
    pub domains_enabled: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            speech_rule: "clearspeak".into(),
            speech_locale: "ko".into(),
            tts_rate: 1.0,
            tts_voice: None,
            ui_locale: "ko".into(),
            domains_enabled: vec![],
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsOptions {
    pub rate: f64,
    pub lang: String,
}
```

**분석 / Analysis**

- `Settings::default()` 값이 JS-side `DEFAULT_SETTINGS` 와 정확히 일치. 한 곳을 바꾸면 다른 쪽도 갱신 필요.
- `domains_enabled` 만 `#[serde(default)]` — 다른 필드 누락 시 deserialize 실패. `settings::load` 의 `unwrap_or_default` 가 전체 fallback.

**짚어볼 점 / Watch-outs**
- 단일 필드 누락이 전체 reset 으로 이어지는 점(`settings::load` 와 함께 봤을 때).
- `symbol_type`, `format` 가 자유 문자열 — enum 으로 좁히면 DB seed 단계에서 즉시 잘못된 값 감지 가능.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: camelCase rename, flatten, Default impl.
- 위험: enum 강제력 부재, 모든 Settings 필드 누락 시 reset.
- 개선 권고:
  1. `symbol_type: SymbolType (enum)` 으로 좁히고 seed.json 파싱 시 검증.
  2. 모든 `Settings` 필드에 `#[serde(default = "...")]`.
- 재읽기 권고 레벨: 2.
