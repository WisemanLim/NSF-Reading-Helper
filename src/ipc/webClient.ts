import type {
  AnalyzeRequest,
  AnalyzeResult,
  HistoryItem,
  Note,
  Settings,
  SymbolDetail,
  SymbolExample,
  SymbolHit,
} from "../types";
import { apiBaseUrl } from "../lib/env";
import type { SearchOptions, TtsOptions } from "./tauriClient";

const base = () => apiBaseUrl();

async function jget<T>(path: string): Promise<T> {
  const r = await fetch(`${base()}/api${path}`);
  if (!r.ok) throw new Error(await r.text().catch(() => r.statusText));
  return (await r.json()) as T;
}
async function jsend<T>(
  method: string,
  path: string,
  body?: unknown,
): Promise<T | void> {
  const r = await fetch(`${base()}/api${path}`, {
    method,
    headers: body !== undefined ? { "content-type": "application/json" } : {},
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });
  if (!r.ok) throw new Error(await r.text().catch(() => r.statusText));
  if (r.status === 204) return;
  const ct = r.headers.get("content-type") ?? "";
  if (!ct.includes("application/json")) return;
  return (await r.json()) as T;
}

function qs(opts?: SearchOptions): string {
  if (!opts) return "";
  const p = new URLSearchParams();
  if (opts.domain) p.set("domain", opts.domain);
  if (opts.symbolType) p.set("symbolType", opts.symbolType);
  if (opts.limit) p.set("limit", String(opts.limit));
  const s = p.toString();
  return s ? `&${s}` : "";
}

export const webIpc = {
  analyze: (input: AnalyzeRequest) =>
    jsend<AnalyzeResult>("POST", "/expressions/analyze", input) as Promise<AnalyzeResult>,

  symbolSearch: (q: string, opts?: SearchOptions) =>
    jget<SymbolHit[]>(`/symbols/search?q=${encodeURIComponent(q)}${qs(opts)}`),
  symbolGet: (id: number) => jget<SymbolDetail>(`/symbols/${id}`),
  symbolExamples: (id: number) => jget<SymbolExample[]>(`/symbols/${id}/examples`),

  historyList: () => jget<HistoryItem[]>("/history"),
  favoritesAdd: async (symbolId: number): Promise<void> => {
    await jsend("POST", "/favorites", { symbolId });
  },
  favoritesRemove: async (symbolId: number): Promise<void> => {
    await jsend("DELETE", `/favorites/${symbolId}`);
  },
  favoritesList: () => jget<SymbolHit[]>("/favorites"),
  notesUpsert: (n: Note) =>
    jsend<Note>("POST", "/notes", n) as Promise<Note>,
  notesList: () => jget<Note[]>("/notes"),

  settingsGet: () => jget<Settings>("/settings"),
  settingsSet: (s: Settings) =>
    jsend<Settings>("PUT", "/settings", s) as Promise<Settings>,

  ttsSpeak: async (_text: string, _opts: TtsOptions): Promise<void> => {
    /* browser uses speechSynthesis directly in lib/tts.ts */
  },
  ttsStop: async (): Promise<void> => {
    /* same as above */
  },
};
