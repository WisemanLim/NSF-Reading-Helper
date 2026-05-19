// MathJax v3 wrapper. v4 is still beta as of writing — v3 has stable a11y/SRE
// integration which is what we lean on for korean readings.

let _ready: Promise<typeof window.MathJax> | null = null;

declare global {
  interface Window {
    MathJax?: any;
  }
}

const CDN = "https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-chtml.js";

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

export function clearMathJaxCache(): void {
  if (typeof window === "undefined" || !window.MathJax) return;
  try {
    window.MathJax.typesetClear?.();
  } catch {
    /* noop */
  }
}
