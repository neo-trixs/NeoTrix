import { defineConfig } from "vite";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  clearScreen: false,
  server: { port: 1422, strictPort: true },
  build: {
    outDir: "dist",
    target: "es2021",
    rollupOptions: {
      input: {
        main: resolve(root, "index.html"),
        settings: resolve(root, "settings.html"),
      },
    },
  },
});
