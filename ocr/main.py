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


@asynccontextmanager
async def lifespan(_app: FastAPI):
    if os.getenv("PRELOAD_MODEL", "0") == "1":
        _load_model()
    yield


app = FastAPI(title="NSF OCR Sidecar", lifespan=lifespan)
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_methods=["*"],
    allow_headers=["*"],
)


class OcrResult(BaseModel):
    latex: str
    confidence: Optional[float] = None
    width: Optional[int] = None
    height: Optional[int] = None


@app.get("/healthz")
def healthz() -> dict:
    return {"ok": True}


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
