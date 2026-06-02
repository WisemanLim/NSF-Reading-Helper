# src/lib/mathjax.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: MathJax v3 의 CDN-script 로더 + LaTeX → DOM 렌더 helper.
- LOC: 67
- 주요 export: `loadMathJax`, `renderLatex`, `clearMathJaxCache`
- 외부 패키지: 없음 (런타임 `<script>` 삽입)
- 부수효과: `<script>` DOM 삽입, 전역 `window.MathJax` 세팅

---

```ts path=src/lib/mathjax.ts lines=1-12
// MathJax v3 wrapper. v4 is still beta as of writing — v3 has stable a11y/SRE
// integration which is what we lean on for korean readings.

let _ready: Promise<typeof window.MathJax> | null = null;

declare global {
  interface Window {
    MathJax?: any;
  }
}

const CDN = "https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-chtml.js";
```

**분석 / Analysis**

- Lazy singleton 캐시(`_ready`) — `sre.ts` 와 동일 패턴.
- *왜 v3*: 주석이 명시 — v4 베타, a11y/SRE 안정성.
- `CDN` URL 하드코딩 — 오프라인/사내망 차단 시 실패. `mathjax-full` npm 의존을 선언만 하고 실제론 CDN 사용 → 데드 의존성.

```ts path=src/lib/mathjax.ts lines=14-46
export function loadMathJax(): Promise<typeof window.MathJax> {
  if (_ready) return _ready;
  _ready = new Promise((resolve, reject) => {
    if (typeof window === "undefined") {
      reject(new Error("MathJax requires a browser environment"));
      return;
    }
    window.MathJax = {
      tex: {
        inlineMath: [["$", "$"], ["\\(", "\\)"]],
        displayMath: [["$$", "$$"], ["\\[", "\\]"]],
        packages: { "[+]": ["ams", "mhchem", "noerrors"] },
      },
      options: {
        enableMenu: false,
        renderActions: { addMenu: [] },
      },
      loader: { load: ["[tex]/mhchem", "[tex]/noerrors", "a11y/semantic-enrich"] },
      startup: {
        ready: () => {
          window.MathJax.startup.defaultReady();
          window.MathJax.startup.promise.then(() => resolve(window.MathJax));
        },
      },
    };
    const s = document.createElement("script");
    s.src = CDN;
    s.async = true;
    s.onerror = () => reject(new Error("Failed to load MathJax"));
    document.head.appendChild(s);
  });
  return _ready;
}
```

**분석 / Analysis**

- **MathJax v3 의 표준 부트 패턴**: `window.MathJax = {...설정}` 을 *script src 로딩 전에* 세팅하면, 스크립트가 자동으로 그 설정을 읽음. 그 후 `startup.ready` 콜백에서 `defaultReady()` 호출 → init 완료 후 `startup.promise.then(...)` 으로 resolve.
- `packages: { "[+]": [...] }` — 기본 패키지 + 추가 패키지 결합 문법. `mhchem` 은 화학 수식, `noerrors` 는 파싱 실패해도 raw 표시, `a11y/semantic-enrich` 는 SRE 와 연동(접근성).
- `enableMenu: false`, `renderActions.addMenu: []` — 우클릭 메뉴 제거. UX 단순화.
- `s.async = true` — non-blocking script 로딩.
- *흔한 실수*: `script.onload` 없이 `startup.ready` 만 의지 — MathJax 가 init 중 throw 하면 promise 가 영원히 미해결. `onerror` 만 reject. 부분 실패(설정 잘못)는 행 상태.

```ts path=src/lib/mathjax.ts lines=48-58
export async function renderLatex(
  container: HTMLElement,
  latex: string,
  display = true,
): Promise<void> {
  const mj = await loadMathJax();
  container.textContent = "";
  const node = await mj.tex2chtmlPromise(latex, { display });
  container.appendChild(node);
  await mj.startup.document.updateDocument();
}
```

**분석 / Analysis**

- `container.textContent = ""` — 기존 내용 정리(replace).
- `tex2chtmlPromise(latex, { display })` — display vs inline 모드.
- `mj.startup.document.updateDocument()` — 새 노드를 DOM 트리에 등록하고 접근성 enrich 트리거. 한국어 a11y 가 SRE 와 함께 동작하는 핵심.
- *왜 await*: 비동기 — 컨테이너에 결과가 보이기까지 대기.

```ts path=src/lib/mathjax.ts lines=60-67
export function clearMathJaxCache(): void {
  if (typeof window === "undefined" || !window.MathJax) return;
  try {
    window.MathJax.typesetClear?.();
  } catch {
    /* noop */
  }
}
```

**분석 / Analysis**

- 메모리 누수 방지용 보조 함수. `RenderView` 등이 unmount 시 호출 가능. `?.()` optional call.

**짚어볼 점 / Watch-outs**
- CDN 의존 — 오프라인 환경/사내 네트워크/CDN 장애 시 수식 렌더 실패.
- `mathjax-full` npm dep 가 설치되어 있으나 import 되지 않음 → 데드 의존성. 향후 self-hosted bundle 로 옮기는 게 좋음.
- `: any` 캐스팅 — 타입 안전성 낮음.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: v3 부트 패턴 정확, lazy 캐시.
- 위험: CDN 단일 출처, npm dep 데드.
- 개선 권고:
  1. `mathjax-full` 을 dynamic import 로 사용해 self-hosted.
  2. CDN 실패 시 fallback (예: local 동일 파일).
  3. 타입 정의 보강.
- 재읽기 권고 레벨: 2.
