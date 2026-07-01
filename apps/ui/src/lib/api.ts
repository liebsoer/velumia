import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { webInvoke, subscribePromptRunEvents } from "./web-api";

export type {
  ConnectionWidgetState,
  LangdockProfile,
  LibraryCounts,
  ListPromptFilters,
  ProfileInput,
  PromptContentSyntax,
  PromptFolder,
  PromptLifecycleStatus,
  PromptSummary,
  PromptVersionSummary,
  PromptRunChunkPayload,
  PromptRunErrorPayload,
  PromptRunSessionPayload,
  SendPromptMessageInput,
  SessionSummary,
  StartPromptRunInput,
  StartPromptRunResult,
  StopPromptRunInput,
  TagSummary,
  TranscriptLine,
} from "./ipc-types";
export { PROMPT_RUN_EVENTS } from "./ipc-types";
import type {
  ConnectionWidgetState,
  LangdockProfile,
  LibraryCounts,
  ListPromptFilters,
  ProfileInput,
  PromptContentSyntax,
  PromptFolder,
  PromptSummary,
  PromptVersionSummary,
  SendPromptMessageInput,
  SessionSummary,
  StartPromptRunInput,
  StartPromptRunResult,
  StopPromptRunInput,
  TagSummary,
  TranscriptLine,
} from "./ipc-types";

function hasTauriBridge(): boolean {
  return typeof window !== "undefined" && !!window.__TAURI_INTERNALS__;
}

async function ipc<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
  if (isTauri() && hasTauriBridge()) {
    return invoke<T>(cmd, args);
  }
  return webInvoke<T>(cmd, args);
}

export const api = {
  ping: () => ipc<string>("ping"),
  isFirstLaunch: () => ipc<boolean>("is_first_launch"),
  completeWizard: (skipped: boolean) => ipc<void>("complete_wizard", { skipped }),
  getConnectionWidget: () => ipc<ConnectionWidgetState>("get_connection_widget"),
  listProfiles: () => ipc<LangdockProfile[]>("list_langdock_profiles"),
  saveProfile: (input: ProfileInput, profileId?: string, testConnectivity = true) =>
    ipc<LangdockProfile>("save_langdock_profile", {
      input,
      profileId: profileId ?? null,
      testConnectivity,
    }),
  testConnection: (profileId: string) =>
    ipc<LangdockProfile>("test_langdock_connection", { profileId }),
  setDefaultProfile: (profileId: string) =>
    ipc<LangdockProfile>("set_default_langdock_profile", { profileId }),
  deleteProfile: (profileId: string) =>
    ipc<void>("delete_langdock_profile", { profileId }),
  listPrompts: (filters: ListPromptFilters = {}) =>
    ipc<PromptSummary[]>("list_prompts", {
      folderId: filters.folderId ?? null,
      tagId: filters.tagId ?? null,
      favoritesOnly: filters.favoritesOnly ?? null,
      lifecycleFilter: filters.lifecycleFilter ?? null,
    }),
  getPrompt: (promptId: string) => ipc<PromptSummary>("get_prompt", { promptId }),
  createPrompt: (title: string, folderId?: string | null) =>
    ipc<string>("create_prompt", { title, folderId: folderId ?? null }),
  updatePrompt: (
    promptId: string,
    options: {
      title?: string;
      folderId?: string | null;
      contentSyntax?: PromptContentSyntax;
    } = {},
  ) => {
    const args: Record<string, unknown> = { promptId };
    if (options.title !== undefined) args.title = options.title;
    if (options.folderId !== undefined) args.folderId = options.folderId;
    if (options.contentSyntax !== undefined) args.contentSyntax = options.contentSyntax;
    return ipc<PromptSummary>("update_prompt", args);
  },
  trashPrompt: (promptId: string) => ipc<void>("trash_prompt", { promptId }),
  archivePrompt: (promptId: string) => ipc<void>("archive_prompt", { promptId }),
  unarchivePrompt: (promptId: string) => ipc<void>("unarchive_prompt", { promptId }),
  restorePrompt: (promptId: string) => ipc<void>("restore_prompt", { promptId }),
  listPromptFolders: () => ipc<PromptFolder[]>("list_prompt_folders"),
  createPromptFolder: (title: string, parentId?: string | null) =>
    ipc<PromptFolder>("create_prompt_folder", { title, parentId: parentId ?? null }),
  movePromptToFolder: (promptId: string, folderId?: string | null) =>
    ipc<PromptSummary>("move_prompt_to_folder", {
      promptId,
      folderId: folderId ?? null,
    }),
  listTags: () => ipc<TagSummary[]>("list_tags"),
  setPromptTags: (promptId: string, tagNames: string[]) =>
    ipc<PromptSummary>("set_prompt_tags", { promptId, tagNames }),
  addPromptTag: (promptId: string, tagName: string) =>
    ipc<PromptSummary>("add_prompt_tag", { promptId, tagName }),
  removePromptTag: (promptId: string, tagId: string) =>
    ipc<PromptSummary>("remove_prompt_tag", { promptId, tagId }),
  setPromptFavorite: (promptId: string) => ipc<void>("set_prompt_favorite", { promptId }),
  unsetPromptFavorite: (promptId: string) => ipc<void>("unset_prompt_favorite", { promptId }),
  canRunPrompt: () => ipc<boolean>("can_run_prompt"),
  seedSamples: () => ipc<void>("seed_starter_samples"),
  libraryCounts: () => ipc<LibraryCounts>("library_counts"),
  checkAuthorize: (action: string) =>
    ipc<{ allowed: boolean }>("check_authorize", { action }),
  savePromptContent: (promptId: string, content: string) =>
    ipc<PromptVersionSummary>("save_prompt_content", { promptId, content }),
  listPromptVersions: (promptId: string) =>
    ipc<PromptVersionSummary[]>("list_prompt_versions", { promptId }),
  getPromptVersionContent: (versionId: string) =>
    ipc<string>("get_prompt_version_content", { versionId }),
  restorePromptVersion: (promptId: string, versionId: string) =>
    ipc<PromptVersionSummary>("restore_prompt_version", { promptId, versionId }),
  startPromptRun: (input: StartPromptRunInput) =>
    ipc<StartPromptRunResult>("start_prompt_run", { input }),
  sendPromptMessage: (input: SendPromptMessageInput) =>
    ipc<StartPromptRunResult>("send_prompt_message", { input }),
  stopPromptRun: (input: StopPromptRunInput) =>
    ipc<void>("stop_prompt_run", { input }),
  listPromptSessions: (promptId: string) =>
    ipc<SessionSummary[]>("list_prompt_sessions", { promptId }),
  getSessionTranscript: (promptId: string, sessionId: string) =>
    ipc<TranscriptLine[]>("get_session_transcript", { promptId, sessionId }),
  deletePromptSession: (promptId: string, sessionId: string) =>
    ipc<void>("delete_prompt_session", { promptId, sessionId }),
};

export function onPromptRunEvent<T>(
  event: string,
  handler: (payload: T) => void,
): () => void {
  if (isTauri() && hasTauriBridge()) {
    let unlisten: UnlistenFn | undefined;
    void listen<T>(event, (e) => handler(e.payload)).then((fn) => {
      unlisten = fn;
    });
    return () => {
      void unlisten?.();
    };
  }
  return subscribePromptRunEvents((ev, payload) => {
    if (ev === event) handler(payload as T);
  });
}

export function statusLabel(status: ConnectionWidgetState["status"]): string {
  switch (status) {
    case "connected":
      return "Connected";
    case "configuration_error":
      return "Configuration error";
    case "offline":
      return "Offline";
    default:
      return "Not configured";
  }
}

export function statusClass(status: ConnectionWidgetState["status"]): string {
  switch (status) {
    case "connected":
      return "status-success";
    case "configuration_error":
      return "status-error";
    case "offline":
      return "status-warning";
    default:
      return "status-muted";
  }
}

declare global {
  interface Window {
    __TAURI_INTERNALS__?: { invoke: typeof invoke };
  }
}
