import { invoke } from "@tauri-apps/api/core";
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

export interface TtsOptions {
  rate: number;
  lang: string;
}
export interface SearchOptions {
  domain?: string;
  symbolType?: string;
  limit?: number;
}

export const tauriIpc = {
  analyze: (input: AnalyzeRequest) =>
    invoke<AnalyzeResult>("analyze", { input }),

  symbolSearch: (q: string, opts?: SearchOptions) =>
    invoke<SymbolHit[]>("symbol_search", { q, opts: opts ?? null }),
  symbolGet: (id: number) => invoke<SymbolDetail>("symbol_get", { id }),
  symbolExamples: (id: number) =>
    invoke<SymbolExample[]>("symbol_examples", { id }),

  historyList: () => invoke<HistoryItem[]>("history_list"),
  favoritesAdd: (symbolId: number) =>
    invoke<void>("favorites_add", { symbolId }),
  favoritesRemove: (symbolId: number) =>
    invoke<void>("favorites_remove", { symbolId }),
  favoritesList: () => invoke<SymbolHit[]>("favorites_list"),
  notesUpsert: (n: Note) => invoke<Note>("notes_upsert", { note: n }),
  notesList: () => invoke<Note[]>("notes_list"),

  settingsGet: () => invoke<Settings>("settings_get"),
  settingsSet: (s: Settings) =>
    invoke<Settings>("settings_set", { settings: s }),

  ttsSpeak: (text: string, opts: TtsOptions) =>
    invoke<void>("tts_speak", { text, opts }),
  ttsStop: () => invoke<void>("tts_stop"),
};
