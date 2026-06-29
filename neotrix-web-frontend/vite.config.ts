import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: process.env.NODE_ENV !== 'production' ? [
      { find: "@tauri-apps/api/core", replacement: path.resolve(__dirname, "mocks/tauri-core.ts") },
      { find: "@tauri-apps/api/event", replacement: path.resolve(__dirname, "mocks/tauri-event.ts") },
      { find: "@tauri-apps/api/window", replacement: path.resolve(__dirname, "mocks/tauri-window.ts") },
      { find: "@tauri-apps/plugin-dialog", replacement: path.resolve(__dirname, "mocks/plugin-dialog.ts") },
      { find: "@tauri-apps/plugin-fs", replacement: path.resolve(__dirname, "mocks/plugin-fs.ts") },
      { find: "@tauri-apps/plugin-deep-link", replacement: path.resolve(__dirname, "mocks/plugin-deep-link.ts") },
      { find: "@tauri-apps/plugin-shell", replacement: path.resolve(__dirname, "mocks/plugin-shell.ts") },
      { find: "@tauri-apps/plugin-updater", replacement: path.resolve(__dirname, "mocks/plugin-updater.ts") },
    ] : [],
  },
  server: {
    port: 3420,
    strictPort: false,
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
  },
});
