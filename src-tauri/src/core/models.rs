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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolDomainMeaning {
    pub code: String,
    pub name_ko: String,
    pub meaning_ko: Option<String>,
    pub example_usage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolRow {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolExample {
    pub id: i64,
    pub symbol_id: i64,
    pub domain_code: Option<String>,
    pub example_latex: String,
    pub example_reading_ko: Option<String>,
    pub example_explanation_ko: Option<String>,
    pub difficulty_level: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolDetail {
    #[serde(flatten)]
    pub row: SymbolRow,
    pub aliases: Vec<String>,
    pub domains: Vec<SymbolDomainMeaning>,
    pub examples: Vec<SymbolExample>,
    pub similar: Vec<SymbolHit>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeRequest {
    pub raw_input: String,
    pub format: String,
    pub locale: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeToken {
    pub index: i64,
    pub token_text: String,
    pub role: Option<String>,
    pub symbol_id: Option<i64>,
    pub name_ko: Option<String>,
    pub reading_ko: Option<String>,
    pub meaning_ko: Option<String>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryItem {
    pub expression_id: i64,
    pub raw_input: String,
    pub format: String,
    pub speech_ko: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: Option<i64>,
    pub expression_id: Option<i64>,
    pub symbol_id: Option<i64>,
    pub body_md: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchOptions {
    pub domain: Option<String>,
    pub symbol_type: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
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
