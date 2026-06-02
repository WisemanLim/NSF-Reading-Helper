# src-tauri/src/core/pack.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: `.nsfpack`(zip) 형식 dictionary 패키지의 import/export. additive insert; 기존 기호와 충돌 시 skip.
- LOC: 402
- 주요 export: `PackManifest`, `PackImportResult`, `import_pack(db, path)`, `export_pack(db, path, name)`
- 의존 모듈: `core::db`, `core::error`, `core::seed`
- 외부 패키지: `zip`, `serde`, `serde_json`
- 부수효과: 파일 I/O (zip read/write), DB 다수 INSERT
- 동시성: sync

---

```rust path=src-tauri/src/core/pack.rs lines=1-20
//! `.nsfpack` — dictionary package format.
//!
//! Layout (zip):
//!   manifest.json   — { name, version, author, created_at, license, counts }
//!   symbols.json    — same schema as `data/seed.json` (domains + symbols)
//!
//! Import is additive: existing symbols (matched by latex or unicode) are
//! skipped; new ones are inserted with `pack:<pack-name>` alias for traceability.
//! Export collects the entire current symbol set into the same format so packs
//! can be shared between installations.

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::core::db::Db;
use crate::core::error::{AppError, AppResult};
```

**분석 / Analysis**

- 문서화 모듈 docstring 이 spec 역할 — 좋은 핸드오프.
- `use rusqlite::params` — `Db::with_conn` 안에서 ad-hoc 쿼리 사용.

```rust path=src-tauri/src/core/pack.rs lines=22-48
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackManifest {
    pub name: String,
    pub version: String,
    #[serde(default)] pub author: Option<String>,
    #[serde(default)] pub license: Option<String>,
    #[serde(default)] pub created_at: Option<i64>,
    #[serde(default)] pub description: Option<String>,
    #[serde(default)] pub symbol_count: i64,
    #[serde(default)] pub domain_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackImportResult {
    pub manifest: PackManifest,
    pub inserted_symbols: i64,
    pub skipped_symbols: i64,
    pub inserted_domains: i64,
}
```

**분석 / Analysis**

- 모든 optional 필드 `#[serde(default)]` — forward-compat.
- `i64` counts → JSON number. 큰 사전(>2^53) 비현실적.

```rust path=src-tauri/src/core/pack.rs lines=50-97
#[derive(Debug, Deserialize)]
struct SeedFile {
    #[serde(default)]
    domains: Vec<SeedDomain>,
    symbols: Vec<SeedSymbol>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SeedDomain { code: String, name_ko: String, #[serde(default)] name_en: Option<String> }

#[derive(Debug, Deserialize, Serialize, Clone)]
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

- **`SeedFile`/`SeedSymbol` 가 `core::seed::SymbolSeedRaw` 와 거의 동일** — *코드 중복*. 둘이 정확히 같지는 않음(notes 필드 누락). 단일 출처로 통합 가능.
- `(String, String)` 튜플 — JSON 작성자 실수 위험 동일.

```rust path=src-tauri/src/core/pack.rs lines=99-170
pub fn import_pack(db: &Db, path: &Path) -> AppResult<PackImportResult> {
    let file = File::open(path)?;
    let mut zip = zip::ZipArchive::new(file)
        .map_err(|e| AppError::Generic(format!("invalid .nsfpack: {e}")))?;

    let manifest: PackManifest = read_json(&mut zip, "manifest.json")?;
    let seed: SeedFile = read_json(&mut zip, "symbols.json")?;

    // 1. Domains — add new only.
    let mut inserted_domains = 0;
    for d in &seed.domains {
        let before: i64 = db_count_domain(db, &d.code)?;
        db.insert_domain(&d.code, &d.name_ko, d.name_en.as_deref())?;
        let after: i64 = db_count_domain(db, &d.code)?;
        if before == 0 && after == 1 {
            inserted_domains += 1;
        }
    }

    // 2. Symbols — skip if a row with the same latex/unicode already exists.
    let mut inserted_symbols = 0;
    let mut skipped_symbols = 0;
    let pack_alias = format!("pack:{}", manifest.name);
    for s in &seed.symbols {
        if symbol_exists(db, s.unicode_char.as_deref(), s.latex_command.as_deref())? {
            skipped_symbols += 1;
            continue;
        }
        let seed_ref = build_seed_ref(s);
        let id = db.insert_symbol(&seed_ref)?;

        if let Some(c) = &s.unicode_char {
            db.insert_alias(id, c, "unicode", "ko")?;
        }
        if let Some(c) = &s.latex_command {
            db.insert_alias(id, c, "latex", "ko")?;
        }
        db.insert_alias(id, &s.canonical_name_ko, "ko-name", "ko")?;
        if let Some(en) = &s.canonical_name_en {
            db.insert_alias(id, en, "en-name", "en")?;
        }
        if let Some(r) = &s.reading_ko {
            db.insert_alias(id, r, "pronunciation", "ko")?;
        }
        for (alias, ty) in &s.aliases {
            db.insert_alias(id, alias, ty, "ko")?;
        }
        // Traceability: tag every imported symbol with the pack name.
        db.insert_alias(id, &pack_alias, "pack-source", "ko")?;

        // Domain links — only if the domain code exists.
        for (code, meaning) in &s.domains {
            if let Some(did) = db_domain_id(db, code)? {
                db.link_domain(id, did, Some(meaning), 1)?;
            }
        }
        for (latex, explain) in &s.examples {
            db.insert_example(id, latex, Some(explain), s.difficulty_level)?;
        }
        inserted_symbols += 1;
    }

    db.rebuild_fts()?;

    Ok(PackImportResult { manifest, inserted_symbols, skipped_symbols, inserted_domains })
}
```

**분석 / Analysis**

- `File::open(path)?` — `AppError::Io` 로 자동 변환.
- `zip::ZipArchive::new` — random-access zip 핸들러.
- `read_json` (헬퍼) 가 zip 내부 파일 두 개를 읽음.
- **Domain 카운트 전후 비교** (before/after) — `insert or ignore` 의 효과를 인서트 여부로 측정. 합리적이지만 두 쿼리 비용. 더 간단하게 `INSERT ... RETURNING` 사용 가능(SQLite 3.35+).
- **Symbol skip 조건**: latex 또는 unicode 가 이미 존재. 두 키가 모두 None 인 비정상 symbol 은 skip 되지 않고 항상 insert(중복 생성 가능). JSON 작성자 책임.
- **`pack_alias = "pack:{name}"`** — 추적용. `alias_type = "pack-source"` 로 별도. 향후 패키지 단위 제거 시 활용 가능.
- **domain 매핑**: code → id 조회 후 link. **N개 symbol × M개 domain 마다 별도 쿼리** — 효율 낮음. 시작 시 한 번 `HashMap<String, i64>` 캐시하는 게 좋음(seed.rs 처럼).
- `db.rebuild_fts()` — import 후 FTS 재구축. 정확하지만 큰 사전엔 비용.
- **트랜잭션 부재** — 중간 실패 시 partial state. 사용자가 다시 import 하면 일부는 skip(중복), 일부는 insert. 이상적이진 않음.
- *짚어볼 점*: `s.aliases` 의 `ty` 가 신뢰되지 않는 사용자 입력 — DB 의 `alias_type` 컬럼에 그대로 저장. 카테고리 오염.

```rust path=src-tauri/src/core/pack.rs lines=171-205
pub fn export_pack(db: &Db, path: &Path, pack_name: &str) -> AppResult<PackManifest> {
    let domains = export_domains(db)?;
    let symbols = export_symbols(db)?;

    let manifest = PackManifest {
        name: pack_name.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        author: None,
        license: Some("MIT".into()),
        created_at: Some(now_ms()),
        description: Some("Exported from NSF Reading Helper".into()),
        symbol_count: symbols.len() as i64,
        domain_count: domains.len() as i64,
    };

    let file = File::create(path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let manifest_json = serde_json::to_vec_pretty(&manifest)?;
    zip.start_file("manifest.json", options)
        .map_err(|e| AppError::Generic(format!("zip: {e}")))?;
    zip.write_all(&manifest_json)?;

    let payload = serde_json::json!({ "domains": domains, "symbols": symbols });
    let payload_json = serde_json::to_vec_pretty(&payload)?;
    zip.start_file("symbols.json", options)
        .map_err(|e| AppError::Generic(format!("zip: {e}")))?;
    zip.write_all(&payload_json)?;

    zip.finish()
        .map_err(|e| AppError::Generic(format!("zip finish: {e}")))?;
    Ok(manifest)
}
```

**분석 / Analysis**

- `env!("CARGO_PKG_VERSION")` — 컴파일 시 Cargo.toml 의 version. 좋은 출처.
- `license: "MIT"` 하드코딩 — 사용자가 export 한 모든 팩이 MIT 라고 표기. *짚어볼 점*: 사용자가 다른 라이선스를 원할 경우 미지원.
- `CompressionMethod::Deflated` — 표준 zip 압축.
- `start_file` + `write_all` 두 번 — `manifest.json` 과 `symbols.json` 별도 엔트리.
- `zip.finish()` — central directory write. 누락 시 zip 손상.

```rust path=src-tauri/src/core/pack.rs lines=207-263
fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn read_json<T: serde::de::DeserializeOwned>(
    zip: &mut zip::ZipArchive<File>,
    name: &str,
) -> AppResult<T> {
    let mut entry = zip
        .by_name(name)
        .map_err(|_| AppError::Generic(format!("missing {name} in .nsfpack")))?;
    let mut buf = String::new();
    entry.read_to_string(&mut buf)?;
    Ok(serde_json::from_str(&buf)?)
}

fn db_count_domain(db: &Db, code: &str) -> AppResult<i64> { ... }

fn db_domain_id(db: &Db, code: &str) -> AppResult<Option<i64>> { ... }

fn symbol_exists(db: &Db, unicode: Option<&str>, latex: Option<&str>) -> AppResult<bool> {
    db.with_conn(|c| {
        let n: i64 = c.query_row(
            "select count(*) from symbols where (latex_command = ?1 and ?1 is not null)
                or (unicode_char = ?2 and ?2 is not null) limit 1",
            params![latex, unicode],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    })
}
```

**분석 / Analysis**

- `now_ms` — `unwrap_or(0)` 시스템 시간 실패는 비현실적. 보호적.
- `read_json` 가 `zip.by_name` 사용 — **zip-slip 안전**: 파일명을 string 으로만 지정하므로 임의 디렉토리 작성 안 됨. 외부 파일 시스템 접근 없음.
- `symbol_exists` 의 SQL: `(latex = ?1 and ?1 is not null) or (unicode = ?2 and ?2 is not null)` — 둘 다 None 이면 false. **결함**: 둘 다 동일 ?1 placeholder 를 두 번 사용 — rusqlite params 가 positional 이라 `?1` 가 어디서나 `latex` 와 같은 값 가리킴. 따라서 NULL 가드는 `latex` 의 NULL 만 검사. unicode 가드도 동일. 정상.

```rust path=src-tauri/src/core/pack.rs lines=265-290
/// Borrow the seed struct fields with `&str` references into the heap-owned
/// strings. Needed because Db::insert_symbol takes a SymbolSeed<'_>.
fn build_seed_ref<'a>(s: &'a SeedSymbol) -> crate::core::seed::SymbolSeed<'a> {
    let has = |k: &str| s.flags.iter().any(|f| f == k);
    crate::core::seed::SymbolSeed {
        unicode_char: s.unicode_char.as_deref(),
        latex_command: s.latex_command.as_deref(),
        mathml_tag: s.mathml_tag.as_deref(),
        canonical_name_ko: &s.canonical_name_ko,
        canonical_name_en: s.canonical_name_en.as_deref(),
        reading_ko: s.reading_ko.as_deref(),
        reading_en: s.reading_en.as_deref(),
        short_meaning_ko: &s.short_meaning_ko,
        short_meaning_en: s.short_meaning_en.as_deref(),
        symbol_type: &s.symbol_type,
        difficulty_level: s.difficulty_level,
        is_operator: has("operator"),
        is_relation: has("relation"),
        is_delimiter: has("delimiter"),
        is_greek: has("greek"),
        is_constant: has("constant"),
        is_function: has("function"),
        notes: None,
    }
}
```

**분석 / Analysis**

- 클로저 `has` — 매 호출마다 flags 선형 스캔. 6번 호출이라 O(6n) — 작은 n 에서 무시 가능.
- `notes: None` — SeedSymbol 에 notes 필드가 없음(SymbolSeedRaw 에는 있음). **데이터 손실**: 사용자가 export 했다 다시 import 하면 notes 가 사라짐.

```rust path=src-tauri/src/core/pack.rs lines=291-402
fn export_domains(db: &Db) -> AppResult<Vec<SeedDomain>> { ... }

fn export_symbols(db: &Db) -> AppResult<Vec<SeedSymbol>> {
    db.with_conn(|c| {
        let mut sym_stmt = c.prepare(
            "select id, unicode_char, latex_command, mathml_tag,
                    canonical_name_ko, canonical_name_en,
                    reading_ko, reading_en,
                    short_meaning_ko, short_meaning_en,
                    symbol_type, difficulty_level,
                    is_operator, is_relation, is_delimiter, is_greek, is_constant, is_function
             from symbols order by id",
        )?;
        let raw: Vec<(i64, SeedSymbol)> = sym_stmt
            .query_map([], |r| {
                ...
                Ok((id, SeedSymbol {
                    ...
                    flags,
                    aliases: vec![],
                    domains: vec![],
                    examples: vec![],
                }))
            })?
            .collect::<Result<_, _>>()?;

        // Attach aliases, domain meanings, examples per symbol.
        let mut symbols = Vec::with_capacity(raw.len());
        for (id, mut s) in raw {
            let aliases: Vec<(String, String)> = c.prepare(...)?.query_map(...)?.collect::<Result<_, _>>()?;
            let domains: Vec<(String, String)> = c.prepare(...)?.query_map(...)?.collect::<Result<_, _>>()?;
            let examples: Vec<(String, String)> = c.prepare(...)?.query_map(...)?.collect::<Result<_, _>>()?;
            s.aliases = aliases;
            s.domains = domains;
            s.examples = examples;
            symbols.push(s);
        }
        Ok(symbols)
    })
}
```

**분석 / Analysis**

- **N+1 쿼리**: 각 symbol 마다 aliases/domains/examples 별도 select. 1000개 symbol → 3000+ 쿼리. 큰 사전에선 느림.
- 단일 락 안에서 수행 (`with_conn`) — 다른 쿼리는 export 동안 대기.
- `aliases.push((alias_text, alias_type))` 순서 — `import_pack` 에서 `for (alias, ty) in s.aliases` 와 일치.
- `domains` 의 `(code, meaning)` — `link_domain` 의 매핑과 일치.

**짚어볼 점 / Watch-outs**
- N+1 쿼리(import 의 domain 조회, export 의 aliases/domains/examples).
- 트랜잭션 부재 (import).
- 라이선스 하드코딩.
- notes 필드 round-trip 손실.
- alias_type 사용자 입력 받음(import) — 카테고리 오염.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 포맷 문서화, additive 정책 명확, traceability alias.
- 위험: N+1, 트랜잭션 부재, notes 손실.
- 개선 권고:
  1. import 시작에 `BEGIN`/`COMMIT` 트랜잭션.
  2. 시작 시 `HashMap<String, i64>` 로 domain code → id 매핑 캐시.
  3. export 의 aliases/domains/examples 를 `JOIN` 한 번 쿼리.
  4. `SeedSymbol` 에 notes 필드 추가(round-trip 보존).
  5. license 를 manifest 인자로 받기.
- 재읽기 권고 레벨: 3 — 트랜잭션·N+1 검토 우선.
