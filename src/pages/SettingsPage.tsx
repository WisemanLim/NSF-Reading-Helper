import { useState } from "react";
import { isTauri } from "../lib/env";
import { checkForUpdate, downloadAndInstall, type UpdateInfo } from "../lib/updater";
import { useAppStore } from "../store/useAppStore";
import { ipc } from "../ipc/client";

export function SettingsPage() {
  const settings = useAppStore((s) => s.settings);
  const save = useAppStore((s) => s.saveSettings);

  const [update, setUpdate] = useState<UpdateInfo | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);

  const onCheckUpdate = async () => {
    setBusy("update");
    setStatus(null);
    try {
      const r = await checkForUpdate();
      setUpdate(r);
      setStatus(
        r.available
          ? `새 버전 ${r.version} 사용 가능`
          : "최신 버전입니다",
      );
    } catch (e) {
      setStatus(`업데이트 확인 실패: ${e instanceof Error ? e.message : String(e)}`);
    } finally {
      setBusy(null);
    }
  };

  const onInstall = async () => {
    setBusy("install");
    setStatus("다운로드/설치 중...");
    try {
      await downloadAndInstall();
      setStatus("설치 완료. 앱을 재시작하세요.");
    } catch (e) {
      setStatus(`설치 실패: ${e instanceof Error ? e.message : String(e)}`);
    } finally {
      setBusy(null);
    }
  };

  const onImportPack = async () => {
    setBusy("import");
    setStatus(null);
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const picked = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "NSF Pack", extensions: ["nsfpack", "zip"] }],
      });
      if (!picked) {
        setBusy(null);
        return;
      }
      const path =
        typeof picked === "string"
          ? picked
          : (picked as { path: string }).path;
      const r = await ipc.packImport(path);
      setStatus(
        `[${r.manifest.name} v${r.manifest.version}] 가져옴: 기호 ${r.insertedSymbols}개 추가, ${r.skippedSymbols}개 중복`,
      );
    } catch (e) {
      setStatus(`가져오기 실패: ${e instanceof Error ? e.message : String(e)}`);
    } finally {
      setBusy(null);
    }
  };

  const onExportPack = async () => {
    setBusy("export");
    setStatus(null);
    try {
      const { save: saveDialog } = await import("@tauri-apps/plugin-dialog");
      const target = await saveDialog({
        defaultPath: `nsf-${Date.now()}.nsfpack`,
        filters: [{ name: "NSF Pack", extensions: ["nsfpack"] }],
      });
      if (!target) {
        setBusy(null);
        return;
      }
      const m = await ipc.packExport(target, "user-export");
      setStatus(`내보내기 완료: 기호 ${m.symbolCount}개, 도메인 ${m.domainCount}개`);
    } catch (e) {
      setStatus(`내보내기 실패: ${e instanceof Error ? e.message : String(e)}`);
    } finally {
      setBusy(null);
    }
  };

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <header className="flex h-14 shrink-0 items-center border-b border-line px-6">
        <h1 className="text-sm font-semibold text-fg-muted">설정</h1>
      </header>
      <div className="mx-auto flex w-full max-w-2xl flex-1 flex-col gap-6 overflow-y-auto px-6 py-6 text-sm">
        <section className="flex flex-col gap-3">
          <h2 className="text-xs font-semibold uppercase tracking-wide text-fg-muted">
            읽기 및 음성
          </h2>
          <Field label="읽기 규칙">
            <select
              className="input-base"
              value={settings.speechRule}
              onChange={(e) =>
                save({ speechRule: e.target.value as "clearspeak" | "mathspeak" })
              }
            >
              <option value="clearspeak">Clearspeak (자연스러움)</option>
              <option value="mathspeak">Mathspeak (구조 명시)</option>
            </select>
          </Field>
          <Field label="읽기 언어">
            <select
              className="input-base"
              value={settings.speechLocale}
              onChange={(e) =>
                save({ speechLocale: e.target.value as "ko" | "en" })
              }
            >
              <option value="ko">한국어</option>
              <option value="en">English</option>
            </select>
          </Field>
          <Field label={`TTS 속도 (${settings.ttsRate.toFixed(1)}×)`}>
            <input
              type="range"
              min={0.5}
              max={2.0}
              step={0.1}
              value={settings.ttsRate}
              onChange={(e) => save({ ttsRate: parseFloat(e.target.value) })}
              className="w-full accent-brand"
              aria-label="TTS 속도"
            />
          </Field>
          <Field label="UI 언어">
            <select
              className="input-base"
              value={settings.uiLocale}
              onChange={(e) => save({ uiLocale: e.target.value as "ko" | "en" })}
            >
              <option value="ko">한국어</option>
              <option value="en">English</option>
            </select>
          </Field>
        </section>

        {isTauri && (
          <>
            <section className="flex flex-col gap-3">
              <h2 className="text-xs font-semibold uppercase tracking-wide text-fg-muted">
                업데이트
              </h2>
              <div className="flex flex-wrap gap-2">
                <button
                  className="btn-outline"
                  onClick={onCheckUpdate}
                  disabled={!!busy}
                >
                  {busy === "update" ? "확인 중..." : "업데이트 확인"}
                </button>
                {update?.available && (
                  <button
                    className="btn-primary"
                    onClick={onInstall}
                    disabled={!!busy}
                  >
                    {busy === "install"
                      ? "설치 중..."
                      : `v${update.version} 설치`}
                  </button>
                )}
              </div>
            </section>

            <section className="flex flex-col gap-3">
              <h2 className="text-xs font-semibold uppercase tracking-wide text-fg-muted">
                기호 사전 팩 (.nsfpack)
              </h2>
              <div className="flex flex-wrap gap-2">
                <button
                  className="btn-outline"
                  onClick={onImportPack}
                  disabled={!!busy}
                >
                  {busy === "import" ? "가져오는 중..." : "팩 가져오기"}
                </button>
                <button
                  className="btn-outline"
                  onClick={onExportPack}
                  disabled={!!busy}
                >
                  {busy === "export" ? "내보내는 중..." : "팩 내보내기"}
                </button>
              </div>
              <p className="text-xs text-fg-subtle">
                .nsfpack 은 manifest.json + symbols.json 을 담은 zip 입니다.
                기존 기호와 중복되면 건너뜁니다.
              </p>
            </section>
          </>
        )}

        {status && (
          <div
            role="status"
            aria-live="polite"
            className="rounded-lg border border-line bg-bg-elevated px-3 py-2 text-xs text-fg-muted"
          >
            {status}
          </div>
        )}
      </div>
    </div>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label className="flex flex-col gap-1">
      <span className="text-xs font-medium text-fg-muted">{label}</span>
      {children}
    </label>
  );
}
