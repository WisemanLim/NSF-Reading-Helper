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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackManifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub created_at: Option<i64>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub symbol_count: i64,
    #[serde(default)]
    pub domain_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackImportResult {
    pub manifest: PackManifest,
    pub inserted_symbols: i64,
    pub skipped_symbols: i64,
    pub inserted_domains: i64,
}

#[derive(Debug, Deserialize)]
struct SeedFile {
    #[serde(default)]
    domains: Vec<SeedDomain>,
    symbols: Vec<SeedSymbol>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SeedDomain {
    code: String,
    name_ko: String,
    #[serde(default)]
    name_en: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SeedSymbol {
    #[serde(default)]
    unicode_char: Option<String>,
    #[serde(default)]
    latex_command: Option<String>,
    #[serde(default)]
    mathml_tag: Option<String>,
    canonical_name_ko: String,
    #[serde(default)]
    canonical_name_en: Option<String>,
    #[serde(default)]
    reading_ko: Option<String>,
    #[serde(default)]
    reading_en: Option<String>,
    short_meaning_ko: String,
    #[serde(default)]
    short_meaning_en: Option<String>,
    symbol_type: String,
    #[serde(default = "one")]
    difficulty_level: i64,
    #[serde(default)]
    flags: Vec<String>,
    #[serde(default)]
    aliases: Vec<(String, String)>,
    #[serde(default)]
    domains: Vec<(String, String)>,
    #[serde(default)]
    examples: Vec<(String, String)>,
}
fn one() -> i64 {
    1
}

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

    Ok(PackImportResult {
        manifest,
        inserted_symbols,
        skipped_symbols,
        inserted_domains,
    })
}

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

fn db_count_domain(db: &Db, code: &str) -> AppResult<i64> {
    db.with_conn(|c| {
        let n: i64 = c.query_row(
            "select count(*) from domains where code = ?",
            params![code],
            |r| r.get(0),
        )?;
        Ok(n)
    })
}

fn db_domain_id(db: &Db, code: &str) -> AppResult<Option<i64>> {
    db.with_conn(|c| {
        let id = c
            .query_row("select id from domains where code = ?", params![code], |r| {
                r.get::<_, i64>(0)
            })
            .ok();
        Ok(id)
    })
}

fn symbol_exists(
    db: &Db,
    unicode: Option<&str>,
    latex: Option<&str>,
) -> AppResult<bool> {
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

fn export_domains(db: &Db) -> AppResult<Vec<SeedDomain>> {
    db.with_conn(|c| {
        let mut stmt =
            c.prepare("select code, name_ko, name_en from domains order by id")?;
        let rows = stmt
            .query_map([], |r| {
                Ok(SeedDomain {
                    code: r.get(0)?,
                    name_ko: r.get(1)?,
                    name_en: r.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    })
}

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
                let id: i64 = r.get(0)?;
                let mut flags = Vec::new();
                let is_op: bool = r.get(12)?;
                let is_rel: bool = r.get(13)?;
                let is_del: bool = r.get(14)?;
                let is_grk: bool = r.get(15)?;
                let is_con: bool = r.get(16)?;
                let is_fun: bool = r.get(17)?;
                if is_op {
                    flags.push("operator".into());
                }
                if is_rel {
                    flags.push("relation".into());
                }
                if is_del {
                    flags.push("delimiter".into());
                }
                if is_grk {
                    flags.push("greek".into());
                }
                if is_con {
                    flags.push("constant".into());
                }
                if is_fun {
                    flags.push("function".into());
                }
                Ok((
                    id,
                    SeedSymbol {
                        unicode_char: r.get(1)?,
                        latex_command: r.get(2)?,
                        mathml_tag: r.get(3)?,
                        canonical_name_ko: r.get(4)?,
                        canonical_name_en: r.get(5)?,
                        reading_ko: r.get(6)?,
                        reading_en: r.get(7)?,
                        short_meaning_ko: r.get(8)?,
                        short_meaning_en: r.get(9)?,
                        symbol_type: r.get(10)?,
                        difficulty_level: r.get(11)?,
                        flags,
                        aliases: vec![],
                        domains: vec![],
                        examples: vec![],
                    },
                ))
            })?
            .collect::<Result<_, _>>()?;

        // Attach aliases, domain meanings, examples per symbol.
        let mut symbols = Vec::with_capacity(raw.len());
        for (id, mut s) in raw {
            let aliases: Vec<(String, String)> = c
                .prepare(
                    "select alias_text, alias_type from symbol_aliases where symbol_id = ?",
                )?
                .query_map(params![id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
                .collect::<Result<_, _>>()?;
            let domains: Vec<(String, String)> = c
                .prepare(
                    "select d.code, coalesce(m.meaning_in_domain_ko, '')
                     from symbol_domain_map m
                     join domains d on d.id = m.domain_id
                     where m.symbol_id = ?",
                )?
                .query_map(params![id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
                .collect::<Result<_, _>>()?;
            let examples: Vec<(String, String)> = c
                .prepare(
                    "select example_latex, coalesce(example_explanation_ko, '')
                     from symbol_examples where symbol_id = ?",
                )?
                .query_map(params![id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
                .collect::<Result<_, _>>()?;
            s.aliases = aliases;
            s.domains = domains;
            s.examples = examples;
            symbols.push(s);
        }
        Ok(symbols)
    })
}
