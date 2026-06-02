# src-tauri/src/core/error.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 단일 에러 enum `AppError` 와 `AppResult<T>` 정의. Tauri IPC 직렬화 가능 (`Serialize` impl 수동).
- LOC: 44
- 주요 export: `AppError`, `AppResult<T>`
- 외부 패키지: `thiserror`, `serde`, `anyhow`, `rusqlite`, `serde_json`
- 부수효과: 없음

---

```rust path=src-tauri/src/core/error.rs lines=1-22
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("{0}")]
    Generic(String),
}

pub type AppResult<T> = Result<T, AppError>;
```

**분석 / Analysis**

- `thiserror::Error` 가 `Display`/`Source` 자동 구현. `#[error("...")]` 가 `Display` 메시지 템플릿.
- `#[from]` 가 `From<rusqlite::Error>`, `From<std::io::Error>`, `From<serde_json::Error>` 를 자동 구현 → `?` 연산자로 자연 변환.
- `NotFound(String)` — 명시적 not-found 케이스. `Db::symbol_detail` 등에서 사용. web-server `main.rs` 의 `ApiError::into_response` 가 이를 404 매핑.
- `Generic(String)` — fallback. anyhow 또는 ad-hoc String 에러를 흡수.

```rust path=src-tauri/src/core/error.rs lines=24-29
impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}
```

**분석 / Analysis**

- **수동 Serialize** — *왜*: thiserror 의 자동 derive 는 enum 변형을 객체로 직렬화. Tauri JS 측은 단순 문자열 메시지를 기대 → `Display` 결과를 string 으로 보냄.
- `JS 측에서 catch(e)` 의 `e` 가 plain string. 좋은 UX.

```rust path=src-tauri/src/core/error.rs lines=31-44
impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Generic(e.to_string())
    }
}
impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Generic(s)
    }
}
impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Generic(s.to_string())
    }
}
```

**분석 / Analysis**

- 추가 From 임플:
  - `anyhow::Error` — 일부 라이브러리가 anyhow 를 반환할 때 통합.
  - `String` / `&str` — `Err("message".into())` 또는 `Err(format!(...).into())` 작성을 허용.
- `anyhow` 의 변환은 정보 손실(체인 모두 to_string으로 평탄화). 디버깅 시 원본 백트레이스 잃음.

**짚어볼 점 / Watch-outs**
- `Generic(String)` 가 광범위 — 구체 분기가 누락되면 UX 가 모호.
- `anyhow` 변환이 정보 손실.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 작고 표준적, IPC 직렬화 명료.
- 위험: 정보 손실 변환.
- 개선 권고:
  1. Validation 에러용 별도 variant 추가(`Validation(String)`) — 404 vs 400 구분.
  2. anyhow 변환에서 `format!("{e:#}")` 사용해 체인 보존.
- 재읽기 권고 레벨: 1.
