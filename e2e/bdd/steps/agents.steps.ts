import { Given, When } from "@cucumber/cucumber";
import assert from "node:assert/strict";
import { execSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const desktopDir = resolve(repoRoot, "apps/desktop");

Given("the agent library test harness is ready", function () {
  this.harnessReady = true;
});

When("the agent integration test {string} runs", function (testName: string) {
  assert.ok(this.harnessReady);
  try {
    execSync(`cargo test --test agents_integration ${testName} -- --exact`, {
      cwd: desktopDir,
      stdio: "pipe",
      encoding: "utf8",
    });
    this.testPassed = true;
  } catch (err) {
    this.testPassed = false;
    this.testError = err instanceof Error ? err.message : String(err);
  }
});
