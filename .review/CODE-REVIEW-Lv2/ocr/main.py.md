# ocr/main.py — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: pix2tex(LatexOCR) 를 감싼 작은 FastAPI 사이드카. `POST /ocr/predict` 가 이미지를 받아 LaTeX 반환.
- LOC: 88
- 주요 export: `app` (FastAPI 인스턴스)
- 외부 패키지: `fastapi`, `Pillow`, `pix2tex`(런타임 lazy import), `pydantic`
- 부수효과: 모델 lazy load (큰 메모리), HTTP 응답 생성, 이미지 디코드
- 동시성: ASGI(uvicorn) — 단일 모델 글로벌 변수 공유

---

```python path=ocr/main.py lines=1-20
"""NSF Reading Helper — OCR sidecar.

Tiny FastAPI service that wraps `pix2tex` (LaTeX-OCR by Lukas Blecher) so the
frontend can drop an image of a formula and get LaTeX back. Runs CPU-only by
default; switch to a CUDA-enabled torch image for GPU speedup.

POST /ocr/predict
    multipart field `file`  — image (PNG/JPG/PDF page render)
    -> { "latex": "...", "confidence": 0.0..1.0 }
"""

from __future__ import annotations

import io
import logging
import os
from contextlib import asynccontextmanager
from typing import Optional

from fastapi import FastAPI, File, HTTPException, UploadFile
from fastapi.middleware.cors import CORSMiddleware
from PIL import Image
from pydantic import BaseModel
```

**분석 / Analysis**

- `from __future__ import annotations` — 모든 어노테이션을 string 으로(PEP 563). 순환 import / typing.Optional 단순화에 유리.
- `io`, `logging`, `os` — stdlib.
- `contextlib.asynccontextmanager` — FastAPI lifespan 정의에 필요.
- `Optional` typing import — `from __future__` 가 있어 사실상 string 형식만 필요하지만 명시.

```python path=ocr/main.py lines=22-37
log = logging.getLogger("nsf.ocr")
logging.basicConfig(level=os.getenv("LOG_LEVEL", "INFO"))

_MODEL = None  # lazily initialized; first request pays the warmup cost


def _load_model():
    global _MODEL
    if _MODEL is not None:
        return _MODEL
    log.info("loading pix2tex model (first request)...")
    # Import lazily so the container can start even if pix2tex install is broken.
    from pix2tex.cli import LatexOCR  # type: ignore
    _MODEL = LatexOCR()
    log.info("pix2tex model ready")
    return _MODEL
```

**분석 / Analysis**

- `LOG_LEVEL` env — 운영자가 조정. 기본 INFO.
- **`_MODEL` 글로벌 가변 상태** — 단일 worker 에선 OK. **uvicorn 의 `--workers N` 가 여러 프로세스로 동작** → 각 워커가 자체 `_MODEL` 보유 → 메모리 N배. *왜 lazy*: 컨테이너 시작 속도 우선, 첫 요청자가 비용 부담.
- `_MODEL is not None` 체크 후 import — *왜*: pix2tex 가 망가졌어도 healthz 는 응답 가능. 진단 친화.
- *흔한 실수*: 글로벌 가변 + lazy init 에 대한 동시성 보호 없음. 첫 요청 두 개가 동시에 도착하면 `LatexOCR()` 가 두 번 호출되어 두 인스턴스 생성, 메모리 낭비. ASGI 가 single-threaded coroutine 처리라 `_load_model` 이 await 없이 단번에 실행되므로 사실상 race 안전. 그러나 `_load_model` 이 import + LatexOCR() 둘 다 blocking — async 함수가 아닌 sync 함수 안에서 GIL 보호 하에 한 번에 실행 → 안전.
- 정확히 따져보면: `_load_model` 자체는 sync. `predict` 가 `await model = _load_model()` 같이 호출하지 않고 `model = _load_model()` 직접 호출 → blocking call 동안 다른 코루틴은 양보 불가 → 단일 인스턴스 보장.

```python path=ocr/main.py lines=40-46
@asynccontextmanager
async def lifespan(_app: FastAPI):
    if os.getenv("PRELOAD_MODEL", "0") == "1":
        _load_model()
    yield
```

**분석 / Analysis**

- FastAPI 의 `lifespan` 컨텍스트 — 시작/종료 hook.
- `PRELOAD_MODEL=1` 환경에선 시작 시 모델 로드(첫 요청 latency 제거). production 권장.
- 종료 정리 없음 — `_MODEL` 의 메모리는 프로세스 종료 시 OS 가 회수.

```python path=ocr/main.py lines=48-55
app = FastAPI(title="NSF OCR Sidecar", lifespan=lifespan)
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_methods=["*"],
    allow_headers=["*"],
)
```

**분석 / Analysis**

- CORS Any — credentials 미사용이라 위험 제한.
- nginx 가 프록시 앞단이라 사실상 외부에 노출되지 않을 수도 있음(배포 토폴로지에 따라).

```python path=ocr/main.py lines=58-67
class OcrResult(BaseModel):
    latex: str
    confidence: Optional[float] = None
    width: Optional[int] = None
    height: Optional[int] = None


@app.get("/healthz")
def healthz() -> dict:
    return {"ok": True}
```

**분석 / Analysis**

- Pydantic 응답 모델 — FastAPI 가 자동 검증·문서화.
- `confidence` 는 응답에 정의되어 있으나 `predict` 가 채우지 않음 — JS-side 가 `null` 받음. UI 가 옵셔널 처리 잘하면 무해.
- `healthz` 는 sync `def` (async 아님). FastAPI 가 threadpool 에서 실행 — 짧은 응답이라 무관.

```python path=ocr/main.py lines=69-88
@app.post("/ocr/predict", response_model=OcrResult)
async def predict(file: UploadFile = File(...)) -> OcrResult:
    if not file.content_type or not file.content_type.startswith("image/"):
        raise HTTPException(status_code=400, detail="expected an image upload")
    data = await file.read()
    if not data:
        raise HTTPException(status_code=400, detail="empty file")
    try:
        img = Image.open(io.BytesIO(data)).convert("RGB")
    except Exception as exc:
        raise HTTPException(status_code=400, detail=f"invalid image: {exc}")
    model = _load_model()
    try:
        latex = model(img)
    except Exception as exc:
        log.exception("pix2tex inference failed")
        raise HTTPException(status_code=500, detail=str(exc))
    return OcrResult(latex=latex, width=img.width, height=img.height)
```

**분석 / Analysis**

- **content-type 검증**: `image/` 접두사. 클라이언트 신뢰. 진짜 검증은 Pillow 의 open 시점.
- **`await file.read()`** — 전체를 메모리 로드. *짚어볼 점*: 매우 큰 이미지 업로드 시 메모리 폭증. FastAPI 의 `UploadFile.spool` 이 일부 디스크 사용 가능. 추가로 application-level size limit 권장.
- **Pillow `Image.open` + `convert("RGB")`** — pix2tex 가 RGB 텐서 기대. PNG alpha/grayscale 자동 변환. 좋은 정규화.
- **`model(img)`** — pix2tex 호출. CPU torch 라 느림 (수 초). FastAPI 가 sync 호출을 thread pool 이 아닌 직접 코루틴 내 실행 → **이 동안 다른 요청이 블로킹**. uvicorn workers > 1 이거나 `run_in_executor` 권장.
- `log.exception` — 스택 트레이스까지 로그. 디버깅 친화.
- `HTTPException(status_code=500, detail=str(exc))` — 에러 메시지에 내부 정보 노출 가능(pix2tex 의 detail 메시지). 운영 보안 검토.

**짚어볼 점 / Watch-outs**
- 🟡 **단일 워커 + sync model() 호출** → 동시 요청 직렬 처리. `run_in_executor` 또는 multi-worker 필요.
- 🟡 **요청 크기 제한 없음** — 매우 큰 이미지 OOM 가능.
- 🟡 **에러 detail 누출**.
- 🟡 `confidence` 가 응답 모델엔 있으나 실제 미설정.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: lazy import, content-type 가드, RGB 정규화, lifespan preload.
- 위험: 동시성 직렬화, 메모리/크기 미제한.
- 개선 권고:
  1. `latex = await asyncio.get_running_loop().run_in_executor(None, model, img)` 로 sync inference 를 풀 throttle.
  2. `request.headers.get("content-length")` 또는 `MAX_FILE_SIZE` 검증.
  3. `confidence` 필드 제거 또는 실제 측정.
  4. 운영 에러 메시지는 generic, 상세는 log 만.
- 재읽기 권고 레벨: 2.
