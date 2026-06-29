import { invoke } from "@tauri-apps/api/core";

export interface ConnectionWidgetState {
  status: "not_configured" | "connected" | "configuration_error" | "offline";
  message?: string;
}

export interface LangdockProfile {
  id: string;
  name: string;
  base_url: string;
  is_default: boolean;
  connection_status: ConnectionWidgetState["status"];
  last_tested_at?: string;
}

export interface ProfileInput {
  name: string;
  base_url?: string;
  api_key?: string;
  is_default?: boolean;
}

export interface LibraryCounts {
  prompts: number;
  agents: number;
  skills: number;
}

export interface TagSummary {
  id: string;
  name: string;
}

export interface PromptSummary {
  id: string;
  title: string;
  slug: string;
  folder_id: string | null;
  tags: TagSummary[];
  is_favorite: boolean;
  updated_at: string;
}

export interface PromptFolder {
  id: string;
  title: string;
  slug: string;
  parent_id: string | null;
}

export interface ListPromptFilters {
  folderId?: string | null;
  tagId?: string | null;
  favoritesOnly?: boolean;
}

export const api = {
  ping: () => invoke<string>("ping"),
  isFirstLaunch: () => invoke<boolean>("is_first_launch"),
  completeWizard: (skipped: boolean) => invoke<void>("complete_wizard", { skipped }),
  getConnectionWidget: () => invoke<ConnectionWidgetState>("get_connection_widget"),
  listProfiles: () => invoke<LangdockProfile[]>("list_langdock_profiles"),
  saveProfile: (input: ProfileInput, profileId?: string, testConnectivity = true) =>
    invoke<LangdockProfile>("save_langdock_profile", {
      input,
      profileId: profileId ?? null,
      testConnectivity,
    }),
  testConnection: (profileId: string) =>
    invoke<LangdockProfile>("test_langdock_connection", { profileId }),
  setDefaultProfile: (profileId: string) =>
    invoke<LangdockProfile>("set_default_langdock_profile", { profileId }),
  deleteProfile: (profileId: string) =>
    invoke<void>("delete_langdock_profile", { profileId }),
  listPrompts: (filters: ListPromptFilters = {}) =>
    invoke<PromptSummary[]>("list_prompts", {
      folderId: filters.folderId ?? null,
      tagId: filters.tagId ?? null,
      favoritesOnly: filters.favoritesOnly ?? null,
    }),
  getPrompt: (promptId: string) => invoke<PromptSummary>("get_prompt", { promptId }),
  createPrompt: (title: string, folderId?: string | null) =>
    invoke<string>("create_prompt", { title, folderId: folderId ?? null }),
  updatePrompt: (promptId: string, title?: string, folderId?: string | null) =>
    invoke<PromptSummary>("update_prompt", {
      promptId,
      title: title ?? null,
      folderId: folderId === undefined ? null : folderId,
    }),
  trashPrompt: (promptId: string) => invoke<void>("trash_prompt", { promptId }),
  listPromptFolders: () => invoke<PromptFolder[]>("list_prompt_folders"),
  createPromptFolder: (title: string, parentId?: string | null) =>
    invoke<PromptFolder>("create_prompt_folder", { title, parentId: parentId ?? null }),
  movePromptToFolder: (promptId: string, folderId?: string | null) =>
    invoke<PromptSummary>("move_prompt_to_folder", {
      promptId,
      folderId: folderId ?? null,
    }),
  listTags: () => invoke<TagSummary[]>("list_tags"),
  setPromptTags: (promptId: string, tagNames: string[]) =>
    invoke<PromptSummary>("set_prompt_tags", { promptId, tagNames }),
  addPromptTag: (promptId: string, tagName: string) =>
    invoke<PromptSummary>("add_prompt_tag", { promptId, tagName }),
  removePromptTag: (promptId: string, tagId: string) =>
    invoke<PromptSummary>("remove_prompt_tag", { promptId, tagId }),
  setPromptFavorite: (promptId: string) => invoke<void>("set_prompt_favorite", { promptId }),
  unsetPromptFavorite: (promptId: string) => invoke<void>("unset_prompt_favorite", { promptId }),
  canRunPrompt: () => invoke<boolean>("can_run_prompt"),
  seedSamples: () => invoke<void>("seed_starter_samples"),
  libraryCounts: () => invoke<LibraryCounts>("library_counts"),
  checkAuthorize: (action: string) =>
    invoke<{ allowed: boolean }>("check_authorize", { action }),
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
