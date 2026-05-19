import { useState } from "react";
import { speak, stopSpeaking } from "../lib/tts";
import { IconPlay, IconStop } from "./icons";

interface Props {
  text: string;
  defaultRate?: number;
}

export function SpeechControls({ text, defaultRate = 1.0 }: Props) {
  const [rate, setRate] = useState(defaultRate);
  const [playing, setPlaying] = useState(false);

  const onPlay = () => {
    if (!text) return;
    setPlaying(true);
    speak(text, rate);
    setTimeout(() => setPlaying(false), Math.max(2000, text.length * 80));
  };
  const onStop = () => {
    stopSpeaking();
    setPlaying(false);
  };

  return (
    <div className="flex items-center gap-2 rounded-lg border border-line bg-bg-elevated px-3 py-2">
      {playing ? (
        <button className="btn-primary !py-1" onClick={onStop}>
          <IconStop /> 정지
        </button>
      ) : (
        <button className="btn-primary !py-1" onClick={onPlay} disabled={!text}>
          <IconPlay /> 읽기
        </button>
      )}
      <label htmlFor="tts-rate" className="ml-2 text-xs text-fg-muted">
        속도
      </label>
      <input
        id="tts-rate"
        type="range"
        min={0.5}
        max={2.0}
        step={0.1}
        value={rate}
        onChange={(e) => setRate(parseFloat(e.target.value))}
        aria-label="음성 속도 배수"
        aria-valuemin={0.5}
        aria-valuemax={2.0}
        aria-valuenow={rate}
        aria-valuetext={`${rate.toFixed(1)}배`}
        className="w-32 accent-brand focus:outline-none focus-visible:ring-2 focus-visible:ring-brand"
      />
      <span className="w-10 text-right font-mono text-xs text-fg-muted">
        {rate.toFixed(1)}×
      </span>
      <div className="ml-auto truncate text-xs text-fg-muted" title={text}>
        {text || <span className="italic text-fg-subtle">읽기 텍스트 없음</span>}
      </div>
    </div>
  );
}
