# src/lib/ocr.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: OCR 사이드카(pix2tex) HTTP 클라이언트.
- LOC: 43
- 주요 export: `ocrAvailable`, `ocrImage`, `OcrResult`
- 의존 모듈: `./env`
- 외부 패키지: 없음 (Fetch)
- 부수효과: 네트워크 I/O (`/ocr/healthz`, `/ocr/predict`)

---

```ts path=src/lib/ocr.ts lines=1-13
// Image → LaTeX via the optional pix2tex sidecar.
// The sidecar is reached via the same-origin `/ocr/` path (nginx proxies it),
// or a custom URL via VITE_NSF_OCR_URL for local dev / Tauri.

import { apiBaseUrl } from "./env";

function ocrBaseUrl(): string {
  const env = (import.meta as unknown as { env?: Record<string, string> }).env;
  const explicit = env?.VITE_NSF_OCR_URL;
  if (explicit) return explicit.replace(/\/$/, "");
  // Default — same origin, nginx proxies /ocr/ to the sidecar.
  return `${apiBaseUrl()}/ocr`;
}
```

**분석 / Analysis**

- `VITE_NSF_OCR_URL` 명시 시 override (예: 로컬 dev 가 `http://localhost:8000`).
- `replace(/\/$/, "")` — trailing slash 제거. 아래서 `${...}/healthz` 와 `${...}/ocr/predict` 를 안전 결합.
- `apiBaseUrl()` 가 빈 문자열이면 결과는 `/ocr` — 동일-출처 상대 경로. nginx 가 `/ocr/` 를 사이드카로 프록시.

```ts path=src/lib/ocr.ts lines=15-30
export interface OcrResult {
  latex: string;
  confidence?: number | null;
  width?: number | null;
  height?: number | null;
}

export async function ocrAvailable(): Promise<boolean> {
  try {
    const r = await fetch(`${ocrBaseUrl()}/healthz`, { method: "GET" });
    return r.ok;
  } catch {
    return false;
  }
}
```

**분석 / Analysis**

- `OcrResult` 는 백엔드 FastAPI 의 응답 모델(`ocr/main.py::OcrResult`) 미러.
- `ocrAvailable` 가 사이드카 헬스체크 — 비동기 호출 실패도 false. UI 가 "OCR 비활성" 토글 가능.

```ts path=src/lib/ocr.ts lines=31-43
export async function ocrImage(file: File): Promise<OcrResult> {
  const form = new FormData();
  form.append("file", file, file.name || "formula.png");
  const r = await fetch(`${ocrBaseUrl()}/ocr/predict`, {
    method: "POST",
    body: form,
  });
  if (!r.ok) {
    const detail = await r.text().catch(() => r.statusText);
    throw new Error(`OCR 실패 (${r.status}): ${detail}`);
  }
  return (await r.json()) as OcrResult;
}
```

**분석 / Analysis**

- `FormData` + 파일 — Fetch 가 자동으로 multipart 헤더 + boundary 설정. *흔한 실수*: 수동으로 `Content-Type: multipart/form-data` 헤더 추가 시 boundary 누락으로 서버가 파싱 실패. 여기서는 명시하지 않아 OK.
- `file.name || "formula.png"` — 파일명이 빈 경우 기본값. 일부 클립보드 paste 가 파일명을 비울 수 있음.
- 에러 메시지에 status + detail 모두 — UI 에 그대로 노출 가능한 한국어 prefix.
- `as OcrResult` — 응답 검증 없음.

**짚어볼 점 / Watch-outs**
- HTTPS/http 혼합 환경: `VITE_NSF_OCR_URL` 가 http 인데 사이트가 https 면 mixed-content 차단. 환경 변수 사용자 책임.
- `OcrDropzone.tsx` 가 호출 위치. 파일 크기 한계는 nginx + FastAPI 측 정책에 의존.
- `confidence` 가 응답에 실제론 채워지지 않음(FastAPI 코드 참조) — UI 가 옵셔널 처리 잘하면 무해.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 간결, 헬스체크 별도, 에러 메시지 사용자 친화.
- 위험: 응답 검증 부재.
- 개선 권고:
  1. zod 등으로 `OcrResult` 검증.
  2. AbortController 로 타임아웃 처리.
  3. 파일 크기 사전 검증(예: 5MB 이상 차단).
- 재읽기 권고 레벨: 1.
