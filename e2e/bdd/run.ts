import { runCucumber } from "@cucumber/cucumber/api";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));

const tagExpression = process.env.BDD_TAGS?.trim() || "";

const { success } = await runCucumber({
  sources: {
    defaultDialect: "en",
    paths: [resolve(__dirname, "features/**/*.feature")],
    names: [],
    tagExpression,
    order: "defined",
  },
  support: {
    importPaths: [resolve(__dirname, "steps/**/*.ts")],
  },
  runtime: {
    dryRun: false,
    failFast: false,
    filterStacktraces: true,
    parallel: 0,
    retry: 0,
    retryTagFilter: "",
    strict: false,
    worldParameters: {},
  },
  formats: {
    stdout: "progress",
    files: {},
    publish: false,
    options: {},
  },
});

process.exit(success ? 0 : 1);
