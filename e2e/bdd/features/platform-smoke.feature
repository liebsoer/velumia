@slice0 @platform @mock-langdock
Feature: Platform BDD harness smoke
  LIE-54 scaffold validation — LangDock mock probe only.

  Background:
    Given LangDock HTTP is mocked via the test harness

  Scenario: LangDock mock returns 200 for models probe
    When the harness probes LangDock models endpoint
    Then the probe returns status 200
