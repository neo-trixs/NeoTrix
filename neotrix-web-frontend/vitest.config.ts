import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  plugins: [react()],
  resolve: {
    preserveSymlinks: true,
    alias: [
      { find: "@tauri-apps/api/core", replacement: path.resolve(__dirname, "mocks/tauri-core.ts") },
      { find: "@tauri-apps/api/event", replacement: path.resolve(__dirname, "mocks/tauri-event.ts") },
      { find: "@tauri-apps/api/window", replacement: path.resolve(__dirname, "mocks/tauri-window.ts") },
      { find: "@tauri-apps/plugin-dialog", replacement: path.resolve(__dirname, "mocks/plugin-dialog.ts") },
      { find: "@tauri-apps/plugin-fs", replacement: path.resolve(__dirname, "mocks/plugin-fs.ts") },
      { find: "@tauri-apps/plugin-deep-link", replacement: path.resolve(__dirname, "mocks/plugin-deep-link.ts") },
      { find: "@tauri-apps/plugin-shell", replacement: path.resolve(__dirname, "mocks/plugin-shell.ts") },
      { find: "@tauri-apps/plugin-updater", replacement: path.resolve(__dirname, "mocks/plugin-updater.ts") },
    ],
  },
  server: {
    fs: {
      allow: [path.resolve(__dirname, "..")],
    },
  },
  test: {
    environment: "jsdom",
    globals: true,
    include: ["src/**/*.{test,spec}.{ts,tsx}"],
    exclude: ["**/node_modules/**"],
    setupFiles: ["./vitest-setup.ts"],
    deps: {
      inline: ["@testing-library/jest-dom"],
    },
  },
});
