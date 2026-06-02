# web-server/src/seed_pg.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: Postgres 측 seed 설치자. 데스크톱과 동일한 `src-tauri/data/seed.json` 을 `include_str!` 로 임베드.
- LOC: 132
- 주요 export: `install(db)`
- 의존 모듈: `core::error`, `db_pg`
- 외부 패키지: `serde`, `tracing`

---

```rust path=web-server/src/seed_pg.rs lines=1-12
//! Postgres-side seed installer. Reuses the same `data/seed.json` payload as
//! the SQLite installer in `core::seed`, but issues INSERTs via [`PgDb`].

use serde::Deserialize;

use crate::core::error::AppResult;
use crate::db_pg::PgDb;

// Mirrors the sqlite seed.json schema. We share the JSON file via include_str!
// from the desktop tree so curation stays single-sourced.
const SEED_JSON: &str = include_str!("../../src-tauri/data/seed.json");
```

**분석 / Analysis**

- **단일 데이터 출처**: 데스크톱과 동일 JSON 을 `../../src-tauri/data/seed.json` 으로 참조. cargo build 시 web-server 디렉토리 기준 상대 경로.
- *흔한 실수*: 디렉토리 이동 시 깨짐. 그러나 monorepo 내에서 상대 안정.

```rust path=web-server/src/seed_pg.rs lines=14-56
#[derive(Debug, Deserialize)]
struct SeedFile {
    domains: Vec<SeedDomain>,
    symbols: Vec<SeedSymbol>,
}

#[derive(Debug, Deserialize)]
struct SeedDomain { ... }

#[derive(Debug, Deserialize)]
struct SeedSymbol {
    ...
    #[serde(default = "one")] difficulty_level: i64,
    #[serde(default)] flags: Vec<String>,
    #[serde(default)] aliases: Vec<(String, String)>,
    #[serde(default)] domains: Vec<(String, String)>,
    #[serde(default)] examples: Vec<(String, String)>,
}
fn one() -> i64 { 1 }
```

**분석 / Analysis**

- **`core::seed::SymbolSeedRaw` 와 거의 동일** — **코드 중복 3번째** (`core::seed`, `core::pack`, `seed_pg`).
- 단순화 가능: `core::seed::SymbolSeedRaw` 를 pub 로 만들고 두 곳에서 import.

```rust path=web-server/src/seed_pg.rs lines=58-66
fn has(flags: &[String], v: &str) -> bool {
    flags.iter().any(|f| f == v)
}

pub async fn install(db: &PgDb) -> AppResult<()> {
    let parsed: SeedFile = serde_json::from_str(SEED_JSON)?;
    let mut domain_ids: std::collections::HashMap<String, i64> =
        std::collections::HashMap::new();
```

**분석 / Analysis**

- `has` 가 데스크톱 `core::seed::has` 와 동일. 다시 코드 중복.

```rust path=web-server/src/seed_pg.rs lines=70-99
    for d in &parsed.domains {
        let id = db.insert_domain(&d.code, &d.name_ko, d.name_en.as_deref()).await?;
        domain_ids.insert(d.code.clone(), id);
    }

    for s in &parsed.symbols {
        let id = db.insert_symbol(
            s.unicode_char.as_deref(),
            s.latex_command.as_deref(),
            ...
            &s.symbol_type,
            s.difficulty_level as i16,
            has(&s.flags, "operator"),
            has(&s.flags, "relation"),
            has(&s.flags, "delimiter"),
            has(&s.flags, "greek"),
            has(&s.flags, "constant"),
            has(&s.flags, "function"),
        ).await?;
```

**분석 / Analysis**

- 17 인자 호출 — clippy 가드와 매칭.
- `s.difficulty_level as i16` — JSON 의 `i64` → smallint cast. **lossy**: 32768 이상 값은 wrap. 시드는 1-5 정도라 안전하지만 명시적 clamp 가 더 좋음.

```rust path=web-server/src/seed_pg.rs lines=100-132
        if let Some(c) = &s.unicode_char { db.insert_alias(id, c, "unicode", "ko").await?; }
        if let Some(c) = &s.latex_command { db.insert_alias(id, c, "latex", "ko").await?; }
        db.insert_alias(id, &s.canonical_name_ko, "ko-name", "ko").await?;
        if let Some(en) = &s.canonical_name_en { db.insert_alias(id, en, "en-name", "en").await?; }
        if let Some(r) = &s.reading_ko { db.insert_alias(id, r, "pronunciation", "ko").await?; }
        for (alias, ty) in &s.aliases { db.insert_alias(id, alias, ty, "ko").await?; }
        for (code, meaning) in &s.domains {
            if let Some(did) = domain_ids.get(code) {
                db.link_domain(id, *did, Some(meaning), 1).await?;
            }
        }
        for (latex, explain) in &s.examples {
            db.insert_example(id, latex, Some(explain), s.difficulty_level as i16).await?;
        }
    }
    tracing::info!(
        "postgres seed installed: {} domains, {} symbols",
        parsed.domains.len(),
        parsed.symbols.len()
    );
    Ok(())
}
```

**분석 / Analysis**

- 데스크톱 `core::seed::install` 와 거의 동일 로직 — 차이는 `await` 와 i16 cast.
- 데스크톱은 unknown domain 시 `tracing::warn!`, 여기는 silent skip. **로깅 비대칭** → 운영에서 분석 어려움.
- 트랜잭션 부재 — 부분 실패 시 partial state. PG 라면 더 큰 문제(SQLite 는 단일 파일이라 통제 쉬움).

**짚어볼 점 / Watch-outs**
- 🟡 **코드 중복** (`SeedFile`/`SeedSymbol`/`has`/install 전체).
- 🟡 **트랜잭션 부재** — 모든 INSERT 가 자동커밋.
- 🟡 **silent unknown domain** — 데스크톱과 비대칭.
- 🟡 **lossy i64→i16 cast**.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 데스크톱 시드 JSON 재사용, 단순.
- 위험: 중복, 트랜잭션 부재, 로깅 비대칭.
- 개선 권고:
  1. `core::seed::SymbolSeedRaw`/`SeedFile`/`has` 를 pub 으로 공유.
  2. `let tx = client.transaction().await?;` 후 모든 INSERT 마지막에 `tx.commit().await?`.
  3. unknown domain warn 추가.
  4. `i64 → i16` 변환에 try_from + clamp.
- 재읽기 권고 레벨: 2.
