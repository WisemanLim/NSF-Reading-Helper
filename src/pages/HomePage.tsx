import { Link } from "react-router-dom";
import { IconFunction, IconSearch, IconSparkles } from "../components/icons";

export function HomePage() {
  return (
    <div className="flex h-full flex-col overflow-y-auto">
      <header className="flex h-14 shrink-0 items-center border-b border-line px-6">
        <h1 className="text-sm font-semibold text-fg-muted">홈</h1>
      </header>
      <div className="mx-auto flex w-full max-w-3xl flex-1 flex-col justify-center gap-8 px-6 py-10">
        <div className="space-y-2 text-center">
          <div className="mx-auto mb-3 flex h-12 w-12 items-center justify-center rounded-2xl bg-brand-soft text-brand">
            <IconSparkles />
          </div>
          <h2 className="text-2xl font-semibold tracking-tight">
            자연과학 수식, 읽고 이해해 볼까요?
          </h2>
          <p className="text-sm text-fg-muted">
            수식을 입력하면 한국어로 읽어주고, 기호 하나하나의 뜻을 풀어드려요.
          </p>
        </div>
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
          <Link
            to="/analyze"
            className="flex items-start gap-3 rounded-2xl border border-line bg-bg-elevated p-5 hover:border-brand/60"
          >
            <IconFunction />
            <div>
              <div className="text-sm font-semibold">수식 분석</div>
              <div className="text-xs text-fg-muted">
                LaTeX · MathML · AsciiMath · Unicode 입력
              </div>
            </div>
          </Link>
          <Link
            to="/search"
            className="flex items-start gap-3 rounded-2xl border border-line bg-bg-elevated p-5 hover:border-brand/60"
          >
            <IconSearch />
            <div>
              <div className="text-sm font-semibold">기호 사전 검색</div>
              <div className="text-xs text-fg-muted">
                한글·영어·LaTeX·Unicode 통합 검색
              </div>
            </div>
          </Link>
        </div>
      </div>
    </div>
  );
}
