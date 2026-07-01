import { defineConfig } from "vitest/config";
import vue from "@vitejs/plugin-vue";
import { resolve } from "node:path";

const uiRoot = resolve(__dirname);
const repoRoot = resolve(__dirname, "../..");

export default defineConfig({
  root: uiRoot,
  plugins: [vue()],
  resolve: {
    alias: {
      "@velumia/ui-run-panel": resolve(repoRoot, "libs/ui/run-panel/src/index.ts"),
    },
  },
  test: {
    environment: "happy-dom",
    include: ["src/**/*.test.ts"],
  },
});
