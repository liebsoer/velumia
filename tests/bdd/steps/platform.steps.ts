import { Before, After, Given, When, Then, setDefaultTimeout } from "@cucumber/cucumber";
import assert from "node:assert/strict";
import {
  langDockMockUrl,
  probeLangDockMock,
  startLangDockMock,
  stopLangDockMock,
} from "../support/langdock-mock.js";
import type http from "node:http";

setDefaultTimeout(10_000);

let mockServer: http.Server | undefined;

Before({ tags: "@mock-langdock" }, async function () {
  mockServer = await startLangDockMock();
});

After({ tags: "@mock-langdock" }, async function () {
  if (mockServer) {
    await stopLangDockMock(mockServer);
    mockServer = undefined;
  }
});

Given("LangDock HTTP is mocked via the test harness", function () {
  assert.ok(mockServer, "LangDock mock server should be running");
});

When("the harness probes LangDock models endpoint", async function () {
  this.probeStatus = await probeLangDockMock();
});

Then("the probe returns status {int}", function (expected: number) {
  assert.equal(this.probeStatus, expected);
});

Given("the mock LangDock base URL is configured", function () {
  this.mockBaseUrl = langDockMockUrl();
  assert.ok(this.mockBaseUrl.startsWith("http://127.0.0.1:"));
});
