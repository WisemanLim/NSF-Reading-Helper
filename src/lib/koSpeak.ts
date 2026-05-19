// Korean speech polish layer.
//
// Pipeline:
//   1. Try template matching (frac, sum, int, lim, …) for natural phrasing.
//   2. If no template matches, fall back to SRE for Korean speech, then run a
//      post-processor (regex replacements + lexicon swaps).
//   3. Optionally inject canonical Korean readings from the backend's token
//      table so domain-specific symbols get the curated pronunciation.

import rules from "./ko-clearspeak.json";
import { latexToSpeech } from "./sre";
import type { AnalyzeToken } from "../types";

interface TemplateRule {
  pattern: string;
  speech: string;
}
interface Replace {
  from: string;
  to: string;
}
interface RulePack {
  templates: TemplateRule[];
  replacements: Replace[];
  sre_post: Replace[];
  lexicon: Record<string, string>;
}

const PACK = rules as unknown as RulePack;

/** Try one of the LaTeX templates. Returns the formatted speech or null. */
function tryTemplate(latex: string): string | null {
  const trimmed = latex.trim();
  for (const rule of PACK.templates) {
    const re = new RegExp(rule.pattern);
    const m = trimmed.match(re);
    if (m) {
      let out = rule.speech;
      for (let i = 1; i < m.length; i++) {
        out = out.replace(new RegExp(`\\$${i}`, "g"), polishInline(m[i] ?? ""));
      }
      return out.trim().replace(/\s+/g, " ");
    }
  }
  return null;
}

/** Apply replacements + lexicon swaps to an inline LaTeX fragment so template
 *  captures like `$1` come out reading-friendly. */
function polishInline(s: string): string {
  let r = s;
  for (const rep of PACK.replacements) {
    r = r.replace(new RegExp(rep.from, "g"), rep.to);
  }
  // Greek lexicon — match \alpha / \\alpha / alpha
  r = r.replace(/\\([A-Za-z]+)/g, (_, name: string) => {
    const k = name.toLowerCase();
    return PACK.lexicon[k] ?? name;
  });
  return r.trim();
}

/** Polish an SRE-generated Korean string further. SRE's `ko` locale is partial,
 *  so the output often contains untranslated English fragments. */
function polishSreOutput(raw: string): string {
  let r = raw;
  for (const rep of PACK.sre_post) {
    r = r.replace(new RegExp(rep.from, "g"), rep.to);
  }
  return r.replace(/\s+/g, " ").trim();
}

/** Build a token-driven fallback when SRE and templates both fail. */
function fromTokens(tokens: AnalyzeToken[]): string {
  if (tokens.length === 0) return "";
  return tokens
    .map((t) => t.readingKo || t.nameKo || t.tokenText)
    .join(" ");
}

export interface SpeechInput {
  latex: string;
  tokens?: AnalyzeToken[];
  preferTemplate?: boolean;
}

export async function generateKoreanSpeech(
  input: SpeechInput,
): Promise<string> {
  const { latex, tokens = [], preferTemplate = true } = input;

  if (preferTemplate) {
    const tpl = tryTemplate(latex);
    if (tpl) return tpl;
  }

  try {
    const raw = await latexToSpeech(latex, "ko");
    const polished = polishSreOutput(raw);
    if (polished && polished !== latex) return polished;
  } catch (e) {
    console.debug("SRE failed, falling back to token linearization", e);
  }

  const fallback = fromTokens(tokens);
  return fallback || latex;
}

/** Synchronous version for cases where SRE isn't needed (templates only). */
export function generateKoreanSpeechSync(
  latex: string,
  tokens: AnalyzeToken[] = [],
): string {
  return tryTemplate(latex) ?? fromTokens(tokens) ?? latex;
}
