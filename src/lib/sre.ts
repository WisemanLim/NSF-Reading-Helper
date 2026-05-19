// Speech Rule Engine (SRE) wrapper. Loads lazily so the initial bundle stays small.
// SRE itself ships korean readings; we apply our own post-processor for symbols
// that don't have a canonical korean rule yet.

let _ready: Promise<any> | null = null;

async function load() {
  if (_ready) return _ready;
  _ready = (async () => {
    const sre: any = await import("speech-rule-engine");
    await sre.setupEngine({
      domain: "clearspeak",
      style: "default",
      locale: "ko",
      speech: "shallow",
      modality: "speech",
    });
    return sre;
  })();
  return _ready;
}

export async function latexToSpeech(
  latex: string,
  locale: "ko" | "en" = "ko",
): Promise<string> {
  const sre = await load();
  try {
    if (locale !== "ko") await sre.setupEngine({ locale });
    const out: string = sre.toSpeech(`<math>${sre.toMathml(latex)}</math>`);
    return out.trim();
  } catch (e) {
    console.warn("SRE failure, fallback", e);
    return latex;
  }
}

export async function mathmlToSpeech(
  mathml: string,
  locale: "ko" | "en" = "ko",
): Promise<string> {
  const sre = await load();
  if (locale !== "ko") await sre.setupEngine({ locale });
  return sre.toSpeech(mathml).trim();
}
