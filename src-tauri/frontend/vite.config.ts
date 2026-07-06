/// <reference types="vitest" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { visualizer } from "rollup-plugin-visualizer";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  test: {
    environment: "jsdom",
    globals: true,
    include: ["src/__tests__/**/*.test.{ts,tsx}"],
    setupFiles: ["src/__tests__/setup.ts"],
  },
  plugins: [
    react(),
    visualizer({ filename: "dist/stats.html", open: false }),
  ],
  clearScreen: false,
  server: {
    host: host || false,
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: process.env.TAURI_PLATFORM === "windows" ? "chrome105" : "safari14",
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    rollupOptions: {
      output: {
        manualChunks(id: string) {
          if (id.includes("node_modules/react/") || id.includes("node_modules/react-dom/") || id.includes("node_modules/zustand/")) {
            return "vendor";
          }
          if (id.includes("node_modules/@xyflow/")) {
            return "editor";
          }
          if (id.includes("node_modules/@xterm/")) {
            return "terminal";
          }
          if (id.includes("node_modules/dompurify") || id.includes("node_modules/marked")) {
            return "markup";
          }
          if (id.includes("node_modules/@tauri-apps/")) {
            return "tauri";
          }
        },
      },
    },
    chunkSizeWarningLimit: 1000,
  },
});
