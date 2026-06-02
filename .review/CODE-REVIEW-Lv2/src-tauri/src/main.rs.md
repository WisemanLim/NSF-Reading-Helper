# src-tauri/src/main.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 바이너리 진입점. Windows 릴리스 빌드에서 콘솔 창 숨김.
- LOC: 5
- 주요 export: `fn main()`
- 의존 모듈: `nsf_reading_helper_lib::run` (현 크레이트의 lib crate type)
- 외부 패키지: 없음
- 부수효과: lib::run() 호출
- 동시성: 없음

---

```rust path=src-tauri/src/main.rs lines=1-5
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    nsf_reading_helper_lib::run();
}
```

**분석 / Analysis**

- **`#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`** — 릴리스 빌드(`debug_assertions` 가 off)에서만 `windows_subsystem = "windows"` 어트리뷰트 적용. Windows GUI 앱이 검은 콘솔 창 없이 시작되도록 함. dev 빌드에선 콘솔 출력 보기 위해 유지.
- `nsf_reading_helper_lib::run()` — 본 크레이트의 `lib.rs::run` 호출. `Cargo.toml` 의 `[lib] name="nsf_reading_helper_lib"`, `crate-type=["staticlib","cdylib","rlib"]` 와 매칭.
- **왜 lib + bin 분리**: Tauri 의 모바일 타깃(`tauri::mobile_entry_point`)이 `lib.rs::run` 을 인식해야 하기 때문. iOS/Android 빌드는 main.rs 를 사용하지 않고 lib 의 `run` 진입점만 사용.

**짚어볼 점 / Watch-outs**
- 5줄이지만 빌드 시스템과 강한 결합. crate name 변경 시 이 한 줄이 깨짐.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 최소 표면, 모바일/데스크톱 양립 구조.
- 위험: crate name 의 magic string.
- 개선 권고: 없음 — 표준 Tauri 2 패턴.
- 재읽기 권고 레벨: 0.
