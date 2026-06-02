# src-tauri/src/core/mod.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: core 서브모듈 인덱스. 외부 크레이트(`web-server`)가 `#[path]` 별칭으로 재사용할 수 있도록 `pub`.
- LOC: 8
- 주요 export: 8개 서브모듈
- 의존 모듈: 없음
- 외부 패키지: 없음
- 부수효과: 없음

---

```rust path=src-tauri/src/core/mod.rs lines=1-8
pub mod analyzer;
pub mod db;
pub mod error;
pub mod models;
pub mod pack;
pub mod seed;
pub mod settings;
pub mod tts;
```

**분석 / Analysis**

- 모두 `pub` — `web-server/src/core.rs` 가 `#[path="../../src-tauri/src/core/db.rs"] pub mod db` 식으로 끌어가는데, 별칭이 아닌 직접 컴파일이므로 visibility 가 일치해야 함. 정확.
- 알파벳 순 — 사소한 일관성.

**짚어볼 점 / Watch-outs**
- 없음. 신규 모듈 추가 시 이 목록과 `core.rs` (web-server)의 매핑 양쪽 갱신.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 단순, 알파벳 정렬.
- 위험: 없음.
- 개선 권고: 없음.
- 재읽기 권고 레벨: 0.
