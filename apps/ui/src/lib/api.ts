import { invoke, isTauri } from "@tauri-apps/api/core";
import { webInvoke } from "./web-api";

export type {
  ConnectionWidgetState,
  LangdockProfile,
  LibraryCounts,
  ListPromptFilters,
  ProfileInput,
  PromptFolder,
  PromptSummary,
  TagSummary,
} from "./ipc-types";
import type { ConnectionWidgetState, LangdockProfile, LibraryCounts, ListPromptFilters, ProfileInput, PromptFolder, PromptSummary, TagSummary } from "./ipc-types";

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
    }),
  getPrompt: (promptId: string) => ipc<PromptSummary>("get_prompt", { promptId }),
  createPrompt: (title: string, folderId?: string | null) =>
    ipc<string>("create_prompt", { title, folderId: folderId ?? null }),
  updatePrompt: (promptId: string, title?: string, folderId?: string | null) =>
    ipc<PromptSummary>("update_prompt", {
      promptId,
      title: title ?? null,
      folderId: folderId === undefined ? null : folderId,
    }),
  trashPrompt: (promptId: string) => ipc<void>("trash_prompt", { promptId }),
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
};

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
