import { Component, type ReactNode } from "react";

interface Props {
  children: ReactNode;
}
interface State {
  err: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { err: null };

  static getDerivedStateFromError(err: Error) {
    return { err };
  }
  componentDidCatch(err: Error, info: unknown) {
    console.error("ErrorBoundary", err, info);
  }
  render() {
    if (this.state.err) {
      return (
        <div className="flex h-screen w-screen flex-col items-center justify-center gap-3 bg-bg p-8 text-center">
          <div className="text-base font-semibold text-danger">렌더링 오류</div>
          <pre className="max-w-2xl whitespace-pre-wrap rounded-lg bg-bg-elevated p-4 text-left font-mono text-xs text-fg-muted">
            {this.state.err.message}
          </pre>
        </div>
      );
    }
    return this.props.children;
  }
}
