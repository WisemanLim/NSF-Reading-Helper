# src/pages/SettingsPage.tsx — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 사용자 설정 폼 + Tauri 전용 섹션 (자동 업데이트 + .nsfpack 가져오기/내보내기).
- LOC: 232
- 주요 export: `SettingsPage`
- 의존 모듈: `../lib/env`, `../lib/updater`, `../store/useAppStore`, `../ipc/client`, `@tauri-apps/plugin-dialog`(dynamic)

---

```tsx path=src/pages/SettingsPage.tsx lines=1-13
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
```

**분석 / Analysis**

- `busy: string | null` — 현재 진행 중 작업명(`"update"`, `"install"`, `"import"`, `"export"`). 동시에 둘 이상 차단.
- `status: string | null` — 사용자 표시 메시지(성공/실패/진행).

```tsx path=src/pages/SettingsPage.tsx lines=15-31
  const onCheckUpdate = async () => {
    setBusy("update");
    setStatus(null);
    try {
      const r = await checkForUpdate();
      setUpdate(r);
      setStatus(
        r.available ? `새 버전 ${r.version} 사용 가능` : "최신 버전입니다",
      );
    } catch (e) {
      setStatus(`업데이트 확인 실패: ${e instanceof Error ? e.message : String(e)}`);
    } finally {
      setBusy(null);
    }
  };
```

**분석 / Analysis**

- `checkForUpdate` 자체는 throw 하지 않게 설계됨 (`updater.ts` 의 외부 catch). 즉 여기 catch 는 거의 도달 불능. 방어적 코드.

```tsx path=src/pages/SettingsPage.tsx lines=33-44
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
```

**분석 / Analysis**

- `downloadAndInstall` 은 `updater.ts` 가 throw 가능 — caller 가 catch 필요. 정확.

```tsx path=src/pages/SettingsPage.tsx lines=46-73
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
```

**분석 / Analysis**

- **동적 import** `@tauri-apps/plugin-dialog` — 웹 번들에서 제외. `isTauri` 가드와 함께 안전.
- `open({ multiple: false, directory: false, ... })` — Tauri 2 의 dialog plugin. 반환 타입은 `string | null` 인데 (v2), 본 코드는 `typeof picked === "string" ? picked : (picked as { path: string }).path` 분기를 둠 — v1 호환 흔적. **잠재 데드 분기** — 검증 후 단순화 가능.
- 폴백 분기에서 `setBusy(null)` 후 return → finally 의 `setBusy(null)` 가 또 호출되어 중복. 무해하지만 코드 노이즈.
- 결과 메시지에 manifest.name, version, 추가/중복 개수 표시 — 좋은 UX.

```tsx path=src/pages/SettingsPage.tsx lines=75-95
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
```

**분석 / Analysis**

- `saveDialog` (`save` 의 import 별칭) — `useAppStore.saveSettings` 와의 충돌을 피하려고 별명 사용. 명료.
- `defaultPath` 에 `Date.now()` 사용 — 충돌 방지. ms epoch 가 파일명에 들어가지만 사용자 친화.
- "user-export" 가 pack 의 manifest name 으로 들어감 → import 시 그 이름으로 alias prefix(`pack:user-export`).

```tsx path=src/pages/SettingsPage.tsx lines=97-152
  return (
    <div className="flex h-full flex-col overflow-hidden">
      <header className="flex h-14 shrink-0 items-center border-b border-line px-6">
        <h1 className="text-sm font-semibold text-fg-muted">설정</h1>
      </header>
      <div className="mx-auto flex w-full max-w-2xl flex-1 flex-col gap-6 overflow-y-auto px-6 py-6 text-sm">
        <section className="flex flex-col gap-3">
          <h2 className="text-xs font-semibold uppercase tracking-wide text-fg-muted">읽기 및 음성</h2>
          <Field label="읽기 규칙">
            <select className="input-base" value={settings.speechRule}
              onChange={(e) => save({ speechRule: e.target.value as "clearspeak" | "mathspeak" })}>
              <option value="clearspeak">Clearspeak (자연스러움)</option>
              <option value="mathspeak">Mathspeak (구조 명시)</option>
            </select>
          </Field>
          <Field label="읽기 언어">
            <select className="input-base" value={settings.speechLocale}
              onChange={(e) => save({ speechLocale: e.target.value as "ko" | "en" })}>
              <option value="ko">한국어</option>
              <option value="en">English</option>
            </select>
          </Field>
          <Field label={`TTS 속도 (${settings.ttsRate.toFixed(1)}×)`}>
            <input type="range" min={0.5} max={2.0} step={0.1}
              value={settings.ttsRate}
              onChange={(e) => save({ ttsRate: parseFloat(e.target.value) })}
              className="w-full accent-brand" aria-label="TTS 속도" />
          </Field>
          <Field label="UI 언어">
            <select className="input-base" value={settings.uiLocale}
              onChange={(e) => save({ uiLocale: e.target.value as "ko" | "en" })}>
              <option value="ko">한국어</option>
              <option value="en">English</option>
            </select>
          </Field>
        </section>
```

**분석 / Analysis**

- `as "clearspeak" | "mathspeak"` — `e.target.value: string` 을 union 으로 단언. select option 이 같은 값을 갖는다는 정적 보장.
- `save({...})` — Partial<Settings> 받아 부분 갱신. 한 필드 바꿔도 다른 값 유지.
- `ttsRate` 의 toFixed(1) — 1.0×, 1.5× 표시. `parseFloat(e.target.value)` 변환.
- `aria-label="TTS 속도"` — range input 에 보조 라벨.

```tsx path=src/pages/SettingsPage.tsx lines=155-209
        {isTauri && (
          <>
            <section className="flex flex-col gap-3">
              <h2 className="text-xs font-semibold uppercase tracking-wide text-fg-muted">업데이트</h2>
              <div className="flex flex-wrap gap-2">
                <button className="btn-outline" onClick={onCheckUpdate} disabled={!!busy}>
                  {busy === "update" ? "확인 중..." : "업데이트 확인"}
                </button>
                {update?.available && (
                  <button className="btn-primary" onClick={onInstall} disabled={!!busy}>
                    {busy === "install" ? "설치 중..." : `v${update.version} 설치`}
                  </button>
                )}
              </div>
            </section>

            <section className="flex flex-col gap-3">
              <h2 className="text-xs font-semibold uppercase tracking-wide text-fg-muted">기호 사전 팩 (.nsfpack)</h2>
              <div className="flex flex-wrap gap-2">
                <button className="btn-outline" onClick={onImportPack} disabled={!!busy}>
                  {busy === "import" ? "가져오는 중..." : "팩 가져오기"}
                </button>
                <button className="btn-outline" onClick={onExportPack} disabled={!!busy}>
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
```

**분석 / Analysis**

- `isTauri` 조건부 렌더 — 데스크톱 전용 섹션 둘. *왜 conditional fragment*: 두 섹션을 묶어 일관 처리.
- 모든 버튼 `disabled={!!busy}` — 다른 작업 중일 때 차단.
- 버튼 라벨이 busy 상태에 맞춰 변함 — 좋은 UX.

```tsx path=src/pages/SettingsPage.tsx lines=211-232
        {status && (
          <div role="status" aria-live="polite"
            className="rounded-lg border border-line bg-bg-elevated px-3 py-2 text-xs text-fg-muted">
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
```

**분석 / Analysis**

- `role="status" aria-live="polite"` — 스크린리더가 비강제적으로 읽음. 적절.
- `Field` 내부 컴포넌트 — `<label>` 로 감싸 폼 접근성 보장. `<span>` 라벨과 `children`(input/select).

**짚어볼 점 / Watch-outs**
- Tauri v2 dialog 가 string 반환만 한다고 가정하면 분기 단순화 가능.
- `domainsEnabled` 설정에 UI 부재.
- `status` 자동 사라짐 없음 — 사용자가 다시 동작 안 하면 영구 표시(작은 친화성 이슈).

---

**파일 수준 평가 / File-Level Verdict**

- 강점: busy/status 상태 관리, a11y 적절, 동적 import 분리.
- 위험: v1 호환 분기 잔재, domains UI 부재.
- 개선 권고:
  1. `(picked as { path }).path` 분기 제거(v2 단일 string 가정).
  2. `domainsEnabled` 체크박스 그룹 추가.
  3. `status` 자동 dismiss(`setTimeout` 8초).
- 재읽기 권고 레벨: 2.
