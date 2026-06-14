@slice1 @byok @mock-langdock
Feature: LangDock BYOK backend smoke
  Integration coverage for LIE-57–59 backend paths (Rust tests cover full byok.feature.md).

  Background:
    Given LangDock HTTP is mocked via the test harness

  Scenario: LangDock mock returns 200 for models probe
    When the harness probes LangDock models endpoint
    Then the probe returns status 200
