import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { resolve } from "node:path";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

const uiRoot = resolve(__dirname);
const repoRoot = resolve(__dirname, "../..");

// https://vite.dev/config/
export default defineConfig(async () => ({
  root: uiRoot,
  plugins: [vue()],
  resolve: {
    alias: {
      "@velumia/ui-run-panel": resolve(repoRoot, "libs/ui/run-panel/src/index.ts"),
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching the Tauri Rust project
      ignored: ["**/apps/desktop/**"],
    },
  },
  build: {
    outDir: resolve(uiRoot, "dist"),
    emptyOutDir: true,
  },
}));
