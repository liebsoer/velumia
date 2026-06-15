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
  createPrompt: (title: string) => invoke<string>("create_prompt", { title }),
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
