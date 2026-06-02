# src-tauri/src/core/seed.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 컴파일 타임 시드 데이터(`src-tauri/data/seed.json`) 를 첫 부팅 시 SQLite로 적재.
- LOC: 160
- 주요 export: `SymbolSeed<'a>`, `install(db)`
- 의존 모듈: `core::db`, `core::error`
- 외부 패키지: `serde`, `serde_json`, `tracing`
- 부수효과: DB INSERT 다수, 로깅

---

```rust path=src-tauri/src/core/seed.rs lines=1-13
//! Seed dictionary — installed on first DB boot when `symbols` is empty.
//! Source data lives in `src-tauri/data/seed.json` and is embedded at compile
//! time via `include_str!`. Edit the JSON to expand the dictionary; no code
//! changes needed.

use std::collections::HashMap;

use serde::Deserialize;

use crate::core::db::Db;
use crate::core::error::AppResult;

const SEED_JSON: &str = include_str!("../../data/seed.json");
```

**분석 / Analysis**

- `include_str!` — **컴파일 시 정적 임베드**. 바이너리 크기에 직접 포함. 시드 크기가 커지면 바이너리도 커짐.
- *왜 임베드*: 사용자가 별도 파일을 갖추지 않아도 작동. 데스크톱 첫 부팅에 즉시 사전 사용 가능.

```rust path=src-tauri/src/core/seed.rs lines=15-64
#[derive(Debug, Deserialize)]
struct SeedFile {
    domains: Vec<DomainSeed>,
    symbols: Vec<SymbolSeedRaw>,
}

#[derive(Debug, Deserialize)]
struct DomainSeed {
    code: String,
    name_ko: String,
    #[serde(default)]
    name_en: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SymbolSeedRaw {
    #[serde(default)]
    unicode_char: Option<String>,
    #[serde(default)]
    latex_command: Option<String>,
    ...
    #[serde(default = "default_difficulty")]
    difficulty_level: i64,
    #[serde(default)]
    flags: Vec<String>,
    #[serde(default)]
    notes: Option<String>,
    #[serde(default)]
    aliases: Vec<(String, String)>,
    #[serde(default)]
    domains: Vec<(String, String)>,
    #[serde(default)]
    examples: Vec<(String, String)>,
}

fn default_difficulty() -> i64 {
    1
}
```

**분석 / Analysis**

- 모든 옵셔널 필드에 `#[serde(default)]` — JSON 작성자가 누락해도 안전. 좋은 forward-compat.
- `Vec<(String, String)>` — JSON 배열의 2-tuple. seed.json 작성자가 `["alias", "type"]` 형태로 적음. *주의*: 위치 의존 — `aliases` 의 첫 요소는 alias text, 두 번째는 alias_type. 작성자가 순서 바꾸면 silent 오염.
- `default_difficulty` — 명시적 함수가 단순 리터럴 1 만 반환. 인라인 또는 `#[serde(default)]` + i64 의 Default 인 0 이 아니라 1 이 의도라 함수가 필요.

```rust path=src-tauri/src/core/seed.rs lines=66-86
#[derive(Debug, Clone)]
pub struct SymbolSeed<'a> {
    pub unicode_char: Option<&'a str>,
    pub latex_command: Option<&'a str>,
    ...
    pub notes: Option<&'a str>,
}

fn has(flags: &[String], v: &str) -> bool {
    flags.iter().any(|f| f == v)
}
```

**분석 / Analysis**

- `SymbolSeed<'a>` — heap-owned 데이터(`SymbolSeedRaw`) 의 borrow 표현. `Db::insert_symbol` 의 시그니처가 `&SymbolSeed<'_>` 이라 borrow 만 전달.
- *왜 별도 struct*: SymbolSeedRaw 에는 `aliases/domains/examples` 등 fancy 필드까지 있고 insert_symbol 은 평탄한 컬럼들만 필요. 별 type 으로 정리.
- `pack.rs` 도 같은 `SymbolSeed<'a>` 를 재사용 — 좋은 단일 출처.

```rust path=src-tauri/src/core/seed.rs lines=92-103
pub fn install(db: &Db) -> AppResult<()> {
    let parsed: SeedFile = serde_json::from_str(SEED_JSON).map_err(|e| {
        tracing::error!("seed json parse failed: {e}");
        e
    })?;

    let mut domain_ids: HashMap<String, i64> = HashMap::new();
    for d in &parsed.domains {
        let id = db.insert_domain(&d.code, &d.name_ko, d.name_en.as_deref())?;
        domain_ids.insert(d.code.clone(), id);
    }
```

**분석 / Analysis**

- `map_err(|e| { log; e })` — 에러 발생 시 로그만 추가하고 그대로 전파. 디버깅 친화.
- domain insert → id map. 이후 symbol_domain_map 링크에서 사용.

```rust path=src-tauri/src/core/seed.rs lines=104-152
    for raw in &parsed.symbols {
        let seed = SymbolSeed {
            unicode_char: raw.unicode_char.as_deref(),
            ...
            is_operator: has(&raw.flags, "operator"),
            is_relation: has(&raw.flags, "relation"),
            is_delimiter: has(&raw.flags, "delimiter"),
            is_greek: has(&raw.flags, "greek"),
            is_constant: has(&raw.flags, "constant"),
            is_function: has(&raw.flags, "function"),
            notes: raw.notes.as_deref(),
        };
        let id = db.insert_symbol(&seed)?;
        if let Some(c) = seed.unicode_char {
            db.insert_alias(id, c, "unicode", "ko")?;
        }
        if let Some(c) = seed.latex_command {
            db.insert_alias(id, c, "latex", "ko")?;
        }
        db.insert_alias(id, seed.canonical_name_ko, "ko-name", "ko")?;
        if let Some(en) = seed.canonical_name_en {
            db.insert_alias(id, en, "en-name", "en")?;
        }
        if let Some(r) = seed.reading_ko {
            db.insert_alias(id, r, "pronunciation", "ko")?;
        }
        for (alias, ty) in &raw.aliases {
            db.insert_alias(id, alias, ty, "ko")?;
        }
        for (domain_code, meaning) in &raw.domains {
            if let Some(did) = domain_ids.get(domain_code) {
                db.link_domain(id, *did, Some(meaning), 1)?;
            } else {
                tracing::warn!("seed: unknown domain '{}'", domain_code);
            }
        }
        for (latex, explain) in &raw.examples {
            db.insert_example(id, latex, Some(explain), seed.difficulty_level)?;
        }
    }
```

**분석 / Analysis**

- 각 symbol 마다:
  1. `insert_symbol` (자동 ID).
  2. unicode/latex 별칭 자동 등록.
  3. canonical_name_ko / _en / reading_ko 별칭 등록.
  4. raw.aliases 의 (text, type) 등록 — 사용자 정의 type.
  5. domain_code → domain_id 매핑 시 link, 매핑 실패 시 warn.
  6. 예시 등록.
- **각 symbol 가 5-10개 별칭 등록** — 1000개 symbol 이면 5000-10000 INSERT. Mutex 락이 매 호출마다 잡힘 → 락 경합은 단일 사용자라 무시 가능, 그러나 트랜잭션 부재로 atomicity 없음.
- *흔한 실수*: 중간에 실패하면 partial seed 상태. `seed_if_empty` 가 다시 호출되어도 `count(*)>0` 이라 더 이상 시드하지 않음 → 영구 partial state. **트랜잭션으로 감싸는 게 안전**.

```rust path=src-tauri/src/core/seed.rs lines=154-160
    tracing::info!(
        "seed installed: {} domains, {} symbols",
        parsed.domains.len(),
        parsed.symbols.len()
    );
    Ok(())
}
```

**분석 / Analysis**

- 끝마침 로그.

**짚어볼 점 / Watch-outs**
- 트랜잭션 부재 — partial seed 위험.
- 각 INSERT 마다 Mutex 락 — 빌드 시간이 길어질 수 있으나 한 번 비용.
- `Vec<(String, String)>` 위치 의존 — JSON 작성 실수 silent.
- domain 누락은 warn 후 진행 — 매핑 실패한 symbol-domain 링크가 silently 누락. 사용자가 SymbolPanel 에서 도메인 의미를 못 봄.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: include_str! 정적 임베드, forward-compat, 명확한 로그.
- 위험: 트랜잭션 부재, partial-seed 영구화 위험.
- 개선 권고:
  1. `with_conn` + `conn.transaction()` 으로 전체를 한 트랜잭션.
  2. 시드 schema 자체 validator(JSON Schema).
  3. domain 매핑 실패 시 strict 모드 옵션.
- 재읽기 권고 레벨: 2.
