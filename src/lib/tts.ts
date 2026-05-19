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

export function stopSpeaking() {
  if (isTauri) {
    ipc.ttsStop().catch(console.warn);
    return;
  }
  if (typeof window === "undefined" || !window.speechSynthesis) return;
  window.speechSynthesis.cancel();
}
