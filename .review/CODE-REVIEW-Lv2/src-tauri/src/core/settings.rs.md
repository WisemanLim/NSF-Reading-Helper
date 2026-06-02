# src-tauri/src/core/settings.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: `Settings` JSON 직렬화/역직렬화로 `app_settings` 테이블의 `user_settings_v1` 키에 영속.
- LOC: 17
- 주요 export: `load(db)`, `save(db, settings)`
- 의존 모듈: `core::db`, `core::error`, `core::models`
- 외부 패키지: `serde_json`

---

```rust path=src-tauri/src/core/settings.rs lines=1-17
use crate::core::db::Db;
use crate::core::error::AppResult;
use crate::core::models::Settings;

const KEY: &str = "user_settings_v1";

pub fn load(db: &Db) -> AppResult<Settings> {
    match db.settings_get(KEY)? {
        Some(s) => Ok(serde_json::from_str(&s).unwrap_or_default()),
        None => Ok(Settings::default()),
    }
}

pub fn save(db: &Db, settings: &Settings) -> AppResult<()> {
    let s = serde_json::to_string(settings)?;
    db.settings_set(KEY, &s)
}
```

**분석 / Analysis**

- **버전 접미사 `v1`** — 스키마 변경 시 v2 로 옮기고 마이그레이션 분기 가능. 좋은 미래 대비.
- `load`:
  - 키 없음 → `Settings::default()`.
  - 키 있는데 JSON 파싱 실패 → `unwrap_or_default()` → 기본값. *짚어볼 점*: 사용자가 변경한 설정이 깨졌을 때 silently 초기화. 로그도 없음.
- `save`: 단순 serialize → key/value insert(upsert).
- `db.settings_set` 가 `INSERT ... ON CONFLICT(key) DO UPDATE` 사용 → 안전.

**짚어볼 점 / Watch-outs**
- 파싱 실패 silent reset. `if let Err(e) = serde_json::from_str(...)` 로 로그 권장.
- `Settings` 의 신규 필드가 추가되면 기존 v1 JSON 에 missing → serde 기본값(`#[serde(default)]`)이 채움. 현재 `domains_enabled: Vec<String>` 만 `#[serde(default)]` 명시. 다른 필드는 누락 시 deserialize 실패 → unwrap_or_default 로 통째로 초기화. **사실상 모든 필드 누락 = 전체 리셋**.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 작고 명료, 키 버전 관리.
- 위험: silent reset.
- 개선 권고:
  1. 모든 `Settings` 필드에 `#[serde(default)]` 추가하여 부분 마이그레이션 허용.
  2. 파싱 실패 시 `tracing::warn!`.
- 재읽기 권고 레벨: 1.
