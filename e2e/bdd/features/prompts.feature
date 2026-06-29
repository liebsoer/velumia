@slice2 @prompt-library
Feature: Prompt library CRUD, folders, tags, favorites
  LIE-60 — domain scenarios backed by Rust integration tests.

  Background:
    Given the prompt library test harness is ready

  # PROMPT-06
  Scenario: Create prompt in library
    When the prompt integration test "prompt_06_create_in_library" runs
    Then the integration test passes

  # PROMPT-07
  Scenario: Folder organization with two-level nesting
    When the prompt integration test "prompt_07_folder_two_level_nesting" runs
    Then the integration test passes

  # PROMPT-08
  Scenario: Tag prompts
    When the prompt integration test "prompt_08_tag_prompts" runs
    Then the integration test passes

  # PROMPT-09
  Scenario: Favorite and unfavorite prompts
    When the prompt integration test "prompt_09_favorite_unfavorite" runs
    Then the integration test passes

  # PROMPT-13
  Scenario: List and filter library by folder tag and favorites
    When the prompt integration test "prompt_13_list_and_filter" runs
    Then the integration test passes
