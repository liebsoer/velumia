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

export type PromptContentSyntax = "auto" | "plaintext" | "markdown" | "xml" | "json";

export interface PromptSummary {
  id: string;
  title: string;
  slug: string;
  folder_id: string | null;
  tags: TagSummary[];
  is_favorite: boolean;
  content_syntax: PromptContentSyntax;
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

export interface PromptVersionSummary {
  id: string;
  prompt_id: string;
  version_number: number;
  created_at: string;
  is_head: boolean;
}
