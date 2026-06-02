# src/lib/tts.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: Tauri(데스크톱) → OS TTS 명령, 웹 → `window.speechSynthesis` 의 wrapper.
- LOC: 27
- 주요 export: `speak`, `stopSpeaking`
- 의존 모듈: `./env`, `../ipc/client`
- 외부 패키지: 없음
- 부수효과: 음성 합성 시작/정지

---

```ts path=src/lib/tts.ts lines=1-18
// Web Speech API wrapper for browser TTS playback.
// Desktop fallback (OS TTS) is invoked via Tauri command when isTauri is true.

import { isTauri } from "./env";
import { ipc } from "../ipc/client";

export function speak(text: string, rate = 1.0, lang = "ko-KR") {
  if (isTauri) {
    ipc.ttsSpeak(text, { rate, lang }).catch(console.warn);
    return;
  }
  if (typeof window === "undefined" || !window.speechSynthesis) return;
  window.speechSynthesis.cancel();
  const u = new SpeechSynthesisUtterance(text);
  u.rate = rate;
  u.lang = lang;
  window.speechSynthesis.speak(u);
}
```

**분석 / Analysis**

- 데스크톱 분기 / *왜*: 브라우저 SpeechSynthesis 는 OS의 음성 데이터에 의존(특히 macOS Safari/Edge 에서 ko-KR 보이스가 누락되는 경우 있음). Tauri 는 OS-네이티브 `say`/`espeak-ng`/PowerShell 로 직접 호출.
- 웹 분기:
  - `window.speechSynthesis.cancel()` — 큐에 있는 이전 발화 중단. 새 발화가 즉시 시작되도록.
  - `SpeechSynthesisUtterance` 의 `voice` 미설정 — 브라우저 기본. `Settings.ttsVoice` 가 미전달.
- *흔한 실수*: `SpeechSynthesisUtterance.rate` 는 0.1~10 범위. caller(SpeechControls)가 0.5~2 범위로 제한해 안전.
- `.catch(console.warn)` — Tauri 호출 실패는 무시. UI 에 표시되지 않음.

```ts path=src/lib/tts.ts lines=20-27
export function stopSpeaking() {
  if (isTauri) {
    ipc.ttsStop().catch(console.warn);
    return;
  }
  if (typeof window === "undefined" || !window.speechSynthesis) return;
  window.speechSynthesis.cancel();
}
```

**분석 / Analysis**

- 대칭적 정지 함수.

**짚어볼 점 / Watch-outs**
- `Settings.ttsVoice` 미연결 — 사용자가 보이스를 고를 수 있어야 한다면 추가 필요.
- `lang = "ko-KR"` 하드코딩 — `Settings.speechLocale` 와 동기화되지 않음. 호출자에서 매핑 필요.
- 데스크톱 분기는 fire-and-forget — 합성 완료 이벤트가 없음.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 작은 표면, 분기 명확.
- 위험: voice 미사용, locale 하드코딩.
- 개선 권고:
  1. caller 가 `lang` 을 `Settings.speechLocale==='ko' ? 'ko-KR' : 'en-US'` 로 전달.
  2. voice 매핑 추가.
  3. Tauri 측 합성 완료 이벤트(emit) 도입해 UI 가 진행 상태 표시.
- 재읽기 권고 레벨: 1.
