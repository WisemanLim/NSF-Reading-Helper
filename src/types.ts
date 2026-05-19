export type InputFormat = "latex" | "mathml" | "asciimath" | "unicode";
export type SymbolType =
  | "operator"
  | "relation"
  | "delimiter"
  | "greek-letter"
  | "function"
  | "arrow"
  | "set-symbol"
  | "logic-symbol"
  | "chemistry-notation"
  | "unit-symbol"
  | "biology-notation"
  | "constant"
  | "other";

export interface Domain {
  id: number;
  code: string;
  nameKo: string;
  nameEn?: string | null;
  description?: string | null;
}

export interface SymbolDomainMeaning {
  code: string;
  nameKo: string;
  meaningKo?: string | null;
  exampleUsage?: string | null;
}

export interface SymbolRow {
  id: number;
  unicodeChar?: string | null;
  latexCommand?: string | null;
  mathmlTag?: string | null;
  canonicalNameKo: string;
  canonicalNameEn?: string | null;
  readingKo?: string | null;
  readingEn?: string | null;
  shortMeaningKo: string;
  shortMeaningEn?: string | null;
  symbolType: SymbolType;
  difficultyLevel: number;
  isOperator: boolean;
  isRelation: boolean;
  isDelimiter: boolean;
  isGreek: boolean;
  isConstant: boolean;
  isFunction: boolean;
  notes?: string | null;
}

export interface SymbolHit {
  id: number;
  unicodeChar?: string | null;
  latexCommand?: string | null;
  canonicalNameKo: string;
  readingKo?: string | null;
  shortMeaningKo: string;
  symbolType: SymbolType;
  matchedAlias?: string | null;
}

export interface SymbolExample {
  id: number;
  symbolId: number;
  domainCode?: string | null;
  exampleLatex: string;
  exampleReadingKo?: string | null;
  exampleExplanationKo?: string | null;
  difficultyLevel: number;
}

export interface SymbolDetail extends SymbolRow {
  aliases: string[];
  domains: SymbolDomainMeaning[];
  examples: SymbolExample[];
  similar: SymbolHit[];
}

export interface AnalyzeRequest {
  rawInput: string;
  format: InputFormat;
  locale?: "ko" | "en";
}

export interface AnalyzeToken {
  index: number;
  tokenText: string;
  role?: string | null;
  symbolId?: number | null;
  nameKo?: string | null;
  readingKo?: string | null;
  meaningKo?: string | null;
}

export interface AnalyzeResult {
  expressionId?: number | null;
  normalizedLatex?: string | null;
  normalizedMathml?: string | null;
  speech: { ko: string; en?: string };
  tokens: AnalyzeToken[];
}

export interface HistoryItem {
  expressionId: number;
  rawInput: string;
  format: InputFormat;
  speechKo?: string | null;
  createdAt: number;
}

export interface Note {
  id?: number | null;
  expressionId?: number | null;
  symbolId?: number | null;
  bodyMd: string;
  tags?: string[];
}

export interface Settings {
  speechRule: "clearspeak" | "mathspeak";
  speechLocale: "ko" | "en";
  ttsRate: number;
  ttsVoice?: string | null;
  uiLocale: "ko" | "en";
  domainsEnabled: string[];
}
