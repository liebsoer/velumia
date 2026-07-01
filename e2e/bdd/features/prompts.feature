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

  # PROMPT-01
  Scenario: Version on save
    When the prompt integration test "prompt_01_version_on_save" runs
    Then the integration test passes

  # PROMPT-02
  Scenario: Version list and history
    When the prompt integration test "prompt_02_version_list" runs
    Then the integration test passes

  # PROMPT-15
  Scenario: Diff and restore as new head
    When the prompt integration test "prompt_15_diff_and_restore" runs
    Then the integration test passes

  # PROMPT-04 — Run blocked when LangDock degraded (domain; UI copy in PromptRunPanel)
  Scenario: Prompt run blocked when connection is degraded
    When the prompt integration test "prompt_04_run_blocked_when_not_connected" runs
    Then the integration test passes

  # PROMPT-03 — Stream run, session persistence, multi-turn
  Scenario: User streams a prompt run with multi-turn session persistence
    When the prompt integration test "prompt_03_session_and_transcript_persisted" runs
    Then the integration test passes
    When the prompt integration test "prompt_03_stream_completion_against_mock" runs
    Then the integration test passes

  # PROMPT-12 — Stop streaming run
  Scenario: User stops an in-progress streaming run
    When the prompt integration test "prompt_12_stop_marks_stopped_and_retains_partial" runs
    Then the integration test passes

  # PROMPT-14 — Variables at run
  Scenario: Variables modal blocks run until values are provided
    When the prompt integration test "prompt_14_variables_block_empty_and_substitute" runs
    Then the integration test passes

  Scenario: User may continue with empty variable values
    When the prompt integration test "prompt_14_allow_empty_variables" runs
    Then the integration test passes

  # PROMPT-16 — Delete session
  Scenario: User deletes a completed prompt session
    When the prompt integration test "prompt_16_delete_session_removes_row_and_file" runs
    Then the integration test passes

  # PROMPT-10 — Archive / unarchive
  Scenario: Archive and unarchive a prompt
    When the prompt integration test "prompt_10_archive_unarchive" runs
    Then the integration test passes

  # PROMPT-11 — Trash / restore
  Scenario: Trash and restore a prompt
    When the prompt integration test "prompt_11_trash_restore" runs
    Then the integration test passes

  # PROMPT-17 — Authz deny write
  Scenario: Write operations denied when authz stub denies prompt write
    When the prompt integration test "prompt_17_authz_deny_write" runs
    Then the integration test passes

  # PROMPT-18 — Authz deny execute
  Scenario: Run denied when authz stub denies prompt execute
    When the prompt integration test "prompt_18_authz_deny_execute" runs
    Then the integration test passes

  # X-01 — Durability
  Scenario: Prompt library survives app restart
    When the prompt integration test "x_01_durability_reopen_database" runs
    Then the integration test passes

  # X-02 — Irreversible actions (trash confirm)
  Scenario: Trash requires confirmation
    When the prompt integration test "x_02_trash_requires_confirmation" runs
    Then the integration test passes
