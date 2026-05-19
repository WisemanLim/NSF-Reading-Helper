import { create } from "zustand";
import { ipc } from "../ipc/client";
import type {
  AnalyzeResult,
  AnalyzeToken,
  HistoryItem,
  Settings,
  SymbolDetail,
} from "../types";

const DEFAULT_SETTINGS: Settings = {
  speechRule: "clearspeak",
  speechLocale: "ko",
  ttsRate: 1.0,
  ttsVoice: null,
  uiLocale: "ko",
  domainsEnabled: [],
};

interface AppState {
  loaded: boolean;
  settings: Settings;
  history: HistoryItem[];
  currentInput: string;
  currentResult: AnalyzeResult | null;
  selectedToken: AnalyzeToken | null;
  selectedSymbol: SymbolDetail | null;
  analyzing: boolean;

  bootstrap: () => Promise<void>;
  saveSettings: (s: Partial<Settings>) => Promise<void>;
  setInput: (v: string) => void;
  setResult: (r: AnalyzeResult | null) => void;
  selectToken: (t: AnalyzeToken | null) => Promise<void>;
  setAnalyzing: (b: boolean) => void;
  reloadHistory: () => Promise<void>;
}

export const useAppStore = create<AppState>()((set, get) => ({
  loaded: false,
  settings: DEFAULT_SETTINGS,
  history: [],
  currentInput: "",
  currentResult: null,
  selectedToken: null,
  selectedSymbol: null,
  analyzing: false,

  async bootstrap() {
    try {
      const settings = await ipc.settingsGet();
      const history = await ipc.historyList().catch(() => []);
      set({ settings: { ...DEFAULT_SETTINGS, ...settings }, history, loaded: true });
    } catch (e) {
      console.warn("bootstrap fell back to defaults", e);
      set({ loaded: true });
    }
  },

  async saveSettings(patch) {
    const next = { ...get().settings, ...patch };
    const saved = await ipc.settingsSet(next);
    set({ settings: saved ?? next });
  },

  setInput(v) {
    set({ currentInput: v });
  },
  setResult(r) {
    set({ currentResult: r, selectedToken: null, selectedSymbol: null });
  },

  async selectToken(t) {
    set({ selectedToken: t, selectedSymbol: null });
    if (!t?.symbolId) return;
    try {
      const detail = await ipc.symbolGet(t.symbolId);
      if (get().selectedToken?.symbolId === t.symbolId) {
        set({ selectedSymbol: detail });
      }
    } catch (e) {
      console.warn("symbol lookup failed", e);
    }
  },

  setAnalyzing(b) {
    set({ analyzing: b });
  },

  async reloadHistory() {
    const history = await ipc.historyList().catch(() => []);
    set({ history });
  },
}));
