import type { InputFormat } from "../types";

const UNICODE_MATH = /[∑∫∮∂∇∞≈≠≤≥⊂⊆∈∉∀∃→⇌±×÷√Σπμσ°αβγδεθλ]/;

export function detectInputFormat(input: string): InputFormat {
  const t = input.trim();
  if (!t) return "latex";
  if (t.startsWith("<math") || t.includes("</math>")) return "mathml";
  if (t.includes("\\")) return "latex";
  if (UNICODE_MATH.test(t) && !/[a-zA-Z]{3,}/.test(t)) return "unicode";
  if (/sum_|int_|sqrt|frac|\^|_\(/.test(t)) return "asciimath";
  return "latex";
}
