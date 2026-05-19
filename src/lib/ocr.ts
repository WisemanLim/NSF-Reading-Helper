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
