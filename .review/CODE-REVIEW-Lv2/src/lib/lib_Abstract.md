# src/lib/ — 디렉토리 개요 / Directory Abstract

> [← INDEX.md](../../INDEX.md)

## 파일 목록 / File List

| 파일 / File | LOC | 역할 / Role | 리뷰 링크 |
|---|---|---|---|
| env.ts | 13 | `isTauri`, `apiBaseUrl()` | [env.ts.md](env.ts.md) |
| detectFormat.ts | 13 | 입력 포맷 추정 휴리스틱 | [detectFormat.ts.md](detectFormat.ts.md) |
| sre.ts | 45 | Speech Rule Engine 래퍼 | [sre.ts.md](sre.ts.md) |
| mathjax.ts | 67 | MathJax v3 CDN 로더 | [mathjax.ts.md](mathjax.ts.md) |
| koSpeak.ts | 115 | 한국어 음성 파이프라인 (template→SRE→tokens) | [koSpeak.ts.md](koSpeak.ts.md) |
| tts.ts | 27 | Web Speech / Tauri TTS 분기 | [tts.ts.md](tts.ts.md) |
| ocr.ts | 43 | pix2tex 사이드카 클라이언트 | [ocr.ts.md](ocr.ts.md) |
| detexify.ts | 108 | Detexify 분류기 (override→proxy→public) | [detexify.ts.md](detexify.ts.md) |
| updater.ts | 42 | Tauri updater 호출 (no-op on web) | [updater.ts.md](updater.ts.md) |
| cn.ts | 6 | `clsx + tailwind-merge` | (미선택 / deferred) |
| ko-clearspeak.json | — | 한국어 규칙 팩 데이터 | (데이터 — 미선택) |

## 디렉토리 구조 / Structure Summary

`lib/` 는 **부수효과가 일정 범위로 격리된 순수 모듈**의 집합입니다. React 의존성이 없고, 컴포넌트에서 빌려 쓰기 좋은 함수 단위로 모여 있습니다.

세 가지 클러스터:
1. **환경 추상화** — `env.ts`, `cn.ts`.
2. **수식 도메인** — `detectFormat`, `mathjax`, `sre`, `koSpeak`, `tts` — 입력 식을 표현·발화하는 파이프라인.
3. **외부 서비스** — `ocr` (pix2tex), `detexify` (handwriting), `updater` (Tauri). 모두 동적 import 또는 fetch.

## 주요 의존 관계 / Key Dependencies

- `koSpeak` → `sre` + `ko-clearspeak.json`
- `tts` → `ipc/client` (Tauri 분기)
- `updater` → `@tauri-apps/plugin-updater` (동적 import)
- `ocr`, `detexify` → `env.apiBaseUrl()`
- `mathjax` → `cdn.jsdelivr.net/npm/mathjax@3` (런타임 외부 의존)

## 위험 신호 / Risk Signals

1. `detexify` 가 동일-출처 프록시 실패 시 외부 호스트로 사용자 손그림 데이터를 자동 전송 — 개인정보 측면 검토 필요.
2. `mathjax` 가 CDN 로드 실패 시 fallback 없음 — 오프라인/사내망에서 차단.
3. `koSpeak.tryTemplate` 가 `RegExp(rule.pattern)` 으로 JSON 데이터의 정규식을 그대로 컴파일 — 손상된 패턴이 런타임 오류로 떨어짐(런타임에 try 안에서 처리되긴 하나, 매 요청마다 컴파일).
