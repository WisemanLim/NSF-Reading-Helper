# src-tauri/src/core/analyzer.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 입력 문자열에서 LaTeX 명령(`\foo`) + 유니코드 수학 문자(∫, α …) 추출, 각 토큰을 DB 사전과 조인.
- LOC: 83
- 주요 export: `analyze(db, req)`
- 의존 모듈: `core::{db, error, models}`
- 외부 패키지: `regex`, `once_cell`

---

```rust path=src-tauri/src/core/analyzer.rs lines=1-17
//! Lightweight LaTeX tokenizer + dictionary join. SRE-based speech generation
//! happens on the frontend (JavaScript module) — the backend just supplies
//! tokens with their resolved symbol metadata. Frontend overlays this with
//! SRE's natural-language speech string.

use once_cell::sync::Lazy;
use regex::Regex;

use crate::core::db::Db;
use crate::core::error::AppResult;
use crate::core::models::{
    AnalyzeRequest, AnalyzeResult, AnalyzeSpeech, AnalyzeToken, SymbolRow,
};

static LATEX_CMD: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\([A-Za-z]+)").unwrap());
static UNICODE_MATH: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[\u{2200}-\u{22FF}\u{2190}-\u{21FF}\u{0391}-\u{03C9}∑∫∮∂∇∞°]").unwrap());
```

**분석 / Analysis**

- `once_cell::sync::Lazy` — 첫 접근 시 한 번 컴파일, 이후 재사용. 다중 스레드 안전.
- `LATEX_CMD = r"\\([A-Za-z]+)"` — `\sin`, `\frac` 등. **숫자 명령(`\1`)이나 기호 단명령(`\,`)은 매칭 안 함** — 명령형 명령만 추출.
- `UNICODE_MATH` 범위:
  - `\u{2200}-\u{22FF}`: Mathematical Operators 블록(∀, ∃, ∂, ∇, ∫ 등).
  - `\u{2190}-\u{21FF}`: Arrows 블록.
  - `\u{0391}-\u{03C9}`: 그리스 대/소문자 (Α-ω).
  - 명시적 추가 문자열: `∑∫∮∂∇∞°` (이미 범위에 포함된 문자 일부 중복 — 안전).
- **누락 영역**: Supplemental Math Operators (`\u{2A00}-\u{2AFF}`), Letterlike Symbols (`\u{2100}-\u{214F}`, ℂ, ℕ, ℝ 등). 사용자가 ℝ 입력 시 토큰화 안 됨.

```rust path=src-tauri/src/core/analyzer.rs lines=19-44
pub fn analyze(db: &Db, req: &AnalyzeRequest) -> AppResult<AnalyzeResult> {
    let format = req.format.as_str();
    let raw = req.raw_input.trim();

    let normalized_latex = match format {
        "latex" => Some(raw.to_string()),
        "unicode" => Some(raw.to_string()),
        _ => Some(raw.to_string()),
    };

    let mut tokens = Vec::new();
    let mut idx: i64 = 0;

    for m in LATEX_CMD.captures_iter(raw) {
        let cmd = format!("\\{}", &m[1]);
        let resolved = db.lookup_by_latex(&cmd)?;
        tokens.push(token(idx, cmd, "command", resolved));
        idx += 1;
    }

    for m in UNICODE_MATH.find_iter(raw) {
        let ch = m.as_str().to_string();
        let resolved = db.lookup_by_unicode(&ch)?;
        tokens.push(token(idx, ch, "unicode", resolved));
        idx += 1;
    }
```

**분석 / Analysis**

- `format` match 가 세 분기 모두 `raw.to_string()` 반환 — **무의미한 match**. 향후 mathml→latex 변환을 위한 자리. 현재는 `Some(raw)` 한 줄로 단순화 가능.
- `captures_iter(raw)` — 모든 LaTeX 명령 매칭, `m[1]` 가 캡처 그룹(명령 이름).
- `format!("\\{}", &m[1])` — 다시 `\` prefix 붙여 DB key 로 사용. **흔한 실수**: backslash 이스케이프 `\\` 는 Rust string literal 안에서 단일 `\`. 정확.
- `db.lookup_by_latex(&cmd)?` — Mutex 락 + prepared statement. N개 명령마다 1번 락 → **N+1 쿼리 패턴**. 한 SQL 로 `IN (?,?,?,...)` 호출이 더 빠름. 데스크톱 작은 입력에선 무시 가능.
- LaTeX 명령 → unicode 문자 순서로 토큰 인덱스 부여. **위치 정보 손실**: 원문에서 `\alpha ∑` 의 인덱스 0,1 은 OK 지만 `∑ \alpha` 는 1,0 이어야 정확. 현 구현은 두 정규식이 별도로 fullscan 하므로 **토큰의 시간적 순서가 원문 위치 순서와 다름**.

```rust path=src-tauri/src/core/analyzer.rs lines=46-66
    let speech_seed = if tokens.is_empty() {
        raw.to_string()
    } else {
        tokens
            .iter()
            .map(|t| t.name_ko.clone().unwrap_or_else(|| t.token_text.clone()))
            .collect::<Vec<_>>()
            .join(", ")
    };

    Ok(AnalyzeResult {
        expression_id: None,
        normalized_latex,
        normalized_mathml: None,
        speech: AnalyzeSpeech {
            ko: speech_seed,
            en: None,
        },
        tokens,
    })
}
```

**분석 / Analysis**

- `speech_seed` — 토큰 한국어 이름을 콤마로 연결. 매우 단순한 음성. 프런트엔드 `koSpeak` 가 그 위에 자연어 SRE 폴리시.
- `expression_id: None` — 호출자(`commands::analyze`)가 이후 `db.save_history` 의 결과로 set 한다고 가정하지만 **실제로는 그 후 처리 없이 client 로 그대로 반환**. JS 측 `expressionId` 가 항상 null/undefined.

```rust path=src-tauri/src/core/analyzer.rs lines=68-83
fn token(
    index: i64,
    text: String,
    role: &str,
    sym: Option<SymbolRow>,
) -> AnalyzeToken {
    AnalyzeToken {
        index,
        token_text: text,
        role: Some(role.into()),
        symbol_id: sym.as_ref().map(|s| s.id),
        name_ko: sym.as_ref().map(|s| s.canonical_name_ko.clone()),
        reading_ko: sym.as_ref().and_then(|s| s.reading_ko.clone()),
        meaning_ko: sym.as_ref().map(|s| s.short_meaning_ko.clone()),
    }
}
```

**분석 / Analysis**

- helper — `SymbolRow` 가 Some 이면 4개 필드 채움, None 이면 None 유지.
- `.clone()` 3번 — `SymbolRow` 가 `Clone` 이지만 필드별 clone 이 더 가벼움. 일반적인 패턴.

**짚어볼 점 / Watch-outs**
- 토큰 위치 정보 손실 (`\alpha ∑` 와 `∑ \alpha` 가 동일 결과).
- N+1 DB 쿼리.
- `normalized_mathml: None` — `format == "mathml"` 이어도 normalization 없음.
- `expression_id: None` 가 client 로 흘러감 (commands::analyze 에서 save_history 결과를 set 하지 않음).

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 단순, regex lazy, helper 분리.
- 위험: 토큰 순서, N+1, expression_id 누락.
- 개선 권고:
  1. 두 정규식을 alternation 으로 합쳐 한 번에 스캔(인덱스 보존).
  2. `lookup_many(commands, unicodes)` 배치 쿼리 도입.
  3. `commands::analyze` 가 `save_history` 결과 id 로 `r.expression_id` 채우기.
  4. Letterlike Symbols/Supplemental Math 범위 추가.
- 재읽기 권고 레벨: 2.
