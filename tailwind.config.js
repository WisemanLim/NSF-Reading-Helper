/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        bg: "#0b0d10",
        "bg-elevated": "#13161b",
        "bg-hover": "#1a1e25",
        fg: "#e7eaef",
        "fg-muted": "#9aa3ad",
        "fg-subtle": "#6b7280",
        line: "#262b33",
        brand: "#7ac6ff",
        "brand-soft": "#1a2b3a",
        danger: "#ef4444",
      },
      fontFamily: {
        sans: ["Inter", "Pretendard", "system-ui", "sans-serif"],
        mono: ["JetBrains Mono", "SFMono-Regular", "Consolas", "monospace"],
      },
    },
  },
  plugins: [],
};
