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
