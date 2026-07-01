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
  SessionSummary,
  StartPromptRunResult,
  TagSummary,
  TranscriptLine,
  AgentDetail,
  AgentSummary,
} from "./ipc-types";
import { PROMPT_RUN_EVENTS } from "./ipc-types";

const STORAGE_KEY = "velumia.web-dev.v1";
const WEB_VERSION = "0.1.0-web";
const MAX_CONTENT_LEN = 512 * 1024;
const MAX_TRANSCRIPT_BYTES = 4 * 1024 * 1024;
/** Session-only; never persisted (S1 — web dev key storage). */
const webProfileApiKeys = new Map<string, string>();
const AUTHZ_STUB_DENY_WRITE_KEY = "VELUMIA_AUTHZ_STUB_DENY";
const AUTHZ_STUB_DENY_EXECUTE_KEY = "VELUMIA_AUTHZ_STUB_DENY_EXECUTE";

interface StoredPrompt {
  id: string;
  title: string;
  slug: string;
  folder_id: string | null;
  tag_ids: string[];
  is_favorite: boolean;
  updated_at: string;
  trashed: boolean;
  lifecycle_status?: "active" | "archived" | "trashed";
  head_version_id: string | null;
  content_syntax: PromptContentSyntax;
}

interface StoredPromptVersion {
  id: string;
  prompt_id: string;
  version_number: number;
  content: string;
  created_at: string;
}

interface StoredSession {
  id: string;
  prompt_id: string;
  created_at: string;
  updated_at: string;
  stopped: boolean;
  transcript: TranscriptLine[];
}

interface StoredAgent {
  id: string;
  title: string;
  slug: string;
  lifecycle_status: "active" | "archived" | "trashed";
  instructions: string;
  model: string;
  web_search: boolean;
  prompt_ids: string[];
  skill_ids: string[];
  subagent_ids: string[];
  updated_at: string;
}

interface WebStore {
  wizardCompleted: boolean;
  profiles: LangdockProfile[];
  prompts: StoredPrompt[];
  promptVersions: StoredPromptVersion[];
  folders: PromptFolder[];
  tags: TagSummary[];
  sessions: StoredSession[];
  agents: StoredAgent[];
}

type PromptRunListener = (event: string, payload: unknown) => void;
const promptRunListeners = new Set<PromptRunListener>();

export function subscribePromptRunEvents(listener: PromptRunListener): () => void {
  promptRunListeners.add(listener);
  return () => promptRunListeners.delete(listener);
}

function emitPromptRunEvent(event: string, payload: unknown): void {
  for (const listener of promptRunListeners) listener(event, payload);
}

const STREAM_CHUNK_SIZE = 4;
const STREAM_CHUNK_DELAY_MS = 25;

interface WebActiveRun {
  sessionId: string;
  runId: string;
  promptId: string;
  aborted: boolean;
  timeouts: ReturnType<typeof setTimeout>[];
}

const activeWebRuns = new Map<string, WebActiveRun>();

function nowIso(): string {
  return new Date().toISOString();
}

function newId(): string {
  return crypto.randomUUID();
}

function slugFromId(prefix: string, id: string): string {
  return `${prefix}-${id.slice(0, 8)}`;
}

type WebAuthzResult =
  | { allowed: true }
  | { allowed: false; reason: string; permission: string };

function authzStubEnabled(key: string): boolean {
  try {
    return sessionStorage.getItem(key) === "1";
  } catch {
    return false;
  }
}

function webAuthorize(action: string): WebAuthzResult {
  if (authzStubEnabled(AUTHZ_STUB_DENY_WRITE_KEY) && action === "prompt:write") {
    return { allowed: false, reason: "permission_denied", permission: action };
  }
  if (authzStubEnabled(AUTHZ_STUB_DENY_EXECUTE_KEY) && action === "prompt:execute") {
    return { allowed: false, reason: "permission_denied", permission: action };
  }
  return { allowed: true };
}

function requireWebAuthorize(action: string): void {
  const result = webAuthorize(action);
  if (!result.allowed) {
    throw new Error("permission denied");
  }
}

function migrateLegacyStore(raw: Record<string, unknown>): WebStore {
  const legacyKeys = raw.profileApiKeys;
  if (legacyKeys && typeof legacyKeys === "object" && !Array.isArray(legacyKeys)) {
    for (const [id, key] of Object.entries(legacyKeys as Record<string, string>)) {
      if (key) webProfileApiKeys.set(id, key);
    }
    delete raw.profileApiKeys;
  }
  return normalizeStore(raw as unknown as WebStore);
}

function loadStore(): WebStore {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) return migrateLegacyStore(JSON.parse(raw) as Record<string, unknown>);
  } catch {
    /* ignore corrupt storage */
  }
  return normalizeStore({
    wizardCompleted: false,
    profiles: [],
    prompts: [],
    promptVersions: [],
    folders: [],
    tags: [],
    sessions: [],
    agents: [],
  });
}

function activeAgents(store: WebStore): StoredAgent[] {
  return store.agents.filter((a) => a.lifecycle_status === "active");
}

function findAgent(store: WebStore, agentId: string): StoredAgent {
  const agent = store.agents.find((a) => a.id === agentId);
  if (!agent) throw new Error("agent not found");
  return agent;
}

function toAgentSummary(agent: StoredAgent): AgentSummary {
  return {
    id: agent.id,
    title: agent.title,
    slug: agent.slug,
    lifecycle_status: agent.lifecycle_status,
    model: agent.model,
    updated_at: agent.updated_at,
  };
}

function toAgentDetail(store: WebStore, agent: StoredAgent): AgentDetail {
  const promptMap = new Map(activePrompts(store).map((p) => [p.id, p]));
  return {
    id: agent.id,
    title: agent.title,
    slug: agent.slug,
    lifecycle_status: agent.lifecycle_status,
    model: agent.model,
    web_search: agent.web_search,
    updated_at: agent.updated_at,
    instructions: agent.instructions,
    prompts: agent.prompt_ids.map((promptId, sort_order) => ({
      prompt_id: promptId,
      title: promptMap.get(promptId)?.title ?? promptId,
      sort_order,
    })),
    skills: agent.skill_ids.map((skill_id, sort_order) => ({
      skill_id,
      title: skill_id,
      sort_order,
    })),
    subagents: agent.subagent_ids
      .map((agentId) => store.agents.find((a) => a.id === agentId))
      .filter((a): a is StoredAgent => a !== undefined)
      .map((a) => ({ agent_id: a.id, title: a.title })),
  };
}

function assertSubagentLinkable(store: WebStore, parentId: string, childId: string): void {
  if (parentId === childId) throw new Error("agent cannot be its own sub-agent");
  const child = findAgent(store, childId);
  if (child.lifecycle_status !== "active") throw new Error("sub-agent must be active");
  if (child.subagent_ids.length > 0) {
    throw new Error("sub-agent cannot have its own sub-agents");
  }
  if (store.agents.some((a) => a.subagent_ids.includes(parentId))) {
    throw new Error("agent that is a sub-agent cannot have sub-agents");
  }
  const otherParent = store.agents.find(
    (a) => a.id !== parentId && a.subagent_ids.includes(childId),
  );
  if (otherParent) throw new Error("sub-agent is already linked to another agent");
}

function normalizeStore(store: WebStore): WebStore {
  if (!store.promptVersions) store.promptVersions = [];
  if (!store.sessions) store.sessions = [];
  if (!store.agents) store.agents = [];
  for (const prompt of store.prompts) {
    if (!prompt.lifecycle_status) {
      prompt.lifecycle_status = prompt.trashed ? "trashed" : "active";
    }
    if (prompt.head_version_id === undefined) prompt.head_version_id = null;
    if (!prompt.content_syntax) prompt.content_syntax = "auto";
  }
  return store;
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
    content_syntax: prompt.content_syntax ?? "auto",
    updated_at: prompt.updated_at,
    lifecycle_status: prompt.lifecycle_status ?? (prompt.trashed ? "trashed" : "active"),
  };
}

function promptsWithLifecycle(
  store: WebStore,
  lifecycle: "active" | "archived" | "trashed",
): StoredPrompt[] {
  return store.prompts.filter((p) => {
    const status = p.lifecycle_status ?? (p.trashed ? "trashed" : "active");
    return status === lifecycle;
  });
}

function activePrompts(store: WebStore): StoredPrompt[] {
  return promptsWithLifecycle(store, "active");
}

function findAnyPrompt(store: WebStore, promptId: string): StoredPrompt {
  const prompt = store.prompts.find((p) => p.id === promptId);
  if (!prompt) throw new Error("prompt not found");
  return prompt;
}

function findActivePrompt(store: WebStore, promptId: string): StoredPrompt {
  const prompt = findAnyPrompt(store, promptId);
  const status = prompt.lifecycle_status ?? (prompt.trashed ? "trashed" : "active");
  if (status !== "active") throw new Error("prompt not found");
  return prompt;
}

function findPrompt(store: WebStore, promptId: string): StoredPrompt {
  return findActivePrompt(store, promptId);
}

function versionSummary(
  store: WebStore,
  version: StoredPromptVersion,
): PromptVersionSummary {
  const prompt = store.prompts.find((p) => p.id === version.prompt_id);
  return {
    id: version.id,
    prompt_id: version.prompt_id,
    version_number: version.version_number,
    created_at: version.created_at,
    is_head: prompt?.head_version_id === version.id,
  };
}

function appendPromptVersion(
  store: WebStore,
  promptId: string,
  content: string,
): PromptVersionSummary {
  const prompt = findPrompt(store, promptId);
  const existing = store.promptVersions.filter((v) => v.prompt_id === promptId);
  const nextNumber =
    existing.length > 0
      ? Math.max(...existing.map((v) => v.version_number)) + 1
      : 1;
  const id = newId();
  const createdAt = nowIso();
  const version: StoredPromptVersion = {
    id,
    prompt_id: promptId,
    version_number: nextNumber,
    content,
    created_at: createdAt,
  };
  store.promptVersions.push(version);
  prompt.head_version_id = id;
  prompt.updated_at = createdAt;
  return versionSummary(store, version);
}

function listVersionsForPrompt(
  store: WebStore,
  promptId: string,
): PromptVersionSummary[] {
  findPrompt(store, promptId);
  return store.promptVersions
    .filter((v) => v.prompt_id === promptId)
    .sort((a, b) => b.version_number - a.version_number)
    .map((v) => versionSummary(store, v));
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
  const lifecycle = filters.lifecycleFilter ?? "active";
  let prompts = promptsWithLifecycle(store, lifecycle);
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

function parseVariablePlaceholders(content: string): string[] {
  const names: string[] = [];
  const seen = new Set<string>();
  let i = 0;
  while (i + 4 <= content.length) {
    if (content[i] === "{" && content[i + 1] === "{") {
      const end = content.indexOf("}}", i + 2);
      if (end === -1) break;
      const name = content.slice(i + 2, end).trim();
      if (name && !seen.has(name)) {
        seen.add(name);
        names.push(name);
      }
      i = end + 2;
      continue;
    }
    i += 1;
  }
  return names;
}

function validateAndSubstitute(
  content: string,
  variables: Record<string, string> | undefined,
  allowEmptyVariables: boolean,
): string {
  const placeholders = parseVariablePlaceholders(content);
  const placeholderSet = new Set(placeholders);
  const variableKeys = new Set(Object.keys(variables ?? {}));
  if (
    placeholderSet.size !== variableKeys.size ||
    [...placeholderSet].some((k) => !variableKeys.has(k))
  ) {
    throw new Error("variables do not match prompt placeholders");
  }
  for (const name of placeholders) {
    const value = variables?.[name] ?? "";
    if (!value && !allowEmptyVariables) {
      throw new Error(`variable '${name}' is empty`);
    }
  }
  let out = content;
  for (const name of placeholders) {
    const value = variables?.[name] ?? "";
    out = out.split(`{{${name}}}`).join(value);
  }
  return out;
}

function headContentForPrompt(store: WebStore, promptId: string): string {
  const prompt = findPrompt(store, promptId);
  if (!prompt.head_version_id) return "";
  const version = store.promptVersions.find((v) => v.id === prompt.head_version_id);
  return version?.content ?? "";
}

function toSessionSummary(session: StoredSession): SessionSummary {
  return {
    id: session.id,
    prompt_id: session.prompt_id,
    created_at: session.created_at,
    updated_at: session.updated_at,
    stopped: session.stopped,
  };
}

function transcriptByteSize(lines: TranscriptLine[]): number {
  return new TextEncoder().encode(JSON.stringify(lines)).length;
}

function findSession(store: WebStore, promptId: string, sessionId: string): StoredSession {
  const session = store.sessions.find((s) => s.id === sessionId && s.prompt_id === promptId);
  if (!session) throw new Error("session not found");
  return session;
}

function mockReplyForUserMessage(userMessage: string): string {
  return `mock-reply:${userMessage}`;
}

function chunkText(text: string): string[] {
  const chunks: string[] = [];
  for (let i = 0; i < text.length; i += STREAM_CHUNK_SIZE) {
    chunks.push(text.slice(i, i + STREAM_CHUNK_SIZE));
  }
  return chunks.length > 0 ? chunks : [""];
}

function clearRunTimeouts(run: WebActiveRun): void {
  for (const t of run.timeouts) clearTimeout(t);
  run.timeouts = [];
}

function scheduleWebStream(
  store: WebStore,
  run: WebActiveRun,
  replyText: string,
): void {
  const session = findSession(store, run.promptId, run.sessionId);
  const chunks = chunkText(replyText);
  let accumulated = "";
  let index = 0;

  const emitChunk = () => {
    if (run.aborted) return;
    if (index >= chunks.length) {
      session.transcript.push({ type: "message", role: "assistant", content: accumulated });
      session.updated_at = nowIso();
      saveStore(store);
      emitPromptRunEvent(PROMPT_RUN_EVENTS.chunk, {
        session_id: run.sessionId,
        run_id: run.runId,
        chunk: "",
        done: true,
      });
      emitPromptRunEvent(PROMPT_RUN_EVENTS.done, {
        session_id: run.sessionId,
        run_id: run.runId,
      });
      activeWebRuns.delete(run.runId);
      return;
    }
    accumulated += chunks[index];
    index += 1;
    emitPromptRunEvent(PROMPT_RUN_EVENTS.chunk, {
      session_id: run.sessionId,
      run_id: run.runId,
      chunk: chunks[index - 1],
      done: false,
    });
    const timeout = setTimeout(emitChunk, STREAM_CHUNK_DELAY_MS);
    run.timeouts.push(timeout);
  };

  const timeout = setTimeout(emitChunk, STREAM_CHUNK_DELAY_MS);
  run.timeouts.push(timeout);
}

function startWebStream(
  store: WebStore,
  promptId: string,
  sessionId: string,
  userMessage: string,
): StartPromptRunResult {
  const session = findSession(store, promptId, sessionId);
  const runId = newId();
  const run: WebActiveRun = {
    sessionId,
    runId,
    promptId,
    aborted: false,
    timeouts: [],
  };
  activeWebRuns.set(runId, run);

  if (userMessage) {
    session.transcript.push({ type: "message", role: "user", content: userMessage });
  }
  session.updated_at = nowIso();
  saveStore(store);

  scheduleWebStream(store, run, mockReplyForUserMessage(userMessage));
  return { session_id: sessionId, run_id: runId };
}

function webStartPromptRun(
  store: WebStore,
  args: Record<string, unknown>,
): StartPromptRunResult {
  const input = args.input as {
    prompt_id: string;
    user_message?: string;
    variables?: Record<string, string>;
    allow_empty_variables?: boolean;
  };
  const promptId = input.prompt_id;
  if (connectionWidget(store).status !== "connected") {
    throw new Error("LangDock is not connected");
  }
  findPrompt(store, promptId);
  const head = headContentForPrompt(store, promptId);
  const instructions = validateAndSubstitute(
    head,
    input.variables,
    input.allow_empty_variables ?? false,
  );
  const sessionId = newId();
  const createdAt = nowIso();
  const session: StoredSession = {
    id: sessionId,
    prompt_id: promptId,
    created_at: createdAt,
    updated_at: createdAt,
    stopped: false,
    transcript: [{ type: "run_config", instructions, model: "gpt-4o-mini" }],
  };
  store.sessions.push(session);
  saveStore(store);
  return startWebStream(store, promptId, sessionId, input.user_message ?? "");
}

function webSendPromptMessage(
  store: WebStore,
  args: Record<string, unknown>,
): StartPromptRunResult {
  const input = args.input as {
    prompt_id: string;
    session_id: string;
    user_message: string;
  };
  if (connectionWidget(store).status !== "connected") {
    throw new Error("LangDock is not connected");
  }
  findSession(store, input.prompt_id, input.session_id);
  return startWebStream(store, input.prompt_id, input.session_id, input.user_message);
}

function webStopPromptRun(store: WebStore, args: Record<string, unknown>): void {
  const input = args.input as { prompt_id: string; session_id: string; run_id: string };
  const run = activeWebRuns.get(input.run_id);
  if (!run || run.sessionId !== input.session_id || run.promptId !== input.prompt_id) {
    throw new Error("run not found");
  }
  run.aborted = true;
  clearRunTimeouts(run);
  const session = findSession(store, input.prompt_id, input.session_id);
  session.stopped = true;
  session.updated_at = nowIso();
  session.transcript.push({ type: "meta", event: "stopped" });
  saveStore(store);
  emitPromptRunEvent(PROMPT_RUN_EVENTS.stopped, {
    session_id: input.session_id,
    run_id: input.run_id,
  });
  activeWebRuns.delete(input.run_id);
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
      requireWebAuthorize("credential:write");
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
        webProfileApiKeys.set(id, input.api_key);
      }
      if (testConnectivity && webProfileApiKeys.has(id)) {
        profile.connection_status = await probeLangDock(
          profile.base_url,
          webProfileApiKeys.get(id)!,
        );
        profile.last_tested_at = nowIso();
      }
      saveStore(store);
      return { ...profile } as T;
    }

    case "test_langdock_connection": {
      requireWebAuthorize("credential:write");
      const profileId = args.profileId as string;
      const profile = store.profiles.find((p) => p.id === profileId);
      if (!profile) throw new Error("profile not found");
      const apiKey = webProfileApiKeys.get(profileId);
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
      requireWebAuthorize("credential:write");
      const profileId = args.profileId as string;
      const profile = store.profiles.find((p) => p.id === profileId);
      if (!profile) throw new Error("profile not found");
      for (const p of store.profiles) p.is_default = p.id === profileId;
      saveStore(store);
      return { ...profile } as T;
    }

    case "delete_langdock_profile": {
      requireWebAuthorize("credential:write");
      const profileId = args.profileId as string;
      store.profiles = store.profiles.filter((p) => p.id !== profileId);
      webProfileApiKeys.delete(profileId);
      saveStore(store);
      return undefined as T;
    }

    case "list_prompts":
      requireWebAuthorize("prompt:read");
      return listPromptsFiltered(store, {
        folderId: args.folderId as string | null | undefined,
        tagId: args.tagId as string | null | undefined,
        favoritesOnly: (args.favoritesOnly as boolean | null | undefined) ?? undefined,
        lifecycleFilter: (args.lifecycleFilter as ListPromptFilters["lifecycleFilter"]) ?? "active",
      }) as T;

    case "get_prompt": {
      requireWebAuthorize("prompt:read");
      const prompt = findAnyPrompt(store, args.promptId as string);
      return toPromptSummary(store, prompt) as T;
    }

    case "create_prompt": {
      requireWebAuthorize("prompt:write");
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
        lifecycle_status: "active",
        head_version_id: null,
        content_syntax: "auto",
      };
      store.prompts.push(prompt);
      appendPromptVersion(store, id, "");
      saveStore(store);
      return id as T;
    }

    case "update_prompt": {
      requireWebAuthorize("prompt:write");
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
      if (args.contentSyntax !== undefined && args.contentSyntax !== null) {
        const syntax = args.contentSyntax as PromptContentSyntax;
        const allowed: PromptContentSyntax[] = [
          "auto",
          "plaintext",
          "markdown",
          "xml",
          "json",
        ];
        if (!allowed.includes(syntax)) throw new Error("invalid content syntax");
        prompt.content_syntax = syntax;
      }
      prompt.updated_at = nowIso();
      saveStore(store);
      return toPromptSummary(store, prompt) as T;
    }

    case "trash_prompt": {
      requireWebAuthorize("prompt:write");
      const prompt = findAnyPrompt(store, args.promptId as string);
      const status = prompt.lifecycle_status ?? (prompt.trashed ? "trashed" : "active");
      if (status === "trashed") throw new Error("prompt not found");
      prompt.trashed = true;
      prompt.lifecycle_status = "trashed";
      prompt.updated_at = nowIso();
      saveStore(store);
      return undefined as T;
    }

    case "archive_prompt": {
      requireWebAuthorize("prompt:write");
      const prompt = findActivePrompt(store, args.promptId as string);
      prompt.lifecycle_status = "archived";
      prompt.trashed = false;
      prompt.updated_at = nowIso();
      saveStore(store);
      return undefined as T;
    }

    case "unarchive_prompt": {
      requireWebAuthorize("prompt:write");
      const prompt = findAnyPrompt(store, args.promptId as string);
      if (prompt.lifecycle_status !== "archived") throw new Error("prompt not found");
      prompt.lifecycle_status = "active";
      prompt.updated_at = nowIso();
      saveStore(store);
      return undefined as T;
    }

    case "restore_prompt": {
      requireWebAuthorize("prompt:write");
      const prompt = findAnyPrompt(store, args.promptId as string);
      if (prompt.lifecycle_status !== "trashed") throw new Error("prompt not found");
      prompt.lifecycle_status = "active";
      prompt.trashed = false;
      prompt.updated_at = nowIso();
      saveStore(store);
      return undefined as T;
    }

    case "list_prompt_folders":
      return [...store.folders].sort((a, b) => a.title.localeCompare(b.title)) as T;

    case "create_prompt_folder": {
      requireWebAuthorize("prompt:write");
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
      requireWebAuthorize("prompt:write");
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
      requireWebAuthorize("prompt:read");
      return [...store.tags].sort((a, b) => a.name.localeCompare(b.name)) as T;

    case "set_prompt_tags": {
      requireWebAuthorize("prompt:write");
      const prompt = findPrompt(store, args.promptId as string);
      const tagNames = args.tagNames as string[];
      prompt.tag_ids = tagNames.map((name) => findOrCreateTag(store, name).id);
      prompt.updated_at = nowIso();
      saveStore(store);
      return toPromptSummary(store, prompt) as T;
    }

    case "add_prompt_tag": {
      requireWebAuthorize("prompt:write");
      const prompt = findPrompt(store, args.promptId as string);
      const tag = findOrCreateTag(store, args.tagName as string);
      if (!prompt.tag_ids.includes(tag.id)) prompt.tag_ids.push(tag.id);
      prompt.updated_at = nowIso();
      saveStore(store);
      return toPromptSummary(store, prompt) as T;
    }

    case "remove_prompt_tag": {
      requireWebAuthorize("prompt:write");
      const prompt = findPrompt(store, args.promptId as string);
      const tagId = args.tagId as string;
      prompt.tag_ids = prompt.tag_ids.filter((id) => id !== tagId);
      prompt.updated_at = nowIso();
      saveStore(store);
      return toPromptSummary(store, prompt) as T;
    }

    case "set_prompt_favorite": {
      requireWebAuthorize("prompt:write");
      const prompt = findPrompt(store, args.promptId as string);
      prompt.is_favorite = true;
      prompt.updated_at = nowIso();
      saveStore(store);
      return undefined as T;
    }

    case "unset_prompt_favorite": {
      requireWebAuthorize("prompt:write");
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
      requireWebAuthorize("prompt:write");
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
        lifecycle_status: "active",
        head_version_id: null,
        content_syntax: "auto",
      });
      appendPromptVersion(store, id, "");
      saveStore(store);
      return undefined as T;
    }

    case "library_counts": {
      const counts: LibraryCounts = {
        prompts: activePrompts(store).length,
        agents: activeAgents(store).length,
        skills: 0,
      };
      return counts as T;
    }

    case "list_agents": {
      requireWebAuthorize("agent:read");
      const lifecycle = (args.lifecycleFilter as string | null) ?? "active";
      const list = store.agents
        .filter((a) => a.lifecycle_status === lifecycle)
        .map(toAgentSummary);
      return list as T;
    }

    case "get_agent": {
      requireWebAuthorize("agent:read");
      const agent = findAgent(store, args.agentId as string);
      return toAgentDetail(store, agent) as T;
    }

    case "create_agent": {
      requireWebAuthorize("agent:write");
      const title = (args.title as string).trim();
      if (!title) throw new Error("title is required");
      const id = newId();
      const agent: StoredAgent = {
        id,
        title,
        slug: slugFromId("agent", id),
        lifecycle_status: "active",
        instructions: "",
        model: "gpt-4o-mini",
        web_search: false,
        prompt_ids: [],
        skill_ids: [],
        subagent_ids: [],
        updated_at: nowIso(),
      };
      store.agents.push(agent);
      saveStore(store);
      return id as T;
    }

    case "update_agent": {
      requireWebAuthorize("agent:write");
      const agent = findAgent(store, args.agentId as string);
      if (args.title !== undefined) {
        const title = (args.title as string).trim();
        if (!title) throw new Error("title is required");
        agent.title = title;
      }
      if (args.instructions !== undefined) agent.instructions = args.instructions as string;
      if (args.model !== undefined) {
        const model = (args.model as string).trim();
        if (!model) throw new Error("model is required");
        agent.model = model;
      }
      if (args.webSearch !== undefined) agent.web_search = args.webSearch as boolean;
      agent.updated_at = nowIso();
      saveStore(store);
      return toAgentDetail(store, agent) as T;
    }

    case "set_agent_prompts": {
      requireWebAuthorize("agent:write");
      const agent = findAgent(store, args.agentId as string);
      const promptIds = args.promptIds as string[];
      for (const promptId of promptIds) {
        if (!activePrompts(store).some((p) => p.id === promptId)) {
          throw new Error("prompt not found");
        }
      }
      agent.prompt_ids = [...promptIds];
      agent.updated_at = nowIso();
      saveStore(store);
      return toAgentDetail(store, agent) as T;
    }

    case "set_agent_skills": {
      requireWebAuthorize("agent:write");
      const agent = findAgent(store, args.agentId as string);
      agent.skill_ids = [...(args.skillIds as string[])];
      agent.updated_at = nowIso();
      saveStore(store);
      return toAgentDetail(store, agent) as T;
    }

    case "set_agent_subagents": {
      requireWebAuthorize("agent:write");
      const agent = findAgent(store, args.agentId as string);
      const childIds = args.childAgentIds as string[];
      for (const childId of childIds) {
        assertSubagentLinkable(store, agent.id, childId);
      }
      agent.subagent_ids = [...childIds];
      agent.updated_at = nowIso();
      saveStore(store);
      return toAgentDetail(store, agent) as T;
    }

    case "check_authorize": {
      const action = args.action as string;
      return webAuthorize(action) as T;
    }

    case "save_prompt_content": {
      requireWebAuthorize("prompt:write");
      const promptId = args.promptId as string;
      const content = args.content as string;
      if (content.length > MAX_CONTENT_LEN) throw new Error("content too long");
      const prompt = findPrompt(store, promptId);
      if (prompt.head_version_id) {
        const head = store.promptVersions.find((v) => v.id === prompt.head_version_id);
        if (head && head.content === content) {
          saveStore(store);
          return versionSummary(store, head) as T;
        }
      }
      const summary = appendPromptVersion(store, promptId, content);
      saveStore(store);
      return summary as T;
    }

    case "list_prompt_versions":
      requireWebAuthorize("prompt:read");
      return listVersionsForPrompt(store, args.promptId as string) as T;

    case "get_prompt_version_content": {
      requireWebAuthorize("prompt:read");
      const versionId = args.versionId as string;
      const version = store.promptVersions.find((v) => v.id === versionId);
      if (!version) throw new Error("version not found");
      findPrompt(store, version.prompt_id);
      return version.content as T;
    }

    case "restore_prompt_version": {
      requireWebAuthorize("prompt:write");
      const promptId = args.promptId as string;
      const versionId = args.versionId as string;
      const prompt = findPrompt(store, promptId);
      const version = store.promptVersions.find(
        (v) => v.id === versionId && v.prompt_id === promptId,
      );
      if (!version) throw new Error("version not found");
      if (prompt.head_version_id === versionId) {
        throw new Error("already current version");
      }
      if (prompt.head_version_id) {
        const head = store.promptVersions.find((v) => v.id === prompt.head_version_id);
        if (head && head.content === version.content) {
          throw new Error("already current version");
        }
      }
      const summary = appendPromptVersion(store, promptId, version.content);
      saveStore(store);
      return summary as T;
    }

    case "start_prompt_run": {
      requireWebAuthorize("prompt:execute");
      return webStartPromptRun(store, args) as T;
    }

    case "send_prompt_message": {
      requireWebAuthorize("prompt:execute");
      return webSendPromptMessage(store, args) as T;
    }

    case "stop_prompt_run": {
      requireWebAuthorize("prompt:execute");
      webStopPromptRun(store, args);
      return undefined as T;
    }

    case "list_prompt_sessions": {
      requireWebAuthorize("prompt:read");
      const promptId = args.promptId as string;
      findPrompt(store, promptId);
      return store.sessions
        .filter((s) => s.prompt_id === promptId)
        .map(toSessionSummary)
        .sort((a, b) => b.updated_at.localeCompare(a.updated_at)) as T;
    }

    case "get_session_transcript": {
      requireWebAuthorize("prompt:read");
      const promptId = args.promptId as string;
      const sessionId = args.sessionId as string;
      const session = findSession(store, promptId, sessionId);
      const transcript = [...session.transcript];
      if (transcriptByteSize(transcript) > MAX_TRANSCRIPT_BYTES) {
        throw new Error("transcript too large");
      }
      return transcript as T;
    }

    case "delete_prompt_session": {
      requireWebAuthorize("prompt:execute");
      const promptId = args.promptId as string;
      const sessionId = args.sessionId as string;
      findSession(store, promptId, sessionId);
      store.sessions = store.sessions.filter((s) => s.id !== sessionId);
      saveStore(store);
      return undefined as T;
    }

    default:
      throw new Error(`Unknown web command: ${cmd}`);
  }
}
