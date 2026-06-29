import type {
  ConnectionWidgetState,
  LangdockProfile,
  LibraryCounts,
  ListPromptFilters,
  ProfileInput,
  PromptFolder,
  PromptSummary,
  TagSummary,
} from "./ipc-types";

const STORAGE_KEY = "velumia.web-dev.v1";
const WEB_VERSION = "0.1.0-web";

interface StoredPrompt {
  id: string;
  title: string;
  slug: string;
  folder_id: string | null;
  tag_ids: string[];
  is_favorite: boolean;
  updated_at: string;
  trashed: boolean;
}

interface WebStore {
  wizardCompleted: boolean;
  profiles: LangdockProfile[];
  profileApiKeys: Record<string, string>;
  prompts: StoredPrompt[];
  folders: PromptFolder[];
  tags: TagSummary[];
}

function nowIso(): string {
  return new Date().toISOString();
}

function newId(): string {
  return crypto.randomUUID();
}

function slugFromId(prefix: string, id: string): string {
  return `${prefix}-${id.slice(0, 8)}`;
}

function loadStore(): WebStore {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) return JSON.parse(raw) as WebStore;
  } catch {
    /* ignore corrupt storage */
  }
  return {
    wizardCompleted: false,
    profiles: [],
    profileApiKeys: {},
    prompts: [],
    folders: [],
    tags: [],
  };
}

function saveStore(store: WebStore): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(store));
}

function toPromptSummary(store: WebStore, prompt: StoredPrompt): PromptSummary {
  const tagMap = new Map(store.tags.map((t) => [t.id, t]));
  return {
    id: prompt.id,
    title: prompt.title,
    slug: prompt.slug,
    folder_id: prompt.folder_id,
    tags: prompt.tag_ids
      .map((id) => tagMap.get(id))
      .filter((t): t is TagSummary => t !== undefined),
    is_favorite: prompt.is_favorite,
    updated_at: prompt.updated_at,
  };
}

function activePrompts(store: WebStore): StoredPrompt[] {
  return store.prompts.filter((p) => !p.trashed);
}

function findPrompt(store: WebStore, promptId: string): StoredPrompt {
  const prompt = store.prompts.find((p) => p.id === promptId && !p.trashed);
  if (!prompt) throw new Error("prompt not found");
  return prompt;
}

function findOrCreateTag(store: WebStore, name: string): TagSummary {
  const trimmed = name.trim();
  if (!trimmed) throw new Error("tag name is required");
  const existing = store.tags.find((t) => t.name.toLowerCase() === trimmed.toLowerCase());
  if (existing) return existing;
  const tag: TagSummary = { id: newId(), name: trimmed };
  store.tags.push(tag);
  return tag;
}

function listPromptsFiltered(
  store: WebStore,
  filters: ListPromptFilters = {},
): PromptSummary[] {
  let prompts = activePrompts(store);
  if (filters.folderId !== undefined && filters.folderId !== null) {
    prompts = prompts.filter((p) => p.folder_id === filters.folderId);
  }
  if (filters.tagId) {
    prompts = prompts.filter((p) => p.tag_ids.includes(filters.tagId!));
  }
  if (filters.favoritesOnly) {
    prompts = prompts.filter((p) => p.is_favorite);
  }
  return prompts
    .map((p) => toPromptSummary(store, p))
    .sort((a, b) => b.updated_at.localeCompare(a.updated_at));
}

function connectionWidget(store: WebStore): ConnectionWidgetState {
  const defaultProfile =
    store.profiles.find((p) => p.is_default) ?? store.profiles[0];
  if (!defaultProfile) {
    return { status: "not_configured", message: "No LangDock profile configured" };
  }
  return {
    status: defaultProfile.connection_status,
    message: defaultProfile.connection_status === "connected" ? undefined : "Web dev mode",
  };
}

async function probeLangDock(baseUrl: string, apiKey: string): Promise<ConnectionWidgetState["status"]> {
  try {
    const url = `${baseUrl.replace(/\/$/, "")}/agent/v1/models`;
    const res = await fetch(url, {
      headers: { Authorization: `Bearer ${apiKey}` },
    });
    if (res.ok) return "connected";
    if (res.status === 401 || res.status === 403) return "configuration_error";
    return "offline";
  } catch {
    return "offline";
  }
}

export async function webInvoke<T>(
  cmd: string,
  args: Record<string, unknown> = {},
): Promise<T> {
  const store = loadStore();

  switch (cmd) {
    case "ping":
      return WEB_VERSION as T;

    case "is_first_launch":
      return (!store.wizardCompleted) as T;

    case "complete_wizard": {
      store.wizardCompleted = true;
      saveStore(store);
      return undefined as T;
    }

    case "get_connection_widget":
      return connectionWidget(store) as T;

    case "list_langdock_profiles":
      return [...store.profiles] as T;

    case "save_langdock_profile": {
      const input = args.input as ProfileInput;
      const profileId = args.profileId as string | null | undefined;
      const testConnectivity = (args.testConnectivity as boolean | undefined) ?? true;
      const id = profileId ?? newId();
      let profile = store.profiles.find((p) => p.id === id);
      if (!profile) {
        profile = {
          id,
          name: input.name,
          base_url: input.base_url ?? "https://api.langdock.com",
          is_default: input.is_default ?? true,
          connection_status: "not_configured",
        };
        store.profiles.push(profile);
      } else {
        profile.name = input.name;
        if (input.base_url) profile.base_url = input.base_url;
        if (input.is_default !== undefined) profile.is_default = input.is_default;
      }
      if (input.is_default) {
        for (const p of store.profiles) p.is_default = p.id === id;
      }
      if (input.api_key) {
        store.profileApiKeys[id] = input.api_key;
      }
      if (testConnectivity && store.profileApiKeys[id]) {
        profile.connection_status = await probeLangDock(
          profile.base_url,
          store.profileApiKeys[id],
        );
        profile.last_tested_at = nowIso();
      }
      saveStore(store);
      return { ...profile } as T;
    }

    case "test_langdock_connection": {
      const profileId = args.profileId as string;
      const profile = store.profiles.find((p) => p.id === profileId);
      if (!profile) throw new Error("profile not found");
      const apiKey = store.profileApiKeys[profileId];
      if (!apiKey) {
        profile.connection_status = "configuration_error";
      } else {
        profile.connection_status = await probeLangDock(profile.base_url, apiKey);
      }
      profile.last_tested_at = nowIso();
      saveStore(store);
      return { ...profile } as T;
    }

    case "set_default_langdock_profile": {
      const profileId = args.profileId as string;
      const profile = store.profiles.find((p) => p.id === profileId);
      if (!profile) throw new Error("profile not found");
      for (const p of store.profiles) p.is_default = p.id === profileId;
      saveStore(store);
      return { ...profile } as T;
    }

    case "delete_langdock_profile": {
      const profileId = args.profileId as string;
      store.profiles = store.profiles.filter((p) => p.id !== profileId);
      delete store.profileApiKeys[profileId];
      saveStore(store);
      return undefined as T;
    }

    case "list_prompts":
      return listPromptsFiltered(store, {
        folderId: args.folderId as string | null | undefined,
        tagId: args.tagId as string | null | undefined,
        favoritesOnly: (args.favoritesOnly as boolean | null | undefined) ?? undefined,
      }) as T;

    case "get_prompt": {
      const prompt = findPrompt(store, args.promptId as string);
      return toPromptSummary(store, prompt) as T;
    }

    case "create_prompt": {
      const title = (args.title as string).trim();
      if (!title) throw new Error("title is required");
      const folderId = args.folderId as string | null | undefined;
      if (folderId && !store.folders.some((f) => f.id === folderId)) {
        throw new Error("folder not found");
      }
      const id = newId();
      const prompt: StoredPrompt = {
        id,
        title,
        slug: slugFromId("prompt", id),
        folder_id: folderId ?? null,
        tag_ids: [],
        is_favorite: false,
        updated_at: nowIso(),
        trashed: false,
      };
      store.prompts.push(prompt);
      saveStore(store);
      return id as T;
    }

    case "update_prompt": {
      const prompt = findPrompt(store, args.promptId as string);
      if (args.title !== undefined && args.title !== null) {
        const title = (args.title as string).trim();
        if (!title) throw new Error("title is required");
        prompt.title = title;
      }
      if (args.folderId !== undefined) {
        const folderId = args.folderId as string | null;
        if (folderId && !store.folders.some((f) => f.id === folderId)) {
          throw new Error("folder not found");
        }
        prompt.folder_id = folderId;
      }
      prompt.updated_at = nowIso();
      saveStore(store);
      return toPromptSummary(store, prompt) as T;
    }

    case "trash_prompt": {
      const prompt = findPrompt(store, args.promptId as string);
      prompt.trashed = true;
      prompt.updated_at = nowIso();
      saveStore(store);
      return undefined as T;
    }

    case "list_prompt_folders":
      return [...store.folders].sort((a, b) => a.title.localeCompare(b.title)) as T;

    case "create_prompt_folder": {
      const title = (args.title as string).trim();
      if (!title) throw new Error("title is required");
      const parentId = args.parentId as string | null | undefined;
      if (parentId) {
        const parent = store.folders.find((f) => f.id === parentId);
        if (!parent) throw new Error("parent folder not found");
        if (parent.parent_id) throw new Error("folder nesting limited to two levels");
      }
      const id = newId();
      const folder: PromptFolder = {
        id,
        title,
        slug: slugFromId("folder", id),
        parent_id: parentId ?? null,
      };
      store.folders.push(folder);
      saveStore(store);
      return folder as T;
    }

    case "move_prompt_to_folder": {
      const prompt = findPrompt(store, args.promptId as string);
      const folderId = args.folderId as string | null | undefined;
      if (folderId && !store.folders.some((f) => f.id === folderId)) {
        throw new Error("folder not found");
      }
      prompt.folder_id = folderId ?? null;
      prompt.updated_at = nowIso();
      saveStore(store);
      return toPromptSummary(store, prompt) as T;
    }

    case "list_tags":
      return [...store.tags].sort((a, b) => a.name.localeCompare(b.name)) as T;

    case "set_prompt_tags": {
      const prompt = findPrompt(store, args.promptId as string);
      const tagNames = args.tagNames as string[];
      prompt.tag_ids = tagNames.map((name) => findOrCreateTag(store, name).id);
      prompt.updated_at = nowIso();
      saveStore(store);
      return toPromptSummary(store, prompt) as T;
    }

    case "add_prompt_tag": {
      const prompt = findPrompt(store, args.promptId as string);
      const tag = findOrCreateTag(store, args.tagName as string);
      if (!prompt.tag_ids.includes(tag.id)) prompt.tag_ids.push(tag.id);
      prompt.updated_at = nowIso();
      saveStore(store);
      return toPromptSummary(store, prompt) as T;
    }

    case "remove_prompt_tag": {
      const prompt = findPrompt(store, args.promptId as string);
      const tagId = args.tagId as string;
      prompt.tag_ids = prompt.tag_ids.filter((id) => id !== tagId);
      prompt.updated_at = nowIso();
      saveStore(store);
      return toPromptSummary(store, prompt) as T;
    }

    case "set_prompt_favorite": {
      const prompt = findPrompt(store, args.promptId as string);
      prompt.is_favorite = true;
      prompt.updated_at = nowIso();
      saveStore(store);
      return undefined as T;
    }

    case "unset_prompt_favorite": {
      const prompt = findPrompt(store, args.promptId as string);
      prompt.is_favorite = false;
      prompt.updated_at = nowIso();
      saveStore(store);
      return undefined as T;
    }

    case "can_run_prompt": {
      const widget = connectionWidget(store);
      return (widget.status === "connected") as T;
    }

    case "seed_starter_samples": {
      const id = newId();
      store.prompts.push({
        id,
        title: "Sample prompt",
        slug: slugFromId("prompt", id),
        folder_id: null,
        tag_ids: [],
        is_favorite: false,
        updated_at: nowIso(),
        trashed: false,
      });
      saveStore(store);
      return undefined as T;
    }

    case "library_counts": {
      const counts: LibraryCounts = {
        prompts: activePrompts(store).length,
        agents: 0,
        skills: 0,
      };
      return counts as T;
    }

    case "check_authorize":
      return { allowed: true } as T;

    default:
      throw new Error(`Unknown web command: ${cmd}`);
  }
}
