# ocr/ — 디렉토리 개요 / Directory Abstract

> [← INDEX.md](../INDEX.md)

## 파일 목록 / File List

| 파일 / File | LOC | 역할 / Role | 리뷰 |
|---|---|---|---|
| main.py | 88 | FastAPI `/healthz` + `/ocr/predict` | [main.py.md](main.py.md) |
| requirements.txt | — | CPU-only torch + pix2tex 핀 | (manifest) |

## 디렉토리 구조 / Structure Summary

`pix2tex` (LatexOCR by Lukas Blecher)를 감싼 작은 FastAPI 사이드카. 첫 요청에 모델을 lazy-load 하고, `PRELOAD_MODEL=1` 환경변수로 lifespan에서 강제 로드할 수 있습니다. nginx (`/ocr/` 경로) 가 프록시.

## 주요 의존 관계 / Key Dependencies

- 외부 / External: FastAPI, Pillow, pix2tex (런타임 import만), torch (CPU wheel).
- 들어오는 호출 / Inbound: 프런트엔드 `src/lib/ocr.ts` 가 `${apiBaseUrl()}/ocr/...` 로 호출. 데스크톱은 이 사이드카를 호출하지 않는 게 기본.

## 위험 신호 / Risk Signals

1. `CORSMiddleware` 가 모든 origin/method/header 허용 — credentials 없는 multipart 업로드라 위험은 제한적.
2. `confidence` 필드는 응답 모델엔 있으나 `pix2tex.cli.LatexOCR.__call__` 의 반환은 LaTeX 문자열 단일 — 현재 코드는 신뢰도를 항상 미설정. 명세와 실데이터의 미세한 어긋남.
3. `_MODEL` 이 글로벌 가변 상태이고 첫 호출자만 비용 부담 — 멀티 워커(`uvicorn --workers N`) 시 각 워커마다 별도 로드, 메모리 사용 N배.
