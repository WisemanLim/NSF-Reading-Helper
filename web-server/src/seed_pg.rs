//! Postgres-side seed installer. Reuses the same `data/seed.json` payload as
//! the SQLite installer in `core::seed`, but issues INSERTs via [`PgDb`].

use serde::Deserialize;

use crate::core::error::AppResult;
use crate::db_pg::PgDb;

// Mirrors the sqlite seed.json schema. We share the JSON file via include_str!
// from the desktop tree so curation stays single-sourced.
const SEED_JSON: &str = include_str!("../../src-tauri/data/seed.json");

#[derive(Debug, Deserialize)]
struct SeedFile {
    domains: Vec<SeedDomain>,
    symbols: Vec<SeedSymbol>,
}

#[derive(Debug, Deserialize)]
struct SeedDomain {
    code: String,
    name_ko: String,
    #[serde(default)]
    name_en: Option<String>,
}

#[derive(Debug, Deserialize)]
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

fn has(flags: &[String], v: &str) -> bool {
    flags.iter().any(|f| f == v)
}

pub async fn install(db: &PgDb) -> AppResult<()> {
    let parsed: SeedFile = serde_json::from_str(SEED_JSON)?;

    let mut domain_ids: std::collections::HashMap<String, i64> =
        std::collections::HashMap::new();
    for d in &parsed.domains {
        let id = db
            .insert_domain(&d.code, &d.name_ko, d.name_en.as_deref())
            .await?;
        domain_ids.insert(d.code.clone(), id);
    }

    for s in &parsed.symbols {
        let id = db
            .insert_symbol(
                s.unicode_char.as_deref(),
                s.latex_command.as_deref(),
                s.mathml_tag.as_deref(),
                &s.canonical_name_ko,
                s.canonical_name_en.as_deref(),
                s.reading_ko.as_deref(),
                s.reading_en.as_deref(),
                &s.short_meaning_ko,
                s.short_meaning_en.as_deref(),
                &s.symbol_type,
                s.difficulty_level as i16,
                has(&s.flags, "operator"),
                has(&s.flags, "relation"),
                has(&s.flags, "delimiter"),
                has(&s.flags, "greek"),
                has(&s.flags, "constant"),
                has(&s.flags, "function"),
            )
            .await?;

        if let Some(c) = &s.unicode_char {
            db.insert_alias(id, c, "unicode", "ko").await?;
        }
        if let Some(c) = &s.latex_command {
            db.insert_alias(id, c, "latex", "ko").await?;
        }
        db.insert_alias(id, &s.canonical_name_ko, "ko-name", "ko").await?;
        if let Some(en) = &s.canonical_name_en {
            db.insert_alias(id, en, "en-name", "en").await?;
        }
        if let Some(r) = &s.reading_ko {
            db.insert_alias(id, r, "pronunciation", "ko").await?;
        }
        for (alias, ty) in &s.aliases {
            db.insert_alias(id, alias, ty, "ko").await?;
        }
        for (code, meaning) in &s.domains {
            if let Some(did) = domain_ids.get(code) {
                db.link_domain(id, *did, Some(meaning), 1).await?;
            }
        }
        for (latex, explain) in &s.examples {
            db.insert_example(id, latex, Some(explain), s.difficulty_level as i16)
                .await?;
        }
    }
    tracing::info!(
        "postgres seed installed: {} domains, {} symbols",
        parsed.domains.len(),
        parsed.symbols.len()
    );
    Ok(())
}
