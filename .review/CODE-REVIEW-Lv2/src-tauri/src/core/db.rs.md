# src-tauri/src/core/db.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: SQLite 단일 진실. 스키마(MIGRATION_V1, V2_FTS), levenshtein UDF, 그리고 검색·history·favorites·notes·settings 모든 쿼리.
- LOC: **773** (블록 모드 적용)
- 주요 export: `Db`, `Db::*` (다수), schema constants(private), `levenshtein` fn(private)
- 의존 모듈: `core::error`, `core::models`, `core::seed`
- 외부 패키지: `rusqlite` (bundled, functions)
- 부수효과: SQLite I/O
- 동시성: `Mutex<Connection>` 단일 — sync API. 호출자(Tauri/Web)가 락 관리.
- 테스트: 없음

> **블록-모드 리뷰**: 773줄 > 600 임계치. 블록 단위로 분석하며 모든 논리 블록을 다룹니다.

---

## 블록 1: 모듈 헤더 + 구조체 정의 (lines 1-17)

```rust path=src-tauri/src/core/db.rs lines=1-17
use std::path::Path;
use std::sync::Mutex;

use rusqlite::functions::FunctionFlags;
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
```

**분석 / Analysis**

- `Mutex<Connection>` — 표준 라이브러리 mutex. *왜 parking_lot 아닌가*: Cargo.toml 에 `parking_lot` 이 선언되어 있지만 사용 안 됨(데드 의존). std Mutex 가 poison 처리 포함.
- `rusqlite::OptionalExtension` — `Result<Option<T>>` 추출용.
- `pub mod core` 로 외부 가시 — `web-server` 도 같은 `Db` 를 컴파일.

## 블록 2: open / seed / fts (lines 19-102)

```rust path=src-tauri/src/core/db.rs lines=19-30
impl Db {
    pub fn open(path: &Path) -> AppResult<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(MIGRATION_V1)?;
        conn.execute_batch(MIGRATION_V2_FTS)?;
        register_levenshtein(&conn)?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    fn lock(&self) -> std::sync::MutexGuard<Connection> {
        self.conn.lock().expect("db poisoned")
    }
```

**분석 / Analysis**

- 마이그레이션 두 batch + UDF 등록 — *왜 idempotent SQL*: `create table if not exists` + `create virtual table if not exists`. 매 부팅 안전.
- **`.expect("db poisoned")`** — 다른 스레드가 락 보유 중 panic 한 경우. Tauri 에서는 가능성이 매우 낮음.
- 마이그레이션 버전 트래킹(예: `user_version` PRAGMA) 부재 — 향후 V3 추가 시 매번 batch 가 실행되지만 idempotent SQL 이라 OK. 단, ALTER 같은 비-idempotent 변경 시 별도 분기 필요.

```rust path=src-tauri/src/core/db.rs lines=31-39
    pub fn with_conn<R>(
        &self,
        f: impl FnOnce(&Connection) -> AppResult<R>,
    ) -> AppResult<R> {
        f(&self.lock())
    }
```

**분석 / Analysis**

- ad-hoc 쿼리를 위한 락 escape hatch. `pack.rs` 가 이 메서드를 통해 직접 SQL 작성. *왜*: 매번 메서드 추가 없이 패키지·도메인 조회 같은 일회성 쿼리 지원.

```rust path=src-tauri/src/core/db.rs lines=41-56
    pub fn seed_if_empty(&self) -> AppResult<()> {
        let conn = self.lock();
        let count: i64 =
            conn.query_row("select count(*) from symbols", [], |r| r.get(0))?;
        if count > 0 {
            drop(conn);
            // Ensure FTS index exists even if seed was installed by an older
            // build that didn't populate symbols_fts.
            self.rebuild_fts_if_empty()?;
            return Ok(());
        }
        drop(conn);
        seed::install(self)?;
        self.rebuild_fts()?;
        Ok(())
    }
```

**분석 / Analysis**

- **명시적 `drop(conn)`** — `seed::install(self)` 가 다시 `self.lock()` 을 호출하므로 deadlock 방지. *흔한 실수*: re-entrant lock 잘못 사용 시 데드락. 정확.
- empty 면 seed → FTS 재구축. non-empty 면 FTS 만 확인. 좋은 마이그레이션 친화.

```rust path=src-tauri/src/core/db.rs lines=58-92
    /// Wipe and rebuild the FTS5 trigram index from `symbols` + `symbol_aliases`.
    pub fn rebuild_fts(&self) -> AppResult<()> {
        let conn = self.lock();
        conn.execute("delete from symbols_fts", [])?;
        let mut stmt = conn.prepare(
            "select s.id, s.canonical_name_ko,
                    coalesce(s.canonical_name_en,''),
                    coalesce(s.latex_command,''),
                    coalesce(s.unicode_char,''),
                    coalesce(s.reading_ko,''),
                    coalesce((select group_concat(alias_text, ' ') from symbol_aliases where symbol_id = s.id),'')
             from symbols s",
        )?;
        let rows: Vec<(i64, String)> = stmt
            .query_map([], |r| {
                let id: i64 = r.get(0)?;
                let ko: String = r.get(1)?;
                let en: String = r.get(2)?;
                let latex: String = r.get(3)?;
                let uni: String = r.get(4)?;
                let reading: String = r.get(5)?;
                let aliases: String = r.get(6)?;
                let text = format!("{ko} {en} {latex} {uni} {reading} {aliases}");
                Ok((id, text))
            })?
            .collect::<Result<_, _>>()?;
        drop(stmt);
        for (id, text) in rows {
            conn.execute(
                "insert into symbols_fts(symbol_id, text) values(?, ?)",
                params![id, text],
            )?;
        }
        Ok(())
    }
```

**분석 / Analysis**

- 전체 wipe → re-populate 전략. *왜*: incremental 보다 단순. 사전 크기가 ~수천 row 면 ms 단위.
- 한 row 의 text 가 ko + en + latex + unicode + reading + aliases 모두 공백 결합 → FTS5 trigram 색인 대상.
- **모든 row 를 메모리에 로드한 뒤 INSERT 루프** — 큰 사전 시 메모리 사용. transaction 으로 묶지 않아 모든 INSERT 가 개별 commit → 느림.
- `drop(stmt)` — 동일 conn 에서 prepare 후 execute 하려면 stmt 를 먼저 drop 해야 함(rusqlite 의 borrow 규칙). 정확.

```rust path=src-tauri/src/core/db.rs lines=94-102
    fn rebuild_fts_if_empty(&self) -> AppResult<()> {
        let n: i64 = self
            .lock()
            .query_row("select count(*) from symbols_fts", [], |r| r.get(0))?;
        if n == 0 {
            self.rebuild_fts()?;
        }
        Ok(())
    }
```

**분석 / Analysis**

- 비어있을 때만 rebuild. 첫 lock 의 guard 가 scope 종료 시 풀림 → `rebuild_fts` 의 새 lock 안전.

## 블록 3: 시드 인서트 헬퍼 (lines 104-195)

```rust path=src-tauri/src/core/db.rs lines=104-147
    pub fn insert_domain(&self, code: &str, name_ko: &str, name_en: Option<&str>) -> AppResult<i64> {
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
    pub fn insert_symbol(&self, s: &seed::SymbolSeed<'_>) -> AppResult<i64> { ... }
```

**분석 / Analysis**

- `insert_domain`: insert-or-ignore + select-id. 2 round-trip. 단일 호출 비용 낮음.
- `insert_symbol`: 18개 컬럼 + 18개 placeholder. `last_insert_rowid()` 로 id 회수.
- `#[allow(clippy::too_many_arguments)]` — 함수가 `&SymbolSeed` 한 인자만 받으므로 사실 불필요한 allow. 잔재 lint 우회.

```rust path=src-tauri/src/core/db.rs lines=149-195
    pub fn insert_alias(&self, symbol_id: i64, alias: &str, alias_type: &str, locale: &str) -> AppResult<()> {
        let conn = self.lock();
        conn.execute(
            "insert or ignore into symbol_aliases(symbol_id, alias_text, alias_type, locale)
             values(?,?,?,?)",
            params![symbol_id, alias, alias_type, locale],
        )?;
        Ok(())
    }

    pub fn link_domain(&self, symbol_id, domain_id, meaning_ko, priority) -> AppResult<()> { ... }
    pub fn insert_example(&self, symbol_id, latex, explain, difficulty) -> AppResult<()> { ... }

    pub fn lookup_by_latex(&self, latex_command: &str) -> AppResult<Option<SymbolRow>> { ... }
    pub fn lookup_by_unicode(&self, ch: &str) -> AppResult<Option<SymbolRow>> { ... }
```

**분석 / Analysis**

- 모든 alias/link/example insert 는 `insert or ignore` — unique 제약(스키마의 unique 키들) 이 중복 차단. 정상.
- `lookup_by_*` 가 `OptionalExtension` 의 `optional()` 사용 → NotFound 가 Err 가 아닌 Ok(None). 호출자가 None 처리.

## 블록 4: symbol_search — 핵심 핫스팟 (lines 217-338)

```rust path=src-tauri/src/core/db.rs lines=217-228
    /// Hybrid search: exact match → FTS5 trigram → Levenshtein fallback.
    /// Results are deduplicated by symbol id and ordered by (exact, fts rank, edit distance).
    pub fn symbol_search(
        &self,
        q: &str,
        opts: Option<&SearchOptions>,
    ) -> AppResult<Vec<SymbolHit>> {
        let q = q.trim();
        if q.is_empty() {
            return Ok(vec![]);
        }
        let limit = opts.and_then(|o| o.limit).unwrap_or(30).max(1);
```

**분석 / Analysis**

- 빈 쿼리 즉시 종료.
- `limit.max(1)` — 0 또는 음수가 들어와도 최소 1.
- `opts.domain`, `opts.symbol_type` 는 받지만 **사용하지 않음** — 향후 필터링 자리. 현재는 데드 필드.

```rust path=src-tauri/src/core/db.rs lines=229-247
        let conn = self.lock();
        let q_lower = q.to_lowercase();
        let like = format!("%{q_lower}%");

        // 1) Exact / prefix hits — score 0 (highest priority).
        let mut exact: Vec<(i64, i64)> = conn
            .prepare(
                "select distinct s.id, 0 as score
                 from symbols s
                 left join symbol_aliases a on a.symbol_id = s.id
                 where s.unicode_char = ?1
                    or lower(s.canonical_name_ko) = ?2
                    or lower(coalesce(s.canonical_name_en,'')) = ?2
                    or lower(coalesce(s.latex_command,'')) = ?2
                    or lower(coalesce(a.alias_text,'')) = ?2",
            )?
            .query_map(params![q, q_lower], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<Result<_, _>>()?;
```

**분석 / Analysis**

- `?1` 가 unicode_char (대소문자 보존), `?2` 가 lowercase (이름/latex/alias).
- left join → distinct — 한 symbol 이 여러 별칭으로 매치돼도 한 번만.
- *흔한 실수*: `?1` 과 `?2` 가 `params![q, q_lower]` 순서로 바인딩. positional. 정확.

```rust path=src-tauri/src/core/db.rs lines=249-261
        // 2) Substring (LIKE) hits — score 1.
        let mut like_hits: Vec<(i64, i64)> = conn
            .prepare(
                "select distinct s.id, 1 as score
                 from symbols s
                 left join symbol_aliases a on a.symbol_id = s.id
                 where lower(s.canonical_name_ko) like ?1
                    or lower(coalesce(s.canonical_name_en,'')) like ?1
                    or lower(coalesce(s.latex_command,'')) like ?1
                    or lower(coalesce(a.alias_text,'')) like ?1",
            )?
            .query_map(params![like], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<Result<_, _>>()?;
```

**분석 / Analysis**

- `%q%` LIKE — 일반 substring. SQLite 의 LIKE 는 기본 case-insensitive(ASCII) 이지만 한국어/유니코드는 그렇지 않으므로 `lower()` 명시.
- 인덱스 없음(`%prefix%`) → full table scan. 사전 크기 ~수천이면 ms 단위.

```rust path=src-tauri/src/core/db.rs lines=262-274
        // 3) FTS5 trigram hits — score 2.
        let fts_hits: Vec<(i64, i64)> = if q.chars().count() >= 3 {
            // FTS5 quoted query — escape embedded quotes.
            let safe = q.replace('"', "\"\"");
            let match_query = format!("\"{safe}\"");
            conn.prepare(
                "select symbol_id, 2 from symbols_fts where text MATCH ?1 limit 200",
            )?
            .query_map(params![match_query], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<Result<_, _>>()?
        } else {
            Vec::new()
        };
```

**분석 / Analysis**

- 3자 이상에서만 FTS — trigram tokenizer 의 최소 가능 grams. *왜*: 2자 쿼리는 trigram 으로는 정확도 떨어지고 false positive 많음.
- **FTS5 쿼리 escape**: `"` → `""` 로 escape 후 `"..."` 안에 감쌈. SQL injection 은 placeholder 가 차단하지만, FTS5 의 매치 표현 문법은 `"`(phrase), `^`(prefix), `*` 등 특별 문자 — 사용자 입력이 그대로 들어가면 의도와 다른 결과. quoted phrase 로 감싸 안전.
- `limit 200` — 너무 많은 row 가 후속 머지 비용. trade-off.

```rust path=src-tauri/src/core/db.rs lines=276-287
        // 4) Levenshtein fuzzy — within edit distance threshold (depends on length).
        let max_dist = (q.chars().count() / 3 + 1) as i64;
        let lev_hits: Vec<(i64, i64)> = conn
            .prepare(
                "select s.id, 3
                 from symbols s
                 where levenshtein(lower(s.canonical_name_ko), ?1) <= ?2
                    or levenshtein(lower(coalesce(s.canonical_name_en,'')), ?1) <= ?2
                 limit 100",
            )?
            .query_map(params![q_lower, max_dist], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<Result<_, _>>()?;
```

**분석 / Analysis**

- `max_dist = len/3 + 1` — 짧은 쿼리에 1-2, 긴 쿼리에 더 관대. 합리적.
- **`levenshtein()` UDF 가 모든 행에 호출됨** — full table scan + per-row O(m·n). 큰 사전에서 비용. 다만 사전 크기가 작아 실용적.

```rust path=src-tauri/src/core/db.rs lines=289-313
        // Merge keeping minimum score per id.
        let mut by_id: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
        for (id, score) in exact.drain(..).chain(like_hits.drain(..))
            .chain(fts_hits.into_iter()).chain(lev_hits.into_iter())
        {
            by_id.entry(id).and_modify(|s| if score < *s { *s = score; }).or_insert(score);
        }
        if by_id.is_empty() { return Ok(vec![]); }
        let mut ranked: Vec<(i64, i64)> = by_id.into_iter().collect();
        ranked.sort_by_key(|(_, s)| *s);
        ranked.truncate(limit as usize);
```

**분석 / Analysis**

- 4개 후보군 union → 최소 score 유지 → score 오름차순 정렬 → limit truncate.
- HashMap 순서 비결정적 → 동점일 때 결과 순서 비결정적. 같은 쿼리 두 번에서 결과 순서가 다를 수 있음(테스트 시 주의).

```rust path=src-tauri/src/core/db.rs lines=315-338
        // Hydrate rows.
        let ids: Vec<i64> = ranked.iter().map(|(id, _)| *id).collect();
        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "select id, unicode_char, latex_command, canonical_name_ko,
                reading_ko, short_meaning_ko, symbol_type, null
             from symbols where id in ({placeholders})"
        );
        let mut stmt = conn.prepare(&sql)?;
        let params: Vec<&dyn rusqlite::ToSql> = ids.iter().map(|v| v as &dyn rusqlite::ToSql).collect();
        let mut rows: std::collections::HashMap<i64, SymbolHit> = stmt
            .query_map(rusqlite::params_from_iter(params), row_to_hit)?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|h| (h.id, h))
            .collect();
        Ok(ids.into_iter().filter_map(|id| rows.remove(&id)).collect())
    }
```

**분석 / Analysis**

- **동적 SQL 빌드** (`format!`) — placeholder 개수만 동적, 값은 여전히 prepared. SQL injection 안전.
- **결과 순서 보존**: HashMap 으로 hydrate 후 ids 순회로 추출 → 원래 ranked 순서 유지.
- `null` 8번째 컬럼 — `matched_alias` 자리. 매번 null 이라 row_to_hit 가 None 으로 받음. 향후 채울 자리.

## 블록 5: symbol_detail / examples (lines 340-438)

```rust path=src-tauri/src/core/db.rs lines=340-413
    pub fn symbol_detail(&self, id: i64) -> AppResult<SymbolDetail> {
        let conn = self.lock();
        let mut stmt = conn.prepare("select * from symbols where id = ?")?;
        let row = stmt.query_row(params![id], row_to_symbol).optional()?
            .ok_or_else(|| AppError::NotFound(format!("symbol#{id}")))?;

        let aliases: Vec<String> = ...;
        let domains: Vec<SymbolDomainMeaning> = ...;
        let examples: Vec<SymbolExample> = ...;
        let similar: Vec<SymbolHit> = ...;

        Ok(SymbolDetail { row, aliases, domains, examples, similar })
    }
```

**분석 / Analysis**

- 한 symbol detail = 5 쿼리(symbol, aliases, domains, examples, similar). 좋은 정상 패턴.
- `select *` — 컬럼 순서 의존. row_to_symbol 가 `r.get("name")` 으로 컬럼명 사용 → schema 변경에도 안전.
- `examples limit 5` — UI 노출 제한. `symbol_examples` 메서드는 limit 없음 (다른 API surface).
- `similar` 가 동일 `symbol_type` 5개 — 매우 단순한 유사도. 향후 cosine/embedding 기반 가능.

## 블록 6: history / favorites / notes (lines 440-556)

```rust path=src-tauri/src/core/db.rs lines=440-509
    pub fn save_history(&self, req: &AnalyzeRequest, result: &AnalyzeResult) -> AppResult<i64> {
        let conn = self.lock();
        conn.execute(
            "insert into expressions(
                input_format, raw_input, normalized_latex, normalized_mathml,
                speech_text_ko, rendered_ok, created_at
             ) values(?,?,?,?, ?, 1, strftime('%s','now')*1000)",
            params![req.format, req.raw_input, result.normalized_latex, result.normalized_mathml, result.speech.ko],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn history_list(&self, limit: i64) -> AppResult<Vec<HistoryItem>> { ... }
    pub fn favorites_add(&self, symbol_id: i64) -> AppResult<()> { ... }
    pub fn favorites_remove(&self, symbol_id: i64) -> AppResult<()> { ... }
    pub fn favorites_list(&self) -> AppResult<Vec<SymbolHit>> { ... }
```

**분석 / Analysis**

- `strftime('%s','now')*1000` — ms epoch. 한 줄 SQL 로 시각 생성.
- `rendered_ok` 가 항상 1 — 분석이 성공하면 history 에 저장하니까 의미는 있음. 그러나 컬럼 자체가 거의 데드 — 모든 row 가 1.
- favorites unique 제약(스키마)으로 중복 add 안전.

```rust path=src-tauri/src/core/db.rs lines=512-556
    pub fn notes_upsert(&self, note: Note) -> AppResult<Note> {
        let conn = self.lock();
        let tags_json = serde_json::to_string(&note.tags)?;
        if let Some(id) = note.id {
            conn.execute("update notes set ... where id = ?", ...)?;
            Ok(note)
        } else {
            conn.execute("insert into notes(...) values(...)", ...)?;
            let mut out = note;
            out.id = Some(conn.last_insert_rowid());
            Ok(out)
        }
    }

    pub fn notes_list(&self) -> AppResult<Vec<Note>> { ... }
```

**분석 / Analysis**

- tags 를 JSON string 컬럼에 저장. 정상.
- update 시 client 가 보낸 note 를 그대로 반환 — 서버 갱신값을 fetch 하지 않음. updated_at 새 값을 client 는 모름.
- list 의 tags 파싱: `serde_json::from_str(s).ok().unwrap_or_default()` — 손상된 JSON 은 빈 vec.

## 블록 7: settings KV + row converters (lines 558-616)

```rust path=src-tauri/src/core/db.rs lines=558-579
    pub fn settings_get(&self, key: &str) -> AppResult<Option<String>> { ... }
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

fn row_to_symbol(r: &Row) -> rusqlite::Result<SymbolRow> { ... }
fn row_to_hit(r: &Row) -> rusqlite::Result<SymbolHit> { ... }
```

**분석 / Analysis**

- `insert ... on conflict do update` — UPSERT. SQLite 3.24+.
- `row_to_symbol` 가 `r.get("name")` named — schema 컬럼 순서 변경에 안전.
- `row_to_hit` 가 `r.get(0)`..`r.get(7)` positional — 호출 SQL 의 컬럼 순서에 강결합. 변경 시 두 쪽 동기 필요.

## 블록 8: 마이그레이션 SQL + UDF (lines 618-773)

```rust path=src-tauri/src/core/db.rs lines=618-720
const MIGRATION_V1: &str = r#"
create table if not exists domains ( ... );
create table if not exists symbols ( ... );
create table if not exists symbol_aliases ( ...
    unique(symbol_id, alias_text, alias_type, locale)
);
create table if not exists symbol_domain_map ( ...
    unique(symbol_id, domain_id)
);
create table if not exists symbol_examples ( ... );
create table if not exists expressions ( ... );
create table if not exists favorites ( ...
    unique(symbol_id)
);
create table if not exists notes ( ... );
create table if not exists app_settings ( key text primary key, value text not null );

create index if not exists idx_symbols_unicode on symbols(unicode_char);
create index if not exists idx_symbols_latex on symbols(latex_command);
create index if not exists idx_symbols_type on symbols(symbol_type);
create index if not exists idx_aliases_text on symbol_aliases(alias_text);
"#;
```

**분석 / Analysis**

- **외래 키 명시** (`references symbols(id) on delete cascade`). 그러나 SQLite 에서 외래 키는 기본 비활성. `PRAGMA foreign_keys = ON` 호출이 안 보임 → **FK 제약이 실제 enforce 되지 않음**. cascade 가 동작하지 않을 가능성.
- 컬럼 default `created_at integer default (strftime('%s','now')*1000)` — 직접 insert 시 timestamp 자동.
- 인덱스 4개 — search 의 exact match 컬럼 + alias text.
- `unique(symbol_id, alias_text, alias_type, locale)` — 같은 (alias, type, locale) 조합 중복 차단. 그러나 같은 alias_text/locale 이라도 alias_type 만 다르면 중복 가능 — 사용자가 동일 텍스트를 여러 카테고리로 등록 가능.

```rust path=src-tauri/src/core/db.rs lines=722-731
/// FTS5 trigram virtual table for fuzzy search on Korean / Latin text.
const MIGRATION_V2_FTS: &str = r#"
create virtual table if not exists symbols_fts using fts5(
    symbol_id UNINDEXED,
    text,
    tokenize = 'trigram'
);
"#;
```

**분석 / Analysis**

- `tokenize = 'trigram'` — SQLite 3.34+ 의 trigram tokenizer. 한국어 자소 분리 안 함(자모 단위 X) — UTF-8 trigram 으로 동작. 한글 검색 효과는 어휘 차원에서 OK.
- `UNINDEXED` — symbol_id 는 FTS5 색인에서 제외. 단순 외래 키 역할.

```rust path=src-tauri/src/core/db.rs lines=733-746
fn register_levenshtein(conn: &Connection) -> AppResult<()> {
    conn.create_scalar_function(
        "levenshtein",
        2,
        FunctionFlags::SQLITE_DETERMINISTIC | FunctionFlags::SQLITE_UTF8,
        |ctx| {
            let a: String = ctx.get(0)?;
            let b: String = ctx.get(1)?;
            Ok(levenshtein(&a, &b) as i64)
        },
    )?;
    Ok(())
}
```

**분석 / Analysis**

- UDF 등록. `SQLITE_DETERMINISTIC` — 같은 입력에 같은 출력 보장 → SQLite 가 일부 최적화 가능(인덱스 사용은 어차피 불가).
- `SQLITE_UTF8` — 텍스트 인코딩 힌트.
- arity=2 고정.

```rust path=src-tauri/src/core/db.rs lines=748-773
/// Iterative O(m·n) edit distance over chars (handles Unicode/Hangul jamo).
fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let m = a.len();
    let n = b.len();
    if m == 0 { return n; }
    if n == 0 { return m; }
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0usize; n + 1];
    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1)
                .min(curr[j - 1] + 1)
                .min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}
```

**분석 / Analysis**

- **표준 동적 계획법 levenshtein**, O(m·n) 시간, O(n) 공간 (row swap).
- `chars()` 단위 — Hangul precomposed 문자는 한 char. NFC 정규화 없이 비교하므로 자모 분리된 입력은 다른 char 시퀀스로 인식. *흔한 실수*: `\u{AC00}` (한글) 입력은 `unicode-normalization` 없이도 보통 NFC 형태로 옴 → 문제 없음. 그러나 사용자 입력이 NFD 일 경우 분리. `unicode-normalization` 가 Cargo.toml 에 있지만 사용 안 됨(데드 의존성).
- *흔한 실수*: `usize::min` 체이닝 — 세 후보 중 최소. Rust 의 `.min()` 메서드 체인. 정확.

**짚어볼 점 / Watch-outs**
- 🔴 **FK 제약 enforce 안 됨** — `PRAGMA foreign_keys = ON` 누락.
- 🟡 **`opts.domain` / `opts.symbol_type` 미사용** — UI 가 보내도 결과 영향 없음.
- 🟡 **검색 4단 쿼리가 단일 Mutex 락 보유** — 락 시간 길어짐.
- 🟡 **`rendered_ok` 항상 1** — 컬럼 데드.
- 🟡 **trigram tokenizer 가 한글 자소 분리 안 함** — 검색 품질에 영향. 운영 데이터로 확인 필요.
- 🟡 **마이그레이션 버전 추적 부재** — V3 이상 추가 시 비-idempotent 변경은 별도 분기 필요.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 명확한 스키마, FTS5 + Levenshtein 하이브리드 검색, 시그니처 일관.
- 위험: FK 비활성, 데드 옵션, 4단 쿼리 락 보유, 마이그레이션 트래킹 부재.
- 개선 권고:
  1. `Db::open` 에서 `conn.execute_batch("PRAGMA foreign_keys = ON;")`.
  2. `symbol_search` 를 단일 CTE 로 통합(`db_pg::symbol_search` 의 패턴 차용).
  3. `opts.domain`/`opts.symbol_type` 필터 구현.
  4. `user_version` PRAGMA + 마이그레이션 순차 적용.
- 재읽기 권고 레벨: 3 — 검색 핵심 로직과 스키마는 중급 이상이 다시 봐야 함.
