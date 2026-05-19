import { isTauri } from "../lib/env";
import { tauriIpc } from "./tauriClient";
import { webIpc } from "./webClient";

export const ipc = isTauri ? tauriIpc : webIpc;
export type { SearchOptions, TtsOptions } from "./tauriClient";
