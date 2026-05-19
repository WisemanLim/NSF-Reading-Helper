import { useEffect, useRef, useState } from "react";
import { ocrAvailable, ocrImage } from "../lib/ocr";

interface Props {
  onResult: (latex: string) => void;
}

export function OcrDropzone({ onResult }: Props) {
  const [available, setAvailable] = useState<boolean | null>(null);
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    let mounted = true;
    ocrAvailable().then((ok) => {
      if (mounted) setAvailable(ok);
    });
    return () => {
      mounted = false;
    };
  }, []);

  if (available === false) {
    return null; // Sidecar not running — hide silently.
  }

  const handle = async (file: File) => {
    setBusy(true);
    setErr(null);
    try {
      const r = await ocrImage(file);
      if (!r.latex.trim()) throw new Error("OCR 결과가 비어 있습니다");
      onResult(r.latex);
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const onDrop = (e: React.DragEvent) => {
    e.preventDefault();
    const f = e.dataTransfer.files?.[0];
    if (f) handle(f);
  };
  const onPaste = (e: React.ClipboardEvent) => {
    const item = Array.from(e.clipboardData.items).find((i) =>
      i.type.startsWith("image/"),
    );
    const f = item?.getAsFile();
    if (f) handle(f);
  };

  return (
    <div
      onDrop={onDrop}
      onDragOver={(e) => e.preventDefault()}
      onPaste={onPaste}
      tabIndex={0}
      role="button"
      aria-label="수식 이미지 업로드 (OCR)"
      onClick={() => inputRef.current?.click()}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          inputRef.current?.click();
        }
      }}
      className="flex cursor-pointer items-center gap-3 rounded-lg border border-dashed border-line bg-bg-elevated px-3 py-2 text-xs text-fg-muted focus:outline-none focus-visible:ring-2 focus-visible:ring-brand"
    >
      <input
        ref={inputRef}
        type="file"
        accept="image/*"
        className="hidden"
        onChange={(e) => {
          const f = e.target.files?.[0];
          if (f) handle(f);
          e.target.value = "";
        }}
      />
      <span
        aria-hidden="true"
        className="inline-block h-2 w-2 rounded-full bg-brand"
      />
      <span>
        {busy
          ? "OCR 인식 중..."
          : err
            ? `OCR 오류: ${err}`
            : available === null
              ? "OCR 상태 확인 중..."
              : "수식 이미지를 끌어다 놓거나 붙여넣기(⌘V) — pix2tex"}
      </span>
    </div>
  );
}
