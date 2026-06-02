# src/lib/detexify.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 손그림 stroke → LaTeX 후보 클라이언트. 환경변수 override → 동일-출처 프록시 → 공개 Detexify 순으로 fallback.
- LOC: 108
- 주요 export: `classifyStrokes`, `ClassifyHit`
- 의존 모듈: `./env`
- 외부 패키지: 없음 (Fetch)
- 부수효과: 네트워크 I/O (잠재적으로 외부 호스트)

---

```ts path=src/lib/detexify.ts lines=1-12
// Detexify-compatible classifier client. Tries (in order):
//   1. VITE_NSF_DETEXIFY_URL (custom — useful for self-hosted)
//   2. Same-origin /api/handwriting/classify (backend proxy)
//   3. https://detexify.kirelabs.org/classify.json (CORS-permitting public host)
//
// Detexify's public API expects:
//   POST { "strokes": [[{x,y},...], ...] }  // application/json
// and replies with an array of { score: number, symbol: { command, package, ... } }.
// We normalize that into a flat ClassifyHit shape.

import { apiBaseUrl } from "./env";
```

**분석 / Analysis**

- 주석으로 외부 API 스키마 명시 — 좋은 핸드오버 문서화.

```ts path=src/lib/detexify.ts lines=13-34
export interface ClassifyHit {
  command: string;
  package?: string | null;
  score: number;
  id?: string;
}

interface Point {
  x: number;
  y: number;
}

function detexifyOverrideUrl(): string | null {
  const env = (import.meta as unknown as { env?: Record<string, string> }).env;
  return env?.VITE_NSF_DETEXIFY_URL || null;
}

function backendProxyUrl(): string {
  return `${apiBaseUrl()}/api/handwriting/classify`;
}

const PUBLIC_DETEXIFY = "https://detexify.kirelabs.org/classify.json";
```

**분석 / Analysis**

- `ClassifyHit.command` 은 LaTeX 명령(`\alpha` 등). `score` 는 정규화된 0~1 (1 이 best).
- `backendProxyUrl()` 가 `/api/handwriting/classify` — web-server 의 `main.rs::handwriting_classify` 라우트와 매핑. **(`lib/ocr.ts` 는 `/ocr/...`, 이 모듈은 `/api/...` — 두 라우팅 컨벤션 공존)**.
- 공개 endpoint 하드코딩 — HTTPS. 정책 변경 시 갱신 필요.

```ts path=src/lib/detexify.ts lines=36-44
async function postJson(url: string, body: unknown): Promise<unknown> {
  const r = await fetch(url, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!r.ok) throw new Error(`HTTP ${r.status}: ${r.statusText}`);
  return r.json();
}
```

**분석 / Analysis**

- 표준 JSON POST. 응답 검증 없음.

```ts path=src/lib/detexify.ts lines=46-79
function normalize(raw: unknown): ClassifyHit[] {
  if (!Array.isArray(raw)) return [];
  const hits: ClassifyHit[] = [];
  for (const item of raw) {
    if (!item || typeof item !== "object") continue;
    const obj = item as Record<string, unknown>;
    const score = typeof obj.score === "number" ? obj.score : null;
    const sym = (obj.symbol ?? obj) as Record<string, unknown>;
    const cmd =
      typeof sym.command === "string"
        ? sym.command
        : typeof sym.id === "string"
          ? sym.id
          : null;
    if (cmd === null || score === null) continue;
    hits.push({
      command: cmd.startsWith("\\") ? cmd : `\\${cmd}`,
      package:
        typeof sym.package === "string" ? (sym.package as string) : undefined,
      score: Math.max(0, Math.min(1, score)),
      id: typeof sym.id === "string" ? (sym.id as string) : undefined,
    });
  }
  // Lower score = better in original Detexify (distance). Normalize so we can
  // sort by ascending score then surface as confidence = 1 - normalized.
  hits.sort((a, b) => a.score - b.score);
  const maxScore = hits[hits.length - 1]?.score ?? 1;
  const minScore = hits[0]?.score ?? 0;
  const span = Math.max(0.0001, maxScore - minScore);
  return hits.map((h) => ({
    ...h,
    score: 1 - (h.score - minScore) / span,
  }));
}
```

**분석 / Analysis**

- 매우 방어적 정규화 — `Array.isArray`, 객체 타입 가드, `score` 와 `command` 둘 다 있을 때만 채택.
- `obj.symbol ?? obj` — 공개 Detexify 는 `{ score, symbol: { command, ... } }`, 일부 self-host 변형은 평탄 객체 (`{ score, command }`). 두 형태 모두 지원.
- `cmd.startsWith("\\") ? cmd : \`\\${cmd}\`` — 일관된 `\command` 형식 보장.
- `Math.max(0, Math.min(1, score))` — 잘못된 score 도 clamp.
- **정규화 알고리즘**:
  1. 거리(distance) → 오름차순 정렬(가장 가까움이 앞).
  2. min/max 로 span 계산, 0.0001 미만 방지.
  3. `1 - (score - min) / span` → confidence (1 이 최선, 0 이 최악).
- *흔한 실수*: span 이 0 일 때(모든 score 동일) Infinity 가 안 되도록 floor 0.0001. 정확.
- *주의*: 결과 정렬 순서가 score 변환 *전* 기준 — 결과는 confidence 내림차순으로 보임(거리 오름차순과 동일).

```ts path=src/lib/detexify.ts lines=81-108
export async function classifyStrokes(
  strokes: Point[][],
): Promise<ClassifyHit[]> {
  if (strokes.length === 0) return [];
  const body = { strokes };

  const override = detexifyOverrideUrl();
  if (override) {
    return normalize(await postJson(override, body));
  }

  // Try same-origin proxy first (avoids CORS, single failure mode).
  try {
    const r = await postJson(backendProxyUrl(), body);
    return normalize(r);
  } catch (proxyErr) {
    // Fall back to public Detexify directly.
    try {
      return normalize(await postJson(PUBLIC_DETEXIFY, body));
    } catch (publicErr) {
      throw new Error(
        `Detexify 호출 실패 — proxy: ${
          proxyErr instanceof Error ? proxyErr.message : String(proxyErr)
        }; public: ${publicErr instanceof Error ? publicErr.message : String(publicErr)}`,
      );
    }
  }
}
```

**분석 / Analysis**

- 빈 strokes → 즉시 빈 배열.
- override 가 있으면 직접 호출 (proxy 시도 안 함).
- proxy → public 의 2단 fallback. *왜 public-first 아닌가*: 동일-출처는 CORS 가 없고, 운영 환경에서 일반적으로 더 빠름. public 은 외부 호스트로 사용자 데이터 송신이라 *프라이버시 측면에서 마지막에 두는 게 적절*.
- *짚어볼 점*: **public fallback 이 사용자 손그림 데이터를 외부 호스트로 자동 전송** — 사용자가 인지하지 못할 수 있음. 운영 정책에 따라 fallback 자체를 제거하거나 명시적 사용자 동의 추가가 안전.

**짚어볼 점 / Watch-outs**
- 외부 호스트 자동 호출 — 개인정보(필체 패턴) 잠재 노출.
- 응답 스키마 변형 두 종을 추측으로 처리 — 향후 Detexify API 변경 시 정규화 실패.
- 타임아웃 없음(`AbortController`) — 느린 응답 시 UX 멈춤.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: 방어적 normalize, 명확한 fallback 순서.
- 위험: 외부 호스트 자동 전송, 타임아웃 없음.
- 개선 권고:
  1. 공개 fallback 호출 전 사용자 확인 토글(`Settings.allowPublicDetexify`).
  2. `AbortController` 로 5-10s 타임아웃.
  3. 운영 환경에선 public fallback 자체 비활성 옵션.
- 재읽기 권고 레벨: 2.
