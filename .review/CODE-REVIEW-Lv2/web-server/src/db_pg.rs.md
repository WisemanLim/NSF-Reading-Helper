# web-server/src/db_pg.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: PostgreSQL 백엔드. SQLite `core::db::Db` 의 API 표면을 그대로 미러링하고 native async + deadpool.
- LOC: **615** (블록 모드)
- 주요 export: `PgDb`
- 의존 모듈: `core::error`, `core::models`
- 외부 패키지: `deadpool-postgres`, `tokio-postgres`, `serde_json`
- 부수효과: Postgres I/O
- 동시성: native async, 16-connection pool
- 테스트: 없음

> **블록-모드 리뷰**: 615 > 600.

---

## 블록 1: 모듈 헤더 + connect / migrate / seed (lines 1-67)

```rust path=web-server/src/db_pg.rs lines=1-22
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
```

**분석 / Analysis**

- `NoTls` — **TLS 없이 평문 통신**. 운영 환경에선 `tokio-postgres-rustls` 또는 `tokio-postgres-native-tls` 로 교체 필요. 도커 내부 망에서는 OK 일 수 있음.
- `MIGRATION_SQL` 가 한 파일 — 모든 스키마/인덱스/extension(`pg_trgm`, `fuzzystrmatch`) 적재. **현재 마이그레이션은 first-time 모드만 지원** (한 번 실행하면 끝). 향후 변경은 별도 도구(예: refinery, sqlx-cli) 필요.

```rust path=web-server/src/db_pg.rs lines=24-54
impl PgDb {
    pub async fn connect(url: &str) -> AppResult<Self> {
        let pg_config = Config::from_str(url)
            .map_err(|e| AppError::Generic(format!("postgres URL parse: {e}")))?;
        let mgr_config = ManagerConfig { recycling_method: RecyclingMethod::Fast };
        let mgr = Manager::from_config(pg_config, NoTls, mgr_config);
        let pool = Pool::builder(mgr).max_size(16).build()
            .map_err(|e| AppError::Generic(format!("pool build: {e}")))?;
        let db = Self { pool };
        db.migrate().await?;
        Ok(db)
    }

    async fn migrate(&self) -> AppResult<()> {
        let client = self.client().await?;
        client.batch_execute(MIGRATION_SQL).await
            .map_err(|e| AppError::Generic(format!("migrate: {e}")))?;
        Ok(())
    }

    async fn client(&self) -> AppResult<deadpool_postgres::Client> {
        self.pool.get().await.map_err(|e| AppError::Generic(format!("pool: {e}")))
    }
```

**분석 / Analysis**

- `max_size(16)` — 풀 크기. 작은 서버용. 트래픽 많으면 부족.
- `RecyclingMethod::Fast` — 클라이언트 재사용 시 health check 안 함. 빠르지만 죽은 연결 감지 부족.
- `batch_execute` — 다중 SQL 문 한 번에. 마이그레이션 SQL 의 모든 `create extension`, `create table` 한 번에 적용.

```rust path=web-server/src/db_pg.rs lines=56-67
    pub async fn seed_if_empty(&self) -> AppResult<()> {
        let client = self.client().await?;
        let row = client
            .query_one("select count(*)::bigint from symbols", &[])
            .await
            .map_err(map_pg)?;
        let n: i64 = row.get(0);
        if n > 0 { return Ok(()); }
        crate::seed_pg::install(self).await
    }
```

**분석 / Analysis**

- `count(*)::bigint` — Postgres 의 count는 `numeric` 이라 `i64` 로 받으려면 cast 필요. 안전.
- 비어있을 때만 seed_pg::install 호출.

## 블록 2: 시드용 INSERT 메서드들 (lines 69-198)

```rust path=web-server/src/db_pg.rs lines=71-91
    pub async fn insert_domain(&self, code: &str, name_ko: &str, name_en: Option<&str>) -> AppResult<i64> {
        let client = self.client().await?;
        client.execute(
            "insert into domains(code, name_ko, name_en) values($1,$2,$3)
             on conflict (code) do nothing",
            &[&code, &name_ko, &name_en],
        ).await.map_err(map_pg)?;
        let row = client.query_one("select id from domains where code = $1", &[&code]).await.map_err(map_pg)?;
        Ok(row.get(0))
    }
```

**분석 / Analysis**

- `$1, $2, ...` — Postgres 의 numbered placeholders. SQLite 의 `?` 와 다른 문법. 컴파일 타임 검증 없음.
- 2 round-trip (insert + select). 더 효율적: `insert ... on conflict do nothing returning id` + select fallback.

```rust path=web-server/src/db_pg.rs lines=93-141
    #[allow(clippy::too_many_arguments)]
    pub async fn insert_symbol(
        &self,
        unicode_char: Option<&str>, latex_command: Option<&str>, mathml_tag: Option<&str>,
        canonical_name_ko: &str, canonical_name_en: Option<&str>,
        reading_ko: Option<&str>, reading_en: Option<&str>,
        short_meaning_ko: &str, short_meaning_en: Option<&str>,
        symbol_type: &str, difficulty_level: i16,
        is_operator: bool, is_relation: bool, is_delimiter: bool,
        is_greek: bool, is_constant: bool, is_function: bool,
    ) -> AppResult<i64> {
        let client = self.client().await?;
        let row = client.query_one(
            "insert into symbols(...) values($1,$2,...,$17) returning id",
            &[
                &unicode_char, &latex_command, &mathml_tag,
                ...
                &is_greek, &is_constant, &is_function,
            ],
        ).await.map_err(map_pg)?;
        Ok(row.get(0))
    }
```

**분석 / Analysis**

- **17개 인자 — clippy 무시 어트리뷰트**. SQLite 의 `Db::insert_symbol(&seed::SymbolSeed<'_>)` 와 시그니처 비대칭. SQLite 는 한 struct, PG 는 17 args.
- *왜 다른가*: `seed::SymbolSeed<'_>` 가 `core` 모듈(SQLite 측) 에 정의됨. web-server 가 import 가능하긴 하지만(같은 `core::seed` 별칭), 여기선 단순 18개 인자로 풀어 씀 — 일관성 부족.
- `difficulty_level: i16` — Postgres 컬럼이 `smallint` 라 추정. SQLite 측은 `i64`. **타입 비대칭**.

```rust path=web-server/src/db_pg.rs lines=143-198
    pub async fn insert_alias(...) -> AppResult<()> { ... on conflict do nothing ... }
    pub async fn link_domain(...) -> AppResult<()> { ... on conflict do nothing ... }
    pub async fn insert_example(...) -> AppResult<()> { ... }
```

**분석 / Analysis**

- 모두 `on conflict do nothing` 으로 idempotency. SQLite 의 `insert or ignore` 와 동등.

## 블록 3: lookup_by_* (lines 200-225)

```rust path=web-server/src/db_pg.rs lines=200-225
    pub async fn lookup_by_latex(&self, latex: &str) -> AppResult<Option<SymbolRow>> {
        let client = self.client().await?;
        let row = client.query_opt(
            "select * from symbols where latex_command = $1 limit 1", &[&latex],
        ).await.map_err(map_pg)?;
        Ok(row.map(row_to_symbol))
    }

    pub async fn lookup_by_unicode(&self, ch: &str) -> AppResult<Option<SymbolRow>> { ... }
```

**분석 / Analysis**

- `query_opt` — 0 또는 1 row. SQLite 의 `OptionalExtension::optional()` 과 동등.
- `select *` — `row_to_symbol` 가 컬럼명으로 추출. 컬럼 순서 무관.

## 블록 4: symbol_search — 단일 CTE (lines 226-302)

```rust path=web-server/src/db_pg.rs lines=226-285
    pub async fn symbol_search(&self, q: &str, opts: Option<&SearchOptions>) -> AppResult<Vec<SymbolHit>> {
        let q = q.trim();
        if q.is_empty() { return Ok(vec![]); }
        let limit = opts.and_then(|o| o.limit).unwrap_or(30).max(1);
        let client = self.client().await?;
        let q_lower = q.to_lowercase();
        let like = format!("%{q_lower}%");

        // Single query: union exact / substring / trigram / Levenshtein scores.
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
            &[&q, &q_lower, &like, &((q.chars().count() as i32 / 3) + 1), &(limit as i64)],
        ).await.map_err(map_pg)?;
```

**분석 / Analysis**

- **단일 CTE** — SQLite 의 4-쿼리 union 보다 깔끔. *왜 PG 에서 가능*: pg_trgm 의 `similarity()` 와 fuzzystrmatch 의 `levenshtein()` 가 모두 SQL 함수로 노출.
- CASE 우선순위: exact (0) → substring (1) → trigram > 0.3 (2) → levenshtein (3).
- **alias 매칭** WHERE 절에 있지만 score 0/1/2 모두 alias 를 직접 매칭하지 않고 substring 만 — 즉 alias 가 exact match 여도 1점(substring)으로 떨어짐. 데스크톱 SQLite 의 exact alias 도 0점 분기와 **결과 불일치 가능**.
- *흔한 실수*: subscription pg_trgm 의 `similarity` 가 한국어 trigram 에서 무의미할 수 있음 — pg_trgm 은 ASCII 위주이고 한국어 trigram 효과는 검증 필요(실험적).
- `(q.chars().count() as i32 / 3) + 1` — i32 로 cast. PG 의 levenshtein 2번째 인자가 i32 인 듯. 데스크톱은 i64 — 미세 비대칭.
- `where score < 9` — `else 9` 가 의도된 dummy. 매치 안 되는 후보 제거.

```rust path=web-server/src/db_pg.rs lines=287-302
        if rows.is_empty() { return Ok(vec![]); }
        let ids: Vec<i64> = rows.iter().map(|r| r.get::<_, i64>(0)).collect();
        let hits = client.query(
            "select id, unicode_char, latex_command, canonical_name_ko,
                    reading_ko, short_meaning_ko, symbol_type
             from symbols where id = any($1::bigint[])",
            &[&ids],
        ).await.map_err(map_pg)?;
        let mut by_id: std::collections::HashMap<i64, SymbolHit> =
            hits.into_iter().map(|r| (r.get(0), row_to_hit(r))).collect();
        Ok(ids.into_iter().filter_map(|id| by_id.remove(&id)).collect())
    }
```

**분석 / Analysis**

- **`= any($1::bigint[])`** — Postgres 의 배열 binding. `IN (...)` 의 동적 placeholder 회피.
- 결과 순서: ranked id 순서. 좋음.

## 블록 5: symbol_detail / examples / save_history / history_list (lines 304-441)

```rust path=web-server/src/db_pg.rs lines=304-366
    pub async fn symbol_detail(&self, id: i64) -> AppResult<SymbolDetail> {
        let client = self.client().await?;
        let row = client.query_opt("select * from symbols where id = $1", &[&id])
            .await.map_err(map_pg)?
            .ok_or_else(|| AppError::NotFound(format!("symbol#{id}")))?;
        let symbol = row_to_symbol(row);

        let aliases: Vec<String> = client.query(...).await...into_iter().map(...).collect();
        let domains: Vec<SymbolDomainMeaning> = client.query(...).await...;
        let examples = self.symbol_examples(id).await?;
        let similar: Vec<SymbolHit> = client.query(...).await...;

        Ok(SymbolDetail { row: symbol, aliases, domains, examples, similar })
    }
```

**분석 / Analysis**

- 5 round-trip. SQLite 와 동일 구조.
- `examples` 만 `self.symbol_examples(id).await` 재사용 — 다른 4개도 헬퍼로 분리하면 일관.

```rust path=web-server/src/db_pg.rs lines=368-394
    pub async fn symbol_examples(&self, id: i64) -> AppResult<Vec<SymbolExample>> {
        ...
        Ok(rows.into_iter().map(|r| SymbolExample {
            id: r.get(0),
            symbol_id: r.get(1),
            domain_code: r.get(2),
            example_latex: r.get(3),
            example_reading_ko: r.get(4),
            example_explanation_ko: r.get(5),
            difficulty_level: r.get::<_, i16>(6) as i64,
        }).collect())
    }
```

**분석 / Analysis**

- `difficulty_level: r.get::<_, i16>(6) as i64` — PG smallint → SymbolExample.difficulty_level i64. **i16 → i64 cast 안전**(upcast).

```rust path=web-server/src/db_pg.rs lines=396-441
    pub async fn save_history(&self, req, result) -> AppResult<i64> {
        let row = client.query_one(
            "insert into expressions(...) values($1,$2,$3,$4, $5, true) returning id",
            &[&req.format, &req.raw_input, ...]).await.map_err(map_pg)?;
        Ok(row.get(0))
    }

    pub async fn history_list(&self, limit: i64) -> AppResult<Vec<HistoryItem>> { ... }
```

**분석 / Analysis**

- `rendered_ok` 를 `true` 명시. SQLite 는 `1`.

## 블록 6: favorites + notes (lines 443-545)

```rust path=web-server/src/db_pg.rs lines=443-477
    pub async fn favorites_add(&self, symbol_id: i64) -> AppResult<()> {
        client.execute("insert into favorites(symbol_id) values($1) on conflict do nothing", &[&symbol_id]).await...;
        Ok(())
    }
    pub async fn favorites_remove(...) -> AppResult<()> { ... delete ... }
    pub async fn favorites_list(&self) -> AppResult<Vec<SymbolHit>> { ... }
```

**분석 / Analysis**

- favorites unique 제약은 마이그레이션 SQL 에 존재해야 함 — 본 리뷰 범위 밖.

```rust path=web-server/src/db_pg.rs lines=479-545
    pub async fn notes_upsert(&self, note: Note) -> AppResult<Note> {
        let tags_json = serde_json::to_string(&note.tags)?;
        if let Some(id) = note.id {
            client.execute(
                "update notes set expression_id=$2, symbol_id=$3,
                    body_md=$4, tags=$5,
                    updated_at = (extract(epoch from now()) * 1000)::bigint
                 where id = $1",
                &[&id, &note.expression_id, &note.symbol_id, &note.body_md, &tags_json],
            ).await.map_err(map_pg)?;
            Ok(note)
        } else {
            let row = client.query_one(
                "insert into notes(expression_id, symbol_id, body_md, tags)
                 values($1,$2,$3,$4) returning id",
                &[&note.expression_id, &note.symbol_id, &note.body_md, &tags_json],
            ).await.map_err(map_pg)?;
            let mut out = note;
            out.id = Some(row.get(0));
            Ok(out)
        }
    }

    pub async fn notes_list(&self) -> AppResult<Vec<Note>> { ... }
```

**분석 / Analysis**

- `(extract(epoch from now()) * 1000)::bigint` — ms epoch (SQLite 의 `strftime('%s','now')*1000` 와 동등).
- `tags` 컬럼 타입이 무엇인지 명확 안 함(text 또는 jsonb). 둘 다 작동하지만 jsonb 라면 indexing 가능.

## 블록 7: settings + rebuild_fts (lines 547-574)

```rust path=web-server/src/db_pg.rs lines=547-573
    pub async fn settings_get(&self, key: &str) -> AppResult<Option<String>> {
        let row = client.query_opt("select value from app_settings where key = $1", &[&key]).await.map_err(map_pg)?;
        Ok(row.map(|r| r.get::<_, String>(0)))
    }

    pub async fn settings_set(&self, key: &str, value: &str) -> AppResult<()> {
        client.execute(
            "insert into app_settings(key,value) values($1,$2)
             on conflict (key) do update set value = excluded.value",
            &[&key, &value],
        ).await.map_err(map_pg)?;
        Ok(())
    }

    pub async fn rebuild_fts(&self) -> AppResult<()> {
        // Postgres uses GIN trigram indexes on the live columns — nothing to rebuild.
        Ok(())
    }
}
```

**분석 / Analysis**

- `rebuild_fts` 는 no-op — Postgres 가 GIN/GIST trigram 인덱스(`pg_trgm`)를 행 변경 시 자동 갱신. SQLite 의 외부 FTS5 테이블 모델과 차이.

## 블록 8: 변환 헬퍼 (lines 576-616)

```rust path=web-server/src/db_pg.rs lines=576-616
fn map_pg(e: tokio_postgres::Error) -> AppError {
    AppError::Generic(format!("postgres: {e}"))
}

fn row_to_symbol(r: Row) -> SymbolRow {
    SymbolRow {
        id: r.get("id"),
        ...
        difficulty_level: r.get::<_, i16>("difficulty_level") as i64,
        is_operator: r.get("is_operator"),
        ...
    }
}

fn row_to_hit(r: Row) -> SymbolHit {
    SymbolHit {
        id: r.get(0), unicode_char: r.get(1), latex_command: r.get(2),
        canonical_name_ko: r.get(3), reading_ko: r.get(4),
        short_meaning_ko: r.get(5), symbol_type: r.get(6),
        matched_alias: None,
    }
}
```

**분석 / Analysis**

- `map_pg` — 모든 PG 에러를 `Generic` 으로 매핑. **변형 손실**: PG specific 에러 코드(constraint violation 등) 가 분기되지 않음. 향후 unique violation → 409 매핑이 필요해지면 보강.
- `row_to_symbol` — 컬럼명 추출. `difficulty_level` 만 i16→i64 명시.
- `row_to_hit` — positional. SQL `select id, unicode_char, ..., symbol_type` 7 컬럼 순서에 강결합.

**짚어볼 점 / Watch-outs**
- 🟡 **TLS 없음** (`NoTls`) — 운영에서 보강 필요.
- 🟡 **마이그레이션 추적 없음** — 한 번 적용 후 변경 어려움.
- 🟡 **insert_symbol 17-인자** — SQLite 측과 시그니처 비대칭.
- 🟡 **alias exact 매칭이 substring 으로 격하** — SQLite 와 결과 불일치.
- 🟡 **map_pg 의 정보 손실**.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 단일 CTE 검색, on conflict 사용, async 친화.
- 위험: TLS 미사용, 마이그레이션 도구 부재, 시그니처 비대칭.
- 개선 권고:
  1. `tokio-postgres-rustls` 통합, env 로 TLS 토글.
  2. `refinery` 또는 `sqlx-cli` 도입.
  3. `insert_symbol` 을 `&seed::SymbolSeed` 받도록 통일.
  4. CTE 의 alias 매칭을 score 0 분기에도 포함.
  5. `map_pg` 가 unique-violation 을 별도 variant 로.
- 재읽기 권고 레벨: 3 — Postgres 마이그레이션·TLS·검색 정합성 모두 중급+ 점검.
