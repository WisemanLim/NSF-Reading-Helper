type P = React.SVGProps<SVGSVGElement>;
const S = (p: P) => ({
  width: 16,
  height: 16,
  viewBox: "0 0 24 24",
  fill: "none",
  stroke: "currentColor",
  strokeWidth: 2,
  strokeLinecap: "round" as const,
  strokeLinejoin: "round" as const,
  ...p,
});

export const IconHome = (p: P) => (
  <svg {...S(p)}><path d="M3 11l9-8 9 8M5 10v10h14V10" /></svg>
);
export const IconFunction = (p: P) => (
  <svg {...S(p)}><path d="M6 20c2-3 2-13 6-13h2M3 12h10" /></svg>
);
export const IconSearch = (p: P) => (
  <svg {...S(p)}><circle cx="11" cy="11" r="7" /><path d="M21 21l-4.3-4.3" /></svg>
);
export const IconBook = (p: P) => (
  <svg {...S(p)}><path d="M4 4h12a4 4 0 014 4v12H8a4 4 0 01-4-4z" /></svg>
);
export const IconSettings = (p: P) => (
  <svg {...S(p)}><circle cx="12" cy="12" r="3" /><path d="M19 12a7 7 0 00-.1-1.2l2-1.5-2-3.5-2.4 1a7 7 0 00-2-1.2L14 3h-4l-.5 2.6a7 7 0 00-2 1.2l-2.4-1-2 3.5 2 1.5A7 7 0 005 12c0 .4 0 .8.1 1.2l-2 1.5 2 3.5 2.4-1a7 7 0 002 1.2L10 21h4l.5-2.6a7 7 0 002-1.2l2.4 1 2-3.5-2-1.5c.1-.4.1-.8.1-1.2z" /></svg>
);
export const IconPlay = (p: P) => (
  <svg {...S(p)}><polygon points="6 4 20 12 6 20 6 4" /></svg>
);
export const IconStop = (p: P) => (
  <svg {...S(p)}><rect x="6" y="6" width="12" height="12" rx="1" /></svg>
);
export const IconSparkles = (p: P) => (
  <svg {...S(p)}><path d="M12 3v4M12 17v4M3 12h4M17 12h4M5.6 5.6l2.8 2.8M15.6 15.6l2.8 2.8M5.6 18.4l2.8-2.8M15.6 8.4l2.8-2.8" /></svg>
);
export const IconStar = (p: P) => (
  <svg {...S(p)}><polygon points="12 2 15 9 22 10 17 15 18 22 12 18 6 22 7 15 2 10 9 9" /></svg>
);
