import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const TAURI_DEV_HOST = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: TAURI_DEV_HOST || false,
    hmr: TAURI_DEV_HOST
      ? { protocol: "ws", host: TAURI_DEV_HOST, port: 1421 }
      : undefined,
    watch: { ignored: ["**/src-tauri/**", "**/web-server/**"] },
  },
  envPrefix: ["VITE_", "TAURI_"],
});
