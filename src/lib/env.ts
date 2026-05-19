declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

export const isTauri =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export function apiBaseUrl(): string {
  const env = (import.meta as unknown as { env?: Record<string, string> }).env;
  return env?.VITE_NSF_API_BASE ?? "";
}
