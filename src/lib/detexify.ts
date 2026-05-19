// Detexify-compatible classifier client. Tries (in order):
//   1. VITE_NSF_DETEXIFY_URL (custom — useful for self-hosted)
//   2. Same-origin /api/handwriting/classify (backend proxy)
//   3. https://detexify.kirelabs.org/classify.json (CORS-permitting public host)
//
// Detexify's public API expects:
//   POST { "strokes": [[{x,y},...], ...] }  // application/json
// and replies with an array of { score: number, symbol: { command, package, ... } }.
// We normalize that into a flat ClassifyHit shape.

import { apiBaseUrl } from "./env";

export interface ClassifyHit {
  command: string;
  package?: string | null;
  score: number;
  id?: string;
}

interface Point {
  x: number;
  y: number;
}

function detexifyOverrideUrl(): string | null {
  const env = (import.meta as unknown as { env?: Record<string, string> }).env;
  return env?.VITE_NSF_DETEXIFY_URL || null;
}

function backendProxyUrl(): string {
  return `${apiBaseUrl()}/api/handwriting/classify`;
}

const PUBLIC_DETEXIFY = "https://detexify.kirelabs.org/classify.json";

async function postJson(url: string, body: unknown): Promise<unknown> {
  const r = await fetch(url, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!r.ok) throw new Error(`HTTP ${r.status}: ${r.statusText}`);
  return r.json();
}

function normalize(raw: unknown): ClassifyHit[] {
  if (!Array.isArray(raw)) return [];
  const hits: ClassifyHit[] = [];
  for (const item of raw) {
    if (!item || typeof item !== "object") continue;
    const obj = item as Record<string, unknown>;
    const score = typeof obj.score === "number" ? obj.score : null;
    const sym = (obj.symbol ?? obj) as Record<string, unknown>;
    const cmd =
      typeof sym.command === "string"
        ? sym.command
        : typeof sym.id === "string"
          ? sym.id
          : null;
    if (cmd === null || score === null) continue;
    hits.push({
      command: cmd.startsWith("\\") ? cmd : `\\${cmd}`,
      package:
        typeof sym.package === "string" ? (sym.package as string) : undefined,
      score: Math.max(0, Math.min(1, score)),
      id: typeof sym.id === "string" ? (sym.id as string) : undefined,
    });
  }
  // Lower score = better in original Detexify (distance). Normalize so we can
  // sort by ascending score then surface as confidence = 1 - normalized.
  hits.sort((a, b) => a.score - b.score);
  const maxScore = hits[hits.length - 1]?.score ?? 1;
  const minScore = hits[0]?.score ?? 0;
  const span = Math.max(0.0001, maxScore - minScore);
  return hits.map((h) => ({
    ...h,
    score: 1 - (h.score - minScore) / span,
  }));
}

export async function classifyStrokes(
  strokes: Point[][],
): Promise<ClassifyHit[]> {
  if (strokes.length === 0) return [];
  const body = { strokes };

  const override = detexifyOverrideUrl();
  if (override) {
    return normalize(await postJson(override, body));
  }

  // Try same-origin proxy first (avoids CORS, single failure mode).
  try {
    const r = await postJson(backendProxyUrl(), body);
    return normalize(r);
  } catch (proxyErr) {
    // Fall back to public Detexify directly.
    try {
      return normalize(await postJson(PUBLIC_DETEXIFY, body));
    } catch (publicErr) {
      throw new Error(
        `Detexify 호출 실패 — proxy: ${
          proxyErr instanceof Error ? proxyErr.message : String(proxyErr)
        }; public: ${publicErr instanceof Error ? publicErr.message : String(publicErr)}`,
      );
    }
  }
}
