import { useEffect, useState } from "react";
import { Outlet, useLocation, useNavigate } from "react-router-dom";
import { Sidebar } from "./Sidebar";

export function Layout() {
  const navigate = useNavigate();
  const location = useLocation();
  const [drawerOpen, setDrawerOpen] = useState(false);

  // Close mobile drawer on route change.
  useEffect(() => {
    setDrawerOpen(false);
  }, [location.pathname]);

  useEffect(() => {
    function handler(e: KeyboardEvent) {
      const mod = e.metaKey || e.ctrlKey;
      if (mod && e.key.toLowerCase() === "k") {
        e.preventDefault();
        navigate("/search");
        setTimeout(() => {
          const input = document.querySelector<HTMLInputElement>(
            "input[type='text'], input[placeholder*='기호']",
          );
          input?.focus();
        }, 50);
      }
      if (mod && e.key === "/") {
        e.preventDefault();
        document.getElementById("main-content")?.focus();
      }
      if (e.key === "Escape") {
        setDrawerOpen(false);
      }
    }
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [navigate]);

  return (
    <div className="flex h-[100dvh] w-screen overflow-hidden bg-bg text-fg">
      <a
        href="#main-content"
        className="absolute left-2 top-2 z-50 -translate-y-12 rounded-md bg-brand px-3 py-1.5 text-sm font-medium text-bg transition-transform focus:translate-y-0 focus:outline-none focus:ring-2 focus:ring-brand"
      >
        본문으로 건너뛰기
      </a>

      {/* Desktop sidebar */}
      <div className="hidden md:block">
        <Sidebar />
      </div>

      {/* Mobile drawer */}
      <div
        id="mobile-drawer"
        className={`fixed inset-0 z-40 md:hidden ${drawerOpen ? "" : "pointer-events-none"}`}
        aria-hidden={!drawerOpen}
      >
        <div
          className={`absolute inset-0 bg-black/60 transition-opacity ${drawerOpen ? "opacity-100" : "opacity-0"}`}
          onClick={() => setDrawerOpen(false)}
        />
        <div
          className={`absolute left-0 top-0 h-full w-64 transform transition-transform ${drawerOpen ? "translate-x-0" : "-translate-x-full"}`}
        >
          <Sidebar />
        </div>
      </div>

      <main
        id="main-content"
        tabIndex={-1}
        className="flex min-w-0 flex-1 flex-col overflow-hidden focus:outline-none"
        aria-label="주 콘텐츠 영역"
      >
        {/* Mobile top bar with hamburger */}
        <div className="flex items-center gap-2 border-b border-line bg-bg-elevated px-3 py-2 md:hidden">
          <button
            type="button"
            aria-label={drawerOpen ? "메뉴 닫기" : "메뉴 열기"}
            aria-expanded={drawerOpen}
            aria-controls="mobile-drawer"
            onClick={() => setDrawerOpen((v) => !v)}
            className="rounded-md border border-line p-1.5 focus:outline-none focus-visible:ring-2 focus-visible:ring-brand"
          >
            <svg
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
              aria-hidden="true"
            >
              <line x1="3" y1="6" x2="21" y2="6" />
              <line x1="3" y1="12" x2="21" y2="12" />
              <line x1="3" y1="18" x2="21" y2="18" />
            </svg>
          </button>
          <span className="text-sm font-semibold">NSF Reading Helper</span>
        </div>
        <Outlet />
      </main>
    </div>
  );
}
