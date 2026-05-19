import { NavLink } from "react-router-dom";
import { cn } from "../lib/cn";
import {
  IconBook,
  IconFunction,
  IconHome,
  IconSearch,
  IconSettings,
} from "./icons";

const NAV = [
  { to: "/", label: "홈", icon: IconHome, end: true },
  { to: "/analyze", label: "분석", icon: IconFunction },
  { to: "/search", label: "검색", icon: IconSearch },
  { to: "/library", label: "라이브러리", icon: IconBook },
  { to: "/settings", label: "설정", icon: IconSettings },
];

export function Sidebar() {
  return (
    <aside
      className="flex w-56 shrink-0 flex-col gap-1 border-r border-line bg-bg-elevated p-3"
      aria-label="주 내비게이션"
    >
      <div className="px-2 pb-3 pt-1 text-sm font-semibold tracking-tight text-fg">
        NSF Reading Helper
      </div>
      <nav aria-label="페이지" className="flex flex-col gap-1">
        {NAV.map(({ to, label, icon: Icon, end }) => (
          <NavLink
            key={to}
            to={to}
            end={end}
            className={({ isActive }) =>
              cn(
                "flex items-center gap-2 rounded-lg px-3 py-2 text-sm focus:outline-none focus-visible:ring-2 focus-visible:ring-brand",
                isActive
                  ? "bg-brand-soft text-brand"
                  : "text-fg-muted hover:bg-bg-hover hover:text-fg",
              )
            }
          >
            <Icon aria-hidden="true" />
            <span>{label}</span>
          </NavLink>
        ))}
      </nav>
      <div className="mt-auto px-2 py-2 text-[10px] text-fg-subtle">
        v0.1.0
      </div>
    </aside>
  );
}
