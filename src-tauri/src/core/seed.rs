//! Seed dictionary — installed on first DB boot when `symbols` is empty.
//! Source data lives in `src-tauri/data/seed.json` and is embedded at compile
//! time via `include_str!`. Edit the JSON to expand the dictionary; no code
//! changes needed.

use std::collections::HashMap;

use serde::Deserialize;

use crate::core::db::Db;
use crate::core::error::AppResult;

const SEED_JSON: &str = include_str!("../../data/seed.json");

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

#[derive(Debug, Clone)]
pub struct SymbolSeed<'a> {
    pub unicode_char: Option<&'a str>,
    pub latex_command: Option<&'a str>,
    pub mathml_tag: Option<&'a str>,
    pub canonical_name_ko: &'a str,
    pub canonical_name_en: Option<&'a str>,
    pub reading_ko: Option<&'a str>,
    pub reading_en: Option<&'a str>,
    pub short_meaning_ko: &'a str,
    pub short_meaning_en: Option<&'a str>,
    pub symbol_type: &'a str,
    pub difficulty_level: i64,
    pub is_operator: bool,
    pub is_relation: bool,
    pub is_delimiter: bool,
    pub is_greek: bool,
    pub is_constant: bool,
    pub is_function: bool,
    pub notes: Option<&'a str>,
}

fn has(flags: &[String], v: &str) -> bool {
    flags.iter().any(|f| f == v)
}

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

    for raw in &parsed.symbols {
        let seed = SymbolSeed {
            unicode_char: raw.unicode_char.as_deref(),
            latex_command: raw.latex_command.as_deref(),
            mathml_tag: raw.mathml_tag.as_deref(),
            canonical_name_ko: &raw.canonical_name_ko,
            canonical_name_en: raw.canonical_name_en.as_deref(),
            reading_ko: raw.reading_ko.as_deref(),
            reading_en: raw.reading_en.as_deref(),
            short_meaning_ko: &raw.short_meaning_ko,
            short_meaning_en: raw.short_meaning_en.as_deref(),
            symbol_type: &raw.symbol_type,
            difficulty_level: raw.difficulty_level,
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

    tracing::info!(
        "seed installed: {} domains, {} symbols",
        parsed.domains.len(),
        parsed.symbols.len()
    );
    Ok(())
}
