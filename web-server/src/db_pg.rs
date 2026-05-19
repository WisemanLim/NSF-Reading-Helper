//! PostgreSQL backend for the web-server. Mirrors the API surface of the
//! sqlite [`crate::core::db::Db`] used by Tauri, but native async on
//! `tokio-postgres` via a deadpool connection pool.

use std::str::FromStr;

use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use tokio_postgres::{Config, NoTls, Row};

use crate::core::error::{AppError, AppResult};
use crate::core::models::{
    AnalyzeRequest, AnalyzeResult, HistoryItem, Note, SearchOptions, SymbolDetail,
    SymbolDomainMeaning, SymbolExample, SymbolHit, SymbolRow,
};

const MIGRATION_SQL: &str = include_str!("../migrations/postgres.sql");

#[derive(Clone)]
pub struct PgDb {
    pool: Pool,
}

impl PgDb {
    pub async fn connect(url: &str) -> AppResult<Self> {
        let pg_config = Config::from_str(url)
            .map_err(|e| AppError::Generic(format!("postgres URL parse: {e}")))?;
        let mgr_config = ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        };
        let mgr = Manager::from_config(pg_config, NoTls, mgr_config);
        let pool = Pool::builder(mgr)
            .max_size(16)
            .build()
            .map_err(|e| AppError::Generic(format!("pool build: {e}")))?;
        let db = Self { pool };
        db.migrate().await?;
        Ok(db)
    }

    async fn migrate(&self) -> AppResult<()> {
        let client = self.client().await?;
        client
            .batch_execute(MIGRATION_SQL)
            .await
            .map_err(|e| AppError::Generic(format!("migrate: {e}")))?;
        Ok(())
    }

    async fn client(&self) -> AppResult<deadpool_postgres::Client> {
        self.pool
            .get()
            .await
            .map_err(|e| AppError::Generic(format!("pool: {e}")))
    }

    pub async fn seed_if_empty(&self) -> AppResult<()> {
        let client = self.client().await?;
        let row = client
            .query_one("select count(*)::bigint from symbols", &[])
            .await
            .map_err(map_pg)?;
        let n: i64 = row.get(0);
        if n > 0 {
            return Ok(());
        }
        crate::seed_pg::install(self).await
    }

    // ── Inserts used by the seed installer ──────────────────────────────

    pub async fn insert_domain(
        &self,
        code: &str,
        name_ko: &str,
        name_en: Option<&str>,
    ) -> AppResult<i64> {
        let client = self.client().await?;
        client
            .execute(
                "insert into domains(code, name_ko, name_en) values($1,$2,$3)
                 on conflict (code) do nothing",
                &[&code, &name_ko, &name_en],
            )
            .await
            .map_err(map_pg)?;
        let row = client
            .query_one("select id from domains where code = $1", &[&code])
            .await
            .map_err(map_pg)?;
        Ok(row.get(0))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn insert_symbol(
        &self,
        unicode_char: Option<&str>,
        latex_command: Option<&str>,
        mathml_tag: Option<&str>,
        canonical_name_ko: &str,
        canonical_name_en: Option<&str>,
        reading_ko: Option<&str>,
        reading_en: Option<&str>,
        short_meaning_ko: &str,
        short_meaning_en: Option<&str>,
        symbol_type: &str,
        difficulty_level: i16,
        is_operator: bool,
        is_relation: bool,
        is_delimiter: bool,
        is_greek: bool,
        is_constant: bool,
        is_function: bool,
    ) -> AppResult<i64> {
        let client = self.client().await?;
        let row = client
            .query_one(
                "insert into symbols(
                    unicode_char, latex_command, mathml_tag,
                    canonical_name_ko, canonical_name_en,
                    reading_ko, reading_en,
                    short_meaning_ko, short_meaning_en,
                    symbol_type, difficulty_level,
                    is_operator, is_relation, is_delimiter,
                    is_greek, is_constant, is_function
                 ) values($1,$2,$3, $4,$5, $6,$7, $8,$9, $10,$11, $12,$13,$14,$15,$16,$17)
                 returning id",
                &[
                    &unicode_char, &latex_command, &mathml_tag,
                    &canonical_name_ko, &canonical_name_en,
                    &reading_ko, &reading_en,
                    &short_meaning_ko, &short_meaning_en,
                    &symbol_type, &difficulty_level,
                    &is_operator, &is_relation, &is_delimiter,
                    &is_greek, &is_constant, &is_function,
                ],
            )
            .await
            .map_err(map_pg)?;
        Ok(row.get(0))
    }

    pub async fn insert_alias(
        &self,
        symbol_id: i64,
        alias_text: &str,
        alias_type: &str,
        locale: &str,
    ) -> AppResult<()> {
        let client = self.client().await?;
        client
            .execute(
                "insert into symbol_aliases(symbol_id, alias_text, alias_type, locale)
                 values($1,$2,$3,$4)
                 on conflict do nothing",
                &[&symbol_id, &alias_text, &alias_type, &locale],
            )
            .await
            .map_err(map_pg)?;
        Ok(())
    }

    pub async fn link_domain(
        &self,
        symbol_id: i64,
        domain_id: i64,
        meaning_ko: Option<&str>,
        priority: i16,
    ) -> AppResult<()> {
        let client = self.client().await?;
        client
            .execute(
                "insert into symbol_domain_map(symbol_id, domain_id, meaning_in_domain_ko, priority)
                 values($1,$2,$3,$4) on conflict do nothing",
                &[&symbol_id, &domain_id, &meaning_ko, &priority],
            )
            .await
            .map_err(map_pg)?;
        Ok(())
    }

    pub async fn insert_example(
        &self,
        symbol_id: i64,
        latex: &str,
        explain: Option<&str>,
        difficulty: i16,
    ) -> AppResult<()> {
        let client = self.client().await?;
        client
            .execute(
                "insert into symbol_examples(symbol_id, example_latex, example_explanation_ko, difficulty_level)
                 values($1,$2,$3,$4)",
                &[&symbol_id, &latex, &explain, &difficulty],
            )
            .await
            .map_err(map_pg)?;
        Ok(())
    }

    // ── Read paths used by routes ───────────────────────────────────────

    pub async fn lookup_by_latex(&self, latex: &str) -> AppResult<Option<SymbolRow>> {
        let client = self.client().await?;
        let row = client
            .query_opt(
                "select * from symbols where latex_command = $1 limit 1",
                &[&latex],
            )
            .await
            .map_err(map_pg)?;
        Ok(row.map(row_to_symbol))
    }

    pub async fn lookup_by_unicode(&self, ch: &str) -> AppResult<Option<SymbolRow>> {
        let client = self.client().await?;
        let row = client
            .query_opt(
                "select * from symbols where unicode_char = $1 limit 1",
                &[&ch],
            )
            .await
            .map_err(map_pg)?;
        Ok(row.map(row_to_symbol))
    }

    pub async fn symbol_search(
        &self,
        q: &str,
        opts: Option<&SearchOptions>,
    ) -> AppResult<Vec<SymbolHit>> {
        let q = q.trim();
        if q.is_empty() {
            return Ok(vec![]);
        }
        let limit = opts.and_then(|o| o.limit).unwrap_or(30).max(1);
        let client = self.client().await?;
        let q_lower = q.to_lowercase();
        let like = format!("%{q_lower}%");

        // Single query: union exact / substring / trigram / Levenshtein scores.
        // Postgres `similarity()` (pg_trgm) and `levenshtein()` (fuzzystrmatch)
        // give us ranking without a separate FTS table.
        let rows = client.query(
            "with candidates as (
                select s.id,
                    case
                        when s.unicode_char = $1 then 0
                        when lower(s.canonical_name_ko) = $2 then 0
                        when lower(coalesce(s.canonical_name_en,'')) = $2 then 0
                        when lower(coalesce(s.latex_command,'')) = $2 then 0
                        when lower(s.canonical_name_ko) like $3 then 1
                        when lower(coalesce(s.canonical_name_en,'')) like $3 then 1
                        when lower(coalesce(s.latex_command,'')) like $3 then 1
                        when similarity(s.canonical_name_ko, $2) > 0.3 then 2
                        when levenshtein(lower(s.canonical_name_ko), $2) <= $4 then 3
                        when levenshtein(lower(coalesce(s.canonical_name_en,'')), $2) <= $4 then 3
                        else 9
                    end as score
                from symbols s
                left join symbol_aliases a on a.symbol_id = s.id
                where s.unicode_char = $1
                    or lower(s.canonical_name_ko) like $3
                    or lower(coalesce(s.canonical_name_en,'')) like $3
                    or lower(coalesce(s.latex_command,'')) like $3
                    or lower(coalesce(a.alias_text,'')) like $3
                    or similarity(s.canonical_name_ko, $2) > 0.3
                    or levenshtein(lower(s.canonical_name_ko), $2) <= $4
            )
            select id, min(score) as best
            from candidates
            where score < 9
            group by id
            order by best asc, id asc
            limit $5",
            &[
                &q,
                &q_lower,
                &like,
                &((q.chars().count() as i32 / 3) + 1),
                &(limit as i64),
            ],
        )
        .await
        .map_err(map_pg)?;

        if rows.is_empty() {
            return Ok(vec![]);
        }
        let ids: Vec<i64> = rows.iter().map(|r| r.get::<_, i64>(0)).collect();
        let hits = client
            .query(
                "select id, unicode_char, latex_command, canonical_name_ko,
                        reading_ko, short_meaning_ko, symbol_type
                 from symbols where id = any($1::bigint[])",
                &[&ids],
            )
            .await
            .map_err(map_pg)?;
        let mut by_id: std::collections::HashMap<i64, SymbolHit> =
            hits.into_iter().map(|r| (r.get(0), row_to_hit(r))).collect();
        Ok(ids.into_iter().filter_map(|id| by_id.remove(&id)).collect())
    }

    pub async fn symbol_detail(&self, id: i64) -> AppResult<SymbolDetail> {
        let client = self.client().await?;
        let row = client
            .query_opt("select * from symbols where id = $1", &[&id])
            .await
            .map_err(map_pg)?
            .ok_or_else(|| AppError::NotFound(format!("symbol#{id}")))?;
        let symbol = row_to_symbol(row);

        let aliases: Vec<String> = client
            .query(
                "select alias_text from symbol_aliases where symbol_id = $1",
                &[&id],
            )
            .await
            .map_err(map_pg)?
            .into_iter()
            .map(|r| r.get::<_, String>(0))
            .collect();

        let domains: Vec<SymbolDomainMeaning> = client
            .query(
                "select d.code, d.name_ko, m.meaning_in_domain_ko, m.example_usage
                 from symbol_domain_map m
                 join domains d on d.id = m.domain_id
                 where m.symbol_id = $1
                 order by m.priority",
                &[&id],
            )
            .await
            .map_err(map_pg)?
            .into_iter()
            .map(|r| SymbolDomainMeaning {
                code: r.get(0),
                name_ko: r.get(1),
                meaning_ko: r.get(2),
                example_usage: r.get(3),
            })
            .collect();

        let examples = self.symbol_examples(id).await?;

        let similar: Vec<SymbolHit> = client
            .query(
                "select id, unicode_char, latex_command, canonical_name_ko,
                        reading_ko, short_meaning_ko, symbol_type
                 from symbols where id != $1 and symbol_type = $2 limit 5",
                &[&id, &symbol.symbol_type],
            )
            .await
            .map_err(map_pg)?
            .into_iter()
            .map(row_to_hit)
            .collect();

        Ok(SymbolDetail {
            row: symbol,
            aliases,
            domains,
            examples,
            similar,
        })
    }

    pub async fn symbol_examples(&self, id: i64) -> AppResult<Vec<SymbolExample>> {
        let client = self.client().await?;
        let rows = client
            .query(
                "select e.id, e.symbol_id, d.code, e.example_latex,
                        e.example_reading_ko, e.example_explanation_ko, e.difficulty_level
                 from symbol_examples e
                 left join domains d on d.id = e.domain_id
                 where e.symbol_id = $1
                 order by e.difficulty_level, e.id",
                &[&id],
            )
            .await
            .map_err(map_pg)?;
        Ok(rows
            .into_iter()
            .map(|r| SymbolExample {
                id: r.get(0),
                symbol_id: r.get(1),
                domain_code: r.get(2),
                example_latex: r.get(3),
                example_reading_ko: r.get(4),
                example_explanation_ko: r.get(5),
                difficulty_level: r.get::<_, i16>(6) as i64,
            })
            .collect())
    }

    pub async fn save_history(
        &self,
        req: &AnalyzeRequest,
        result: &AnalyzeResult,
    ) -> AppResult<i64> {
        let client = self.client().await?;
        let row = client
            .query_one(
                "insert into expressions(
                    input_format, raw_input, normalized_latex, normalized_mathml,
                    speech_text_ko, rendered_ok
                 ) values($1,$2,$3,$4, $5, true) returning id",
                &[
                    &req.format,
                    &req.raw_input,
                    &result.normalized_latex,
                    &result.normalized_mathml,
                    &result.speech.ko,
                ],
            )
            .await
            .map_err(map_pg)?;
        Ok(row.get(0))
    }

    pub async fn history_list(&self, limit: i64) -> AppResult<Vec<HistoryItem>> {
        let client = self.client().await?;
        let rows = client
            .query(
                "select id, raw_input, input_format, speech_text_ko, created_at
                 from expressions order by id desc limit $1",
                &[&limit],
            )
            .await
            .map_err(map_pg)?;
        Ok(rows
            .into_iter()
            .map(|r| HistoryItem {
                expression_id: r.get(0),
                raw_input: r.get(1),
                format: r.get(2),
                speech_ko: r.get(3),
                created_at: r.get(4),
            })
            .collect())
    }

    pub async fn favorites_add(&self, symbol_id: i64) -> AppResult<()> {
        let client = self.client().await?;
        client
            .execute(
                "insert into favorites(symbol_id) values($1) on conflict do nothing",
                &[&symbol_id],
            )
            .await
            .map_err(map_pg)?;
        Ok(())
    }

    pub async fn favorites_remove(&self, symbol_id: i64) -> AppResult<()> {
        let client = self.client().await?;
        client
            .execute("delete from favorites where symbol_id = $1", &[&symbol_id])
            .await
            .map_err(map_pg)?;
        Ok(())
    }

    pub async fn favorites_list(&self) -> AppResult<Vec<SymbolHit>> {
        let client = self.client().await?;
        let rows = client
            .query(
                "select s.id, s.unicode_char, s.latex_command, s.canonical_name_ko,
                        s.reading_ko, s.short_meaning_ko, s.symbol_type
                 from favorites f join symbols s on s.id = f.symbol_id
                 order by f.created_at desc",
                &[],
            )
            .await
            .map_err(map_pg)?;
        Ok(rows.into_iter().map(row_to_hit).collect())
    }

    pub async fn notes_upsert(&self, note: Note) -> AppResult<Note> {
        let client = self.client().await?;
        let tags_json = serde_json::to_string(&note.tags)?;
        if let Some(id) = note.id {
            client
                .execute(
                    "update notes set expression_id=$2, symbol_id=$3,
                        body_md=$4, tags=$5,
                        updated_at = (extract(epoch from now()) * 1000)::bigint
                     where id = $1",
                    &[
                        &id,
                        &note.expression_id,
                        &note.symbol_id,
                        &note.body_md,
                        &tags_json,
                    ],
                )
                .await
                .map_err(map_pg)?;
            Ok(note)
        } else {
            let row = client
                .query_one(
                    "insert into notes(expression_id, symbol_id, body_md, tags)
                     values($1,$2,$3,$4) returning id",
                    &[
                        &note.expression_id,
                        &note.symbol_id,
                        &note.body_md,
                        &tags_json,
                    ],
                )
                .await
                .map_err(map_pg)?;
            let mut out = note;
            out.id = Some(row.get(0));
            Ok(out)
        }
    }

    pub async fn notes_list(&self) -> AppResult<Vec<Note>> {
        let client = self.client().await?;
        let rows = client
            .query(
                "select id, expression_id, symbol_id, body_md, tags from notes order by id desc",
                &[],
            )
            .await
            .map_err(map_pg)?;
        Ok(rows
            .into_iter()
            .map(|r| {
                let tags_json: Option<String> = r.get(4);
                let tags = tags_json
                    .as_deref()
                    .and_then(|s| serde_json::from_str(s).ok())
                    .unwrap_or_default();
                Note {
                    id: r.get(0),
                    expression_id: r.get(1),
                    symbol_id: r.get(2),
                    body_md: r.get(3),
                    tags,
                }
            })
            .collect())
    }

    pub async fn settings_get(&self, key: &str) -> AppResult<Option<String>> {
        let client = self.client().await?;
        let row = client
            .query_opt("select value from app_settings where key = $1", &[&key])
            .await
            .map_err(map_pg)?;
        Ok(row.map(|r| r.get::<_, String>(0)))
    }

    pub async fn settings_set(&self, key: &str, value: &str) -> AppResult<()> {
        let client = self.client().await?;
        client
            .execute(
                "insert into app_settings(key,value) values($1,$2)
                 on conflict (key) do update set value = excluded.value",
                &[&key, &value],
            )
            .await
            .map_err(map_pg)?;
        Ok(())
    }

    pub async fn rebuild_fts(&self) -> AppResult<()> {
        // Postgres uses GIN trigram indexes on the live columns — nothing to rebuild.
        Ok(())
    }
}

fn map_pg(e: tokio_postgres::Error) -> AppError {
    AppError::Generic(format!("postgres: {e}"))
}

fn row_to_symbol(r: Row) -> SymbolRow {
    SymbolRow {
        id: r.get("id"),
        unicode_char: r.get("unicode_char"),
        latex_command: r.get("latex_command"),
        mathml_tag: r.get("mathml_tag"),
        canonical_name_ko: r.get("canonical_name_ko"),
        canonical_name_en: r.get("canonical_name_en"),
        reading_ko: r.get("reading_ko"),
        reading_en: r.get("reading_en"),
        short_meaning_ko: r.get("short_meaning_ko"),
        short_meaning_en: r.get("short_meaning_en"),
        symbol_type: r.get("symbol_type"),
        difficulty_level: r.get::<_, i16>("difficulty_level") as i64,
        is_operator: r.get("is_operator"),
        is_relation: r.get("is_relation"),
        is_delimiter: r.get("is_delimiter"),
        is_greek: r.get("is_greek"),
        is_constant: r.get("is_constant"),
        is_function: r.get("is_function"),
        notes: r.get("notes"),
    }
}

fn row_to_hit(r: Row) -> SymbolHit {
    SymbolHit {
        id: r.get(0),
        unicode_char: r.get(1),
        latex_command: r.get(2),
        canonical_name_ko: r.get(3),
        reading_ko: r.get(4),
        short_meaning_ko: r.get(5),
        symbol_type: r.get(6),
        matched_alias: None,
    }
}
