import { Given, When, Then, setDefaultTimeout } from "@cucumber/cucumber";
import assert from "node:assert/strict";
import { execSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

setDefaultTimeout(120_000);

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const desktopDir = resolve(repoRoot, "apps/desktop");

/** Rust integration tests under apps/desktop/tests/prompt_runs_integration.rs */
const PROMPT_RUN_TESTS = new Set([
  "prompt_03_session_and_transcript_persisted",
  "prompt_03_stream_completion_against_mock",
  "prompt_04_run_blocked_when_not_connected",
  "prompt_12_stop_marks_stopped_and_retains_partial",
  "prompt_14_variables_block_empty_and_substitute",
  "prompt_14_allow_empty_variables",
  "prompt_16_delete_session_removes_row_and_file",
]);

function integrationTestFile(testName: string): string {
  return PROMPT_RUN_TESTS.has(testName) ? "prompt_runs_integration" : "prompts_integration";
}

Given("the prompt library test harness is ready", function () {
  this.harnessReady = true;
});

When("the prompt integration test {string} runs", function (testName: string) {
  assert.ok(this.harnessReady);
  const testFile = integrationTestFile(testName);
  try {
    execSync(`cargo test --test ${testFile} ${testName} -- --exact`, {
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

Then("the integration test passes", function () {
  assert.equal(this.testPassed, true, this.testError ?? "integration test failed");
});
