import { useAppStore } from "../store/useAppStore";

export function SettingsPage() {
  const settings = useAppStore((s) => s.settings);
  const save = useAppStore((s) => s.saveSettings);

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <header className="flex h-14 shrink-0 items-center border-b border-line px-6">
        <h1 className="text-sm font-semibold text-fg-muted">설정</h1>
      </header>
      <div className="mx-auto flex w-full max-w-2xl flex-1 flex-col gap-4 overflow-y-auto px-6 py-6 text-sm">
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
