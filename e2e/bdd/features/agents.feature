@slice3 @agents
Feature: Agent library CRUD, attachments, sub-agents
  LIE-65 — domain scenarios backed by Rust integration tests.

  Background:
    Given the agent library test harness is ready

  Scenario: Create agent in library
    When the agent integration test "agent_10_create_in_library" runs
    Then the integration test passes

  Scenario: Edit agent and attach prompts
    When the agent integration test "agent_11_edit_and_attach_prompts" runs
    Then the integration test passes

  Scenario: Link sub-agents with one-level depth rule
    When the agent integration test "agent_15_subagents_one_level" runs
    Then the integration test passes
