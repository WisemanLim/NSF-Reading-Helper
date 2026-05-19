// Tauri auto-update wrapper. No-op on web — the SettingsPage check returns
// "manual" and the updater button is hidden.

import { isTauri } from "./env";

export interface UpdateInfo {
  available: boolean;
  version?: string;
  currentVersion?: string;
  notes?: string;
}

export async function checkForUpdate(): Promise<UpdateInfo> {
  if (!isTauri) {
    return { available: false };
  }
  try {
    const mod = await import("@tauri-apps/plugin-updater");
    const update = await mod.check();
    if (!update) {
      return { available: false };
    }
    return {
      available: true,
      version: update.version,
      currentVersion: update.currentVersion,
      notes: update.body ?? undefined,
    };
  } catch (e) {
    console.warn("update check failed", e);
    return { available: false };
  }
}

export async function downloadAndInstall(): Promise<void> {
  if (!isTauri) return;
  const mod = await import("@tauri-apps/plugin-updater");
  const update = await mod.check();
  if (!update) return;
  await update.downloadAndInstall();
  // After install Tauri prompts to restart; let the user trigger it.
}
