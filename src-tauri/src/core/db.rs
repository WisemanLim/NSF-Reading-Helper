use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::core::error::{AppError, AppResult};
use crate::core::models::{
    AnalyzeRequest, AnalyzeResult, HistoryItem, Note, SearchOptions, SymbolDetail,
    SymbolDomainMeaning, SymbolExample, SymbolHit, SymbolRow,
};
use crate::core::seed;

pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    pub fn open(path: &Path) -> AppResult<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(MIGRATION_V1)?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    fn lock(&self) -> std::sync::MutexGuard<Connection> {
        self.conn.lock().expect("db poisoned")
    }

    pub fn seed_if_empty(&self) -> AppResult<()> {
        let conn = self.lock();
        let count: i64 =
            conn.query_row("select count(*) from symbols", [], |r| r.get(0))?;
        if count > 0 {
            return Ok(());
        }
        drop(conn);
        seed::install(self)?;
        Ok(())
    }

    pub fn insert_domain(
        &self,
        code: &str,
        name_ko: &str,
        name_en: Option<&str>,
    ) -> AppResult<i64> {
        let conn = self.lock();
        conn.execute(
            "insert or ignore into domains(code, name_ko, name_en) values(?, ?, ?)",
            params![code, name_ko, name_en],
        )?;
        let id: i64 = conn.query_row(
            "select id from domains where code = ?",
            params![code],
            |r| r.get(0),
        )?;
        Ok(id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn insert_symbol(&self, s: &seed::SymbolSeed) -> AppResult<i64> {
        let conn = self.lock();
        conn.execute(
            "insert into symbols(
                unicode_char, latex_command, mathml_tag,
                canonical_name_ko, canonical_name_en,
                reading_ko, reading_en,
                short_meaning_ko, short_meaning_en,
                symbol_type, difficulty_level,
                is_operator, is_relation, is_delimiter, is_greek, is_constant, is_function,
                notes
            ) values(?,?,?, ?,?, ?,?, ?,?, ?,?, ?,?,?,?,?,?, ?)",
            params![
                s.unicode_char, s.latex_command, s.mathml_tag,
                s.canonical_name_ko, s.canonical_name_en,
                s.reading_ko, s.reading_en,
                s.short_meaning_ko, s.short_meaning_en,
                s.symbol_type, s.difficulty_level,
                s.is_operator, s.is_relation, s.is_delimiter, s.is_greek, s.is_constant, s.is_function,
                s.notes,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn insert_alias(
        &self,
        symbol_id: i64,
        alias: &str,
        alias_type: &str,
        locale: &str,
    ) -> AppResult<()> {
        let conn = self.lock();
        conn.execute(
            "insert or ignore into symbol_aliases(symbol_id, alias_text, alias_type, locale)
             values(?,?,?,?)",
            params![symbol_id, alias, alias_type, locale],
        )?;
        Ok(())
    }

    pub fn link_domain(
        &self,
        symbol_id: i64,
        domain_id: i64,
        meaning_ko: Option<&str>,
        priority: i64,
    ) -> AppResult<()> {
        let conn = self.lock();
        conn.execute(
            "insert or ignore into symbol_domain_map(symbol_id, domain_id, meaning_in_domain_ko, priority)
             values(?,?,?,?)",
            params![symbol_id, domain_id, meaning_ko, priority],
        )?;
        Ok(())
    }

    pub fn insert_example(
        &self,
        symbol_id: i64,
        latex: &str,
        explain: Option<&str>,
        difficulty: i64,
    ) -> AppResult<()> {
        let conn = self.lock();
        conn.execute(
            "insert into symbol_examples(symbol_id, example_latex, example_explanation_ko, difficulty_level)
             values(?,?,?,?)",
            params![symbol_id, latex, explain, difficulty],
        )?;
        Ok(())
    }

    pub fn lookup_by_latex(&self, latex_command: &str) -> AppResult<Option<SymbolRow>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "select * from symbols where latex_command = ? limit 1",
        )?;
        let row = stmt
            .query_row(params![latex_command], row_to_symbol)
            .optional()?;
        Ok(row)
    }

    pub fn lookup_by_unicode(&self, ch: &str) -> AppResult<Option<SymbolRow>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "select * from symbols where unicode_char = ? limit 1",
        )?;
        let row = stmt.query_row(params![ch], row_to_symbol).optional()?;
        Ok(row)
    }

    pub fn symbol_search(
        &self,
        q: &str,
        opts: Option<&SearchOptions>,
    ) -> AppResult<Vec<SymbolHit>> {
        let limit = opts.and_then(|o| o.limit).unwrap_or(30);
        let conn = self.lock();
        // Match on canonical_ko, canonical_en, latex, unicode, or aliases.
        let pattern = format!("%{}%", q.to_lowercase());
        let mut stmt = conn.prepare(
            "select distinct s.id, s.unicode_char, s.latex_command, s.canonical_name_ko,
                 s.reading_ko, s.short_meaning_ko, s.symbol_type, null
             from symbols s
             left join symbol_aliases a on a.symbol_id = s.id
             where lower(s.canonical_name_ko) like ?1
                or lower(coalesce(s.canonical_name_en,'')) like ?1
                or lower(coalesce(s.latex_command,'')) like ?1
                or s.unicode_char = ?2
                or lower(coalesce(a.alias_text,'')) like ?1
             order by length(s.canonical_name_ko) asc
             limit ?3",
        )?;
        let rows = stmt
            .query_map(params![pattern, q, limit], row_to_hit)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn symbol_detail(&self, id: i64) -> AppResult<SymbolDetail> {
        let conn = self.lock();
        let mut stmt =
            conn.prepare("select * from symbols where id = ?")?;
        let row = stmt
            .query_row(params![id], row_to_symbol)
            .optional()?
            .ok_or_else(|| AppError::NotFound(format!("symbol#{id}")))?;

        let aliases: Vec<String> = conn
            .prepare("select alias_text from symbol_aliases where symbol_id = ?")?
            .query_map(params![id], |r| r.get::<_, String>(0))?
            .collect::<Result<_, _>>()?;

        let domains: Vec<SymbolDomainMeaning> = conn
            .prepare(
                "select d.code, d.name_ko, m.meaning_in_domain_ko, m.example_usage
                 from symbol_domain_map m
                 join domains d on d.id = m.domain_id
                 where m.symbol_id = ?
                 order by m.priority",
            )?
            .query_map(params![id], |r| {
                Ok(SymbolDomainMeaning {
                    code: r.get(0)?,
                    name_ko: r.get(1)?,
                    meaning_ko: r.get(2)?,
                    example_usage: r.get(3)?,
                })
            })?
            .collect::<Result<_, _>>()?;

        let examples: Vec<SymbolExample> = conn
            .prepare(
                "select e.id, e.symbol_id, d.code, e.example_latex,
                    e.example_reading_ko, e.example_explanation_ko, e.difficulty_level
                 from symbol_examples e
                 left join domains d on d.id = e.domain_id
                 where e.symbol_id = ?
                 order by e.difficulty_level, e.id
                 limit 5",
            )?
            .query_map(params![id], |r| {
                Ok(SymbolExample {
                    id: r.get(0)?,
                    symbol_id: r.get(1)?,
                    domain_code: r.get(2)?,
                    example_latex: r.get(3)?,
                    example_reading_ko: r.get(4)?,
                    example_explanation_ko: r.get(5)?,
                    difficulty_level: r.get(6)?,
                })
            })?
            .collect::<Result<_, _>>()?;

        let similar: Vec<SymbolHit> = conn
            .prepare(
                "select id, unicode_char, latex_command, canonical_name_ko,
                    reading_ko, short_meaning_ko, symbol_type, null
                 from symbols
                 where id != ?1 and symbol_type = ?2
                 limit 5",
            )?
            .query_map(params![id, row.symbol_type], row_to_hit)?
            .collect::<Result<_, _>>()?;

        Ok(SymbolDetail {
            row,
            aliases,
            domains,
            examples,
            similar,
        })
    }

    pub fn symbol_examples(&self, id: i64) -> AppResult<Vec<SymbolExample>> {
        let conn = self.lock();
        let v = conn
            .prepare(
                "select e.id, e.symbol_id, d.code, e.example_latex,
                    e.example_reading_ko, e.example_explanation_ko, e.difficulty_level
                 from symbol_examples e
                 left join domains d on d.id = e.domain_id
                 where e.symbol_id = ?",
            )?
            .query_map(params![id], |r| {
                Ok(SymbolExample {
                    id: r.get(0)?,
                    symbol_id: r.get(1)?,
                    domain_code: r.get(2)?,
                    example_latex: r.get(3)?,
                    example_reading_ko: r.get(4)?,
                    example_explanation_ko: r.get(5)?,
                    difficulty_level: r.get(6)?,
                })
            })?
            .collect::<Result<_, _>>()?;
        Ok(v)
    }

    pub fn save_history(
        &self,
        req: &AnalyzeRequest,
        result: &AnalyzeResult,
    ) -> AppResult<i64> {
        let conn = self.lock();
        conn.execute(
            "insert into expressions(
                input_format, raw_input, normalized_latex, normalized_mathml,
                speech_text_ko, rendered_ok, created_at
             ) values(?,?,?,?, ?, 1, strftime('%s','now')*1000)",
            params![
                req.format,
                req.raw_input,
                result.normalized_latex,
                result.normalized_mathml,
                result.speech.ko,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn history_list(&self, limit: i64) -> AppResult<Vec<HistoryItem>> {
        let conn = self.lock();
        let v = conn
            .prepare(
                "select id, raw_input, input_format, speech_text_ko, created_at
                 from expressions
                 order by id desc
                 limit ?",
            )?
            .query_map(params![limit], |r| {
                Ok(HistoryItem {
                    expression_id: r.get(0)?,
                    raw_input: r.get(1)?,
                    format: r.get(2)?,
                    speech_ko: r.get(3)?,
                    created_at: r.get(4)?,
                })
            })?
            .collect::<Result<_, _>>()?;
        Ok(v)
    }

    pub fn favorites_add(&self, symbol_id: i64) -> AppResult<()> {
        let conn = self.lock();
        conn.execute(
            "insert or ignore into favorites(symbol_id, created_at)
             values(?, strftime('%s','now')*1000)",
            params![symbol_id],
        )?;
        Ok(())
    }
    pub fn favorites_remove(&self, symbol_id: i64) -> AppResult<()> {
        let conn = self.lock();
        conn.execute("delete from favorites where symbol_id = ?", params![symbol_id])?;
        Ok(())
    }
    pub fn favorites_list(&self) -> AppResult<Vec<SymbolHit>> {
        let conn = self.lock();
        let v = conn
            .prepare(
                "select s.id, s.unicode_char, s.latex_command, s.canonical_name_ko,
                    s.reading_ko, s.short_meaning_ko, s.symbol_type, null
                 from favorites f join symbols s on s.id = f.symbol_id
                 order by f.created_at desc",
            )?
            .query_map([], row_to_hit)?
            .collect::<Result<_, _>>()?;
        Ok(v)
    }

    pub fn notes_upsert(&self, note: Note) -> AppResult<Note> {
        let conn = self.lock();
        let tags_json = serde_json::to_string(&note.tags)?;
        if let Some(id) = note.id {
            conn.execute(
                "update notes set expression_id=?, symbol_id=?, body_md=?, tags=?,
                    updated_at=strftime('%s','now')*1000 where id = ?",
                params![note.expression_id, note.symbol_id, note.body_md, tags_json, id],
            )?;
            Ok(note)
        } else {
            conn.execute(
                "insert into notes(expression_id, symbol_id, body_md, tags, created_at, updated_at)
                 values(?,?,?,?, strftime('%s','now')*1000, strftime('%s','now')*1000)",
                params![note.expression_id, note.symbol_id, note.body_md, tags_json],
            )?;
            let mut out = note;
            out.id = Some(conn.last_insert_rowid());
            Ok(out)
        }
    }

    pub fn notes_list(&self) -> AppResult<Vec<Note>> {
        let conn = self.lock();
        let v = conn
            .prepare(
                "select id, expression_id, symbol_id, body_md, tags from notes order by id desc",
            )?
            .query_map([], |r| {
                let tags_json: Option<String> = r.get(4)?;
                let tags: Vec<String> = tags_json
                    .as_deref()
                    .and_then(|s| serde_json::from_str(s).ok())
                    .unwrap_or_default();
                Ok(Note {
                    id: r.get(0)?,
                    expression_id: r.get(1)?,
                    symbol_id: r.get(2)?,
                    body_md: r.get(3)?,
                    tags,
                })
            })?
            .collect::<Result<_, _>>()?;
        Ok(v)
    }

    pub fn settings_get(&self, key: &str) -> AppResult<Option<String>> {
        let conn = self.lock();
        let r = conn
            .query_row(
                "select value from app_settings where key = ?",
                params![key],
                |r| r.get::<_, String>(0),
            )
            .optional()?;
        Ok(r)
    }

    pub fn settings_set(&self, key: &str, value: &str) -> AppResult<()> {
        let conn = self.lock();
        conn.execute(
            "insert into app_settings(key,value) values(?,?)
             on conflict(key) do update set value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }
}

fn row_to_symbol(r: &Row) -> rusqlite::Result<SymbolRow> {
    Ok(SymbolRow {
        id: r.get("id")?,
        unicode_char: r.get("unicode_char")?,
        latex_command: r.get("latex_command")?,
        mathml_tag: r.get("mathml_tag")?,
        canonical_name_ko: r.get("canonical_name_ko")?,
        canonical_name_en: r.get("canonical_name_en")?,
        reading_ko: r.get("reading_ko")?,
        reading_en: r.get("reading_en")?,
        short_meaning_ko: r.get("short_meaning_ko")?,
        short_meaning_en: r.get("short_meaning_en")?,
        symbol_type: r.get("symbol_type")?,
        difficulty_level: r.get("difficulty_level")?,
        is_operator: r.get("is_operator")?,
        is_relation: r.get("is_relation")?,
        is_delimiter: r.get("is_delimiter")?,
        is_greek: r.get("is_greek")?,
        is_constant: r.get("is_constant")?,
        is_function: r.get("is_function")?,
        notes: r.get("notes")?,
    })
}

fn row_to_hit(r: &Row) -> rusqlite::Result<SymbolHit> {
    Ok(SymbolHit {
        id: r.get(0)?,
        unicode_char: r.get(1)?,
        latex_command: r.get(2)?,
        canonical_name_ko: r.get(3)?,
        reading_ko: r.get(4)?,
        short_meaning_ko: r.get(5)?,
        symbol_type: r.get(6)?,
        matched_alias: r.get(7)?,
    })
}

const MIGRATION_V1: &str = r#"
create table if not exists domains (
    id integer primary key autoincrement,
    code text unique not null,
    name_ko text not null,
    name_en text,
    description text,
    created_at integer default (strftime('%s','now')*1000)
);

create table if not exists symbols (
    id integer primary key autoincrement,
    unicode_char text,
    latex_command text,
    mathml_tag text,
    canonical_name_ko text not null,
    canonical_name_en text,
    reading_ko text,
    reading_en text,
    short_meaning_ko text not null,
    short_meaning_en text,
    symbol_type text not null,
    difficulty_level integer default 1,
    is_operator integer default 0,
    is_relation integer default 0,
    is_delimiter integer default 0,
    is_greek integer default 0,
    is_constant integer default 0,
    is_function integer default 0,
    notes text,
    created_at integer default (strftime('%s','now')*1000),
    updated_at integer default (strftime('%s','now')*1000)
);

create table if not exists symbol_aliases (
    id integer primary key autoincrement,
    symbol_id integer not null references symbols(id) on delete cascade,
    alias_text text not null,
    alias_type text not null,
    locale text default 'ko',
    unique(symbol_id, alias_text, alias_type, locale)
);

create table if not exists symbol_domain_map (
    id integer primary key autoincrement,
    symbol_id integer not null references symbols(id) on delete cascade,
    domain_id integer not null references domains(id) on delete cascade,
    priority integer default 1,
    meaning_in_domain_ko text,
    example_usage text,
    unique(symbol_id, domain_id)
);

create table if not exists symbol_examples (
    id integer primary key autoincrement,
    symbol_id integer not null references symbols(id) on delete cascade,
    domain_id integer references domains(id) on delete set null,
    example_latex text not null,
    example_reading_ko text,
    example_explanation_ko text,
    difficulty_level integer default 1,
    created_at integer default (strftime('%s','now')*1000)
);

create table if not exists expressions (
    id integer primary key autoincrement,
    input_format text not null,
    raw_input text not null,
    normalized_latex text,
    normalized_mathml text,
    rendered_ok integer default 0,
    speech_text_ko text,
    speech_text_en text,
    created_at integer default (strftime('%s','now')*1000)
);

create table if not exists favorites (
    id integer primary key autoincrement,
    symbol_id integer not null references symbols(id) on delete cascade,
    created_at integer default (strftime('%s','now')*1000),
    unique(symbol_id)
);

create table if not exists notes (
    id integer primary key autoincrement,
    expression_id integer references expressions(id) on delete set null,
    symbol_id integer references symbols(id) on delete set null,
    body_md text not null,
    tags text default '[]',
    created_at integer default (strftime('%s','now')*1000),
    updated_at integer default (strftime('%s','now')*1000)
);

create table if not exists app_settings (
    key text primary key,
    value text not null
);

create index if not exists idx_symbols_unicode on symbols(unicode_char);
create index if not exists idx_symbols_latex on symbols(latex_command);
create index if not exists idx_symbols_type on symbols(symbol_type);
create index if not exists idx_aliases_text on symbol_aliases(alias_text);
"#;
