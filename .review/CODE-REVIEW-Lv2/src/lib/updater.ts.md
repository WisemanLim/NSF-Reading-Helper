# src/lib/updater.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: Tauri 의 `@tauri-apps/plugin-updater` 동적 래퍼. 웹에선 no-op.
- LOC: 42
- 주요 export: `checkForUpdate`, `downloadAndInstall`, `UpdateInfo`
- 의존 모듈: `./env`
- 외부 패키지: `@tauri-apps/plugin-updater` (dynamic import)
- 부수효과: 업데이트 검사·다운로드·설치 트리거

---

```ts path=src/lib/updater.ts lines=1-12
// Tauri auto-update wrapper. No-op on web — the SettingsPage check returns
// "manual" and the updater button is hidden.

import { isTauri } from "./env";

export interface UpdateInfo {
  available: boolean;
  version?: string;
  currentVersion?: string;
  notes?: string;
}
```

**분석 / Analysis**

- `UpdateInfo` 는 SettingsPage 에 노출되는 결과 형식.

```ts path=src/lib/updater.ts lines=13-33
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
```

**분석 / Analysis**

- `await import(...)` — 웹 빌드에서 정적 분석 시 chunk 가 생성되지만 `isTauri` 가드로 실제 fetch 안 됨. 번들 크기 차원에서는 별도 청크라 영향 최소.
- `mod.check()` — Tauri updater 가 endpoint(`tauri.conf.json` 의 updater 섹션) 를 조회. `null` 반환은 "최신".
- `update.body ?? undefined` — `null` 을 undefined 로 변환. `UpdateInfo.notes` 가 옵셔널.
- 모든 실패는 `available:false` 로 통일 — SettingsPage 가 단순 분기.

```ts path=src/lib/updater.ts lines=35-42
export async function downloadAndInstall(): Promise<void> {
  if (!isTauri) return;
  const mod = await import("@tauri-apps/plugin-updater");
  const update = await mod.check();
  if (!update) return;
  await update.downloadAndInstall();
  // After install Tauri prompts to restart; let the user trigger it.
}
```

**분석 / Analysis**

- *왜 다시 `check()` 호출*: caller(SettingsPage)가 `checkForUpdate` 결과를 캐시하지 않고 별도 호출. 한 번 더 체크하지만 동일 결과면 무해. 미세한 중복.
- 재시작은 사용자 트리거(주석) — Tauri updater 가 자동 restart 옵션도 제공하나 의도적으로 패스.

**짚어볼 점 / Watch-outs**
- 두 번 check — 사용자가 첫 결과 본 뒤 외부 상태가 바뀌면 두 번째 결과만 사용. UX 일관성 측면 미세 위험.
- 다운로드 실패 시 일부 진행 후 throw — SettingsPage 의 catch 가 메시지를 표시하나 부분 다운로드 잔여 정리 정책 없음.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 짧고 분기 명확.
- 위험: 중복 check, 부분 다운로드 정리 정책.
- 개선 권고:
  1. caller 가 `checkForUpdate` 의 결과를 전달해 두 번째 check 회피.
  2. 진행률 콜백 노출(SettingsPage 가 사용자에게 % 표시).
- 재읽기 권고 레벨: 1.
