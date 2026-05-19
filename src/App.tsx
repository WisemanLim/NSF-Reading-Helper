import { useEffect, useState } from "react";
import { Route, Routes } from "react-router-dom";
import { Layout } from "./components/Layout";
import { HomePage } from "./pages/HomePage";
import { AnalyzePage } from "./pages/AnalyzePage";
import { SearchPage } from "./pages/SearchPage";
import { LibraryPage } from "./pages/LibraryPage";
import { SettingsPage } from "./pages/SettingsPage";
import { useAppStore } from "./store/useAppStore";
import { ErrorBoundary } from "./components/ErrorBoundary";

export default function App() {
  const bootstrap = useAppStore((s) => s.bootstrap);
  const loaded = useAppStore((s) => s.loaded);
  const [bootError, setBootError] = useState<string | null>(null);

  useEffect(() => {
    bootstrap().catch((e) => setBootError(e instanceof Error ? e.message : String(e)));
  }, [bootstrap]);

  if (bootError) {
    return (
      <div className="flex h-screen w-screen flex-col items-center justify-center gap-3 bg-bg p-8 text-center">
        <div className="text-base font-semibold text-danger">초기화에 실패했습니다</div>
        <pre className="max-w-2xl whitespace-pre-wrap rounded-lg bg-bg-elevated p-4 text-left font-mono text-xs text-fg-muted">
          {bootError}
        </pre>
      </div>
    );
  }
  if (!loaded) {
    return (
      <div className="flex h-screen w-screen items-center justify-center bg-bg text-sm text-fg-muted">
        초기화 중...
      </div>
    );
  }

  return (
    <ErrorBoundary>
      <Routes>
        <Route element={<Layout />}>
          <Route index element={<HomePage />} />
          <Route path="analyze" element={<AnalyzePage />} />
          <Route path="search" element={<SearchPage />} />
          <Route path="library" element={<LibraryPage />} />
          <Route path="settings" element={<SettingsPage />} />
        </Route>
      </Routes>
    </ErrorBoundary>
  );
}
