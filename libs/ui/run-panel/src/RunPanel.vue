<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import type { RunPanelApi } from "./RunPanelApi";
import type {
  RunChunkPayload,
  RunErrorPayload,
  RunSessionPayload,
  TranscriptLine,
} from "./types";
import { RUN_EVENTS } from "./types";
import VariablesModal from "./VariablesModal.vue";

const props = defineProps<{
  entityId: string;
  headContent: string;
  runApi: RunPanelApi;
}>();

const emit = defineEmits<{
  error: [message: string];
  openSettings: [];
}>();

interface ChatBubble {
  role: "user" | "assistant";
  content: string;
  streaming?: boolean;
}

const sessions = ref<Awaited<ReturnType<RunPanelApi["listSessions"]>>>([]);
const selectedSessionId = ref<string>("");
const writableSessionId = ref<string | null>(null);
const bubbles = ref<ChatBubble[]>([]);
const composerText = ref("");
const canRun = ref(false);
const isStreaming = ref(false);
const activeRunId = ref<string | null>(null);
const runError = ref("");
const transcriptNotice = ref("");
const showVariablesModal = ref(false);
const showDeleteConfirm = ref(false);
const pendingUserMessage = ref("");
const pendingVariables = ref<Record<string, string> | null>(null);
const pendingAllowEmpty = ref(false);
const transcriptEndRef = ref<HTMLElement | null>(null);

const unlisteners: Array<() => void> = [];

const placeholders = computed(() => parseVariablePlaceholders(props.headContent));

const readOnly = computed(
  () =>
    !!selectedSessionId.value &&
    selectedSessionId.value !== writableSessionId.value,
);

const composerDisabled = computed(
  () => readOnly.value || isStreaming.value || !canRun.value,
);

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

function transcriptToBubbles(lines: TranscriptLine[]): ChatBubble[] {
  return lines
    .filter((line): line is Extract<TranscriptLine, { type: "message" }> => line.type === "message")
    .map((line) => ({
      role: line.role === "assistant" ? "assistant" : "user",
      content: line.content,
    }));
}

async function loadSessions() {
  try {
    sessions.value = await props.runApi.listSessions(props.entityId);
  } catch (e) {
    emit("error", e instanceof Error ? e.message : String(e));
  }
}

const TRANSCRIPT_TOO_LARGE_MESSAGE =
  "This session transcript is too large to display (over 4 MB). The session is still saved locally.";

async function loadTranscript(sessionId: string) {
  if (!sessionId) {
    bubbles.value = [];
    transcriptNotice.value = "";
    return;
  }
  transcriptNotice.value = "";
  try {
    const lines = await props.runApi.getSessionTranscript(props.entityId, sessionId);
    bubbles.value = transcriptToBubbles(lines);
    await scrollToEnd();
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    if (msg.toLowerCase().includes("transcript too large")) {
      transcriptNotice.value = TRANSCRIPT_TOO_LARGE_MESSAGE;
      bubbles.value = [];
    } else {
      emit("error", msg);
    }
  }
}

async function refreshCanRun() {
  canRun.value = await props.runApi.canRun();
}

async function scrollToEnd() {
  await nextTick();
  transcriptEndRef.value?.scrollIntoView({ behavior: "smooth", block: "end" });
}

function appendAssistantChunk(chunk: string) {
  const last = bubbles.value[bubbles.value.length - 1];
  if (last?.role === "assistant" && last.streaming) {
    last.content += chunk;
  } else {
    bubbles.value.push({ role: "assistant", content: chunk, streaming: true });
  }
}

function finalizeAssistantStream() {
  const last = bubbles.value[bubbles.value.length - 1];
  if (last?.role === "assistant" && last.streaming) {
    last.streaming = false;
  }
}

function onChunk(payload: RunChunkPayload) {
  if (payload.session_id !== writableSessionId.value) return;
  if (payload.run_id !== activeRunId.value) return;
  if (payload.chunk) appendAssistantChunk(payload.chunk);
  if (payload.done) finalizeAssistantStream();
  void scrollToEnd();
}

function onDone(payload: RunSessionPayload) {
  if (payload.session_id !== writableSessionId.value) return;
  if (payload.run_id !== activeRunId.value) return;
  isStreaming.value = false;
  activeRunId.value = null;
  finalizeAssistantStream();
  void loadSessions();
}

function onRunError(payload: RunErrorPayload) {
  if (payload.session_id !== writableSessionId.value) return;
  if (payload.run_id !== activeRunId.value) return;
  runError.value = payload.message;
  isStreaming.value = false;
  activeRunId.value = null;
  finalizeAssistantStream();
}

function onStopped(payload: RunSessionPayload) {
  if (payload.session_id !== writableSessionId.value) return;
  if (payload.run_id !== activeRunId.value) return;
  isStreaming.value = false;
  activeRunId.value = null;
  finalizeAssistantStream();
  void loadSessions();
  void loadTranscript(payload.session_id);
}

function setupEventListeners() {
  unlisteners.push(
    props.runApi.onRunEvent(RUN_EVENTS.chunk, (p) => onChunk(p as RunChunkPayload)),
  );
  unlisteners.push(
    props.runApi.onRunEvent(RUN_EVENTS.done, (p) => onDone(p as RunSessionPayload)),
  );
  unlisteners.push(
    props.runApi.onRunEvent(RUN_EVENTS.error, (p) => onRunError(p as RunErrorPayload)),
  );
  unlisteners.push(
    props.runApi.onRunEvent(RUN_EVENTS.stopped, (p) => onStopped(p as RunSessionPayload)),
  );
}

function resetWritableState() {
  writableSessionId.value = null;
  selectedSessionId.value = "";
  bubbles.value = [];
  composerText.value = "";
  runError.value = "";
}

async function onSessionSelectChange() {
  if (!selectedSessionId.value) {
    resetWritableState();
    return;
  }
  await loadTranscript(selectedSessionId.value);
}

function showDegradedMessage() {
  runError.value =
    "LangDock is not connected. Open LangDock setup to fix your connection.";
}

async function executeRun(
  userMessage: string,
  variables?: Record<string, string>,
  allowEmptyVariables?: boolean,
) {
  runError.value = "";
  isStreaming.value = true;

  try {
    if (!writableSessionId.value) {
      const result = await props.runApi.startRun({
        entityId: props.entityId,
        userMessage: userMessage || undefined,
        variables: variables ?? undefined,
        allowEmptyVariables,
      });
      writableSessionId.value = result.session_id;
      selectedSessionId.value = result.session_id;
      activeRunId.value = result.run_id;
      if (userMessage) {
        bubbles.value.push({ role: "user", content: userMessage });
      }
    } else {
      const result = await props.runApi.sendMessage({
        entityId: props.entityId,
        session_id: writableSessionId.value,
        user_message: userMessage,
      });
      activeRunId.value = result.run_id;
      bubbles.value.push({ role: "user", content: userMessage });
    }
    composerText.value = "";
    await scrollToEnd();
  } catch (e) {
    isStreaming.value = false;
    activeRunId.value = null;
    emit("error", e instanceof Error ? e.message : String(e));
  }
}

function attemptStartRun(userMessage: string) {
  if (!canRun.value) {
    showDegradedMessage();
    return;
  }
  if (!writableSessionId.value && placeholders.value.length > 0 && !pendingVariables.value) {
    pendingUserMessage.value = userMessage;
    showVariablesModal.value = true;
    return;
  }
  void executeRun(
    userMessage,
    pendingVariables.value ?? undefined,
    pendingAllowEmpty.value,
  );
  pendingVariables.value = null;
  pendingAllowEmpty.value = false;
}

function onVariablesConfirm(variables: Record<string, string>, allowEmpty: boolean) {
  showVariablesModal.value = false;
  pendingVariables.value = variables;
  pendingAllowEmpty.value = allowEmpty;
  void executeRun(pendingUserMessage.value, variables, allowEmpty);
  pendingVariables.value = null;
  pendingUserMessage.value = "";
}

function onVariablesCancel() {
  showVariablesModal.value = false;
  pendingUserMessage.value = "";
}

function onSend() {
  attemptStartRun(composerText.value.trim());
}

function onAttemptRun() {
  attemptStartRun(composerText.value.trim());
}

async function onStop() {
  if (!writableSessionId.value || !activeRunId.value) return;
  try {
    await props.runApi.stopRun({
      entityId: props.entityId,
      session_id: writableSessionId.value,
      run_id: activeRunId.value,
    });
  } catch (e) {
    emit("error", e instanceof Error ? e.message : String(e));
  }
}

function openDeleteConfirm() {
  if (!selectedSessionId.value) return;
  showDeleteConfirm.value = true;
}

async function confirmDeleteSession() {
  const sessionId = selectedSessionId.value;
  if (!sessionId) return;
  showDeleteConfirm.value = false;
  try {
    await props.runApi.deleteSession(props.entityId, sessionId);
    if (writableSessionId.value === sessionId) {
      resetWritableState();
    } else {
      selectedSessionId.value = "";
      bubbles.value = [];
    }
    await loadSessions();
  } catch (e) {
    emit("error", e instanceof Error ? e.message : String(e));
  }
}

function confirmLeaveIfActive(): Promise<boolean> {
  if (!isStreaming.value) return Promise.resolve(true);
  return Promise.resolve(window.confirm("A run is still streaming. Leave anyway?"));
}

onMounted(() => {
  setupEventListeners();
  void refreshCanRun();
  void loadSessions();
});

onUnmounted(() => {
  for (const unlisten of unlisteners) unlisten();
});

watch(
  () => props.entityId,
  () => {
    resetWritableState();
    void refreshCanRun();
    void loadSessions();
  },
);

defineExpose({ confirmLeaveIfActive, onAttemptRun, isStreaming });
</script>

<template>
  <section class="run-panel" data-testid="prompt-run-panel">
    <header class="run-panel-header">
      <h3>Run</h3>
      <div class="run-controls">
        <select
          v-model="selectedSessionId"
          class="session-select"
          data-testid="prompt-run-session-select"
          aria-label="Session history"
          @change="onSessionSelectChange"
        >
          <option value="">New session</option>
          <option v-for="session in sessions" :key="session.id" :value="session.id">
            {{ new Date(session.updated_at).toLocaleString() }}
            {{ session.stopped ? "(stopped)" : "" }}
          </option>
        </select>
        <button
          v-if="selectedSessionId"
          type="button"
          class="text-btn danger"
          data-testid="prompt-run-delete-session"
          :disabled="isStreaming"
          @click="openDeleteConfirm"
        >
          Delete
        </button>
      </div>
    </header>

    <p
      v-if="runError"
      class="run-error"
      data-testid="prompt-run-degraded"
    >
      {{ runError }}
      <button
        type="button"
        class="link-btn"
        data-testid="langdock-setup-link"
        @click="emit('openSettings')"
      >
        LangDock setup
      </button>
    </p>

    <p
      v-if="transcriptNotice"
      class="transcript-notice"
      data-testid="prompt-run-transcript-too-large"
    >
      {{ transcriptNotice }}
    </p>

    <div class="transcript" data-testid="prompt-run-transcript">
      <div
        v-for="(bubble, index) in bubbles"
        :key="index"
        class="bubble"
        :class="bubble.role"
        :data-testid="
          bubble.role === 'user' ? 'prompt-run-message-user' : 'prompt-run-message-assistant'
        "
      >
        <pre class="bubble-text">{{ bubble.content }}</pre>
      </div>
      <div ref="transcriptEndRef" />
    </div>

    <div v-if="!readOnly" class="composer" data-testid="prompt-run-composer">
      <textarea
        v-model="composerText"
        class="composer-input"
        rows="2"
        placeholder="Message…"
        :disabled="composerDisabled"
        data-testid="prompt-run-composer-input"
        @keydown.enter.exact.prevent="onSend"
      />
      <div class="composer-actions">
        <button
          v-if="!writableSessionId"
          type="button"
          data-testid="prompt-run-start"
          :disabled="isStreaming"
          @click="onAttemptRun"
        >
          Start run
        </button>
        <button
          v-if="isStreaming"
          type="button"
          class="stop-btn"
          data-testid="prompt-run-stop"
          @click="onStop"
        >
          Stop
        </button>
        <button
          type="button"
          data-testid="prompt-run-send"
          :disabled="composerDisabled"
          @click="onSend"
        >
          Send
        </button>
      </div>
    </div>
    <p v-else class="read-only-hint">Viewing a past session (read-only).</p>

    <VariablesModal
      v-if="showVariablesModal"
      :placeholders="placeholders"
      @confirm="onVariablesConfirm"
      @cancel="onVariablesCancel"
    />

    <div
      v-if="showDeleteConfirm"
      class="modal-backdrop"
      @click.self="showDeleteConfirm = false"
    >
      <div class="modal" role="dialog" aria-modal="true" @click.stop>
        <h3 class="modal-title">Delete session</h3>
        <p class="modal-body">Remove this session and its transcript?</p>
        <div class="modal-actions">
          <button type="button" @click="showDeleteConfirm = false">Cancel</button>
          <button type="button" data-testid="prompt-run-delete-confirm" @click="confirmDeleteSession">
            Delete
          </button>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.run-panel {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding: 0.75rem;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  background: var(--color-surface);
}

.run-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
}

.run-panel-header h3 {
  margin: 0;
  font-size: 1rem;
}

.run-controls {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.session-select {
  padding: 0.35rem 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-surface);
  color: var(--color-text-primary);
  font-size: 0.8125rem;
  max-width: 14rem;
}

.text-btn {
  background: transparent;
  border: none;
  color: var(--color-accent);
  cursor: pointer;
  font-size: 0.8125rem;
}

.text-btn.danger {
  color: var(--color-error, #dc3545);
}

.text-btn:disabled {
  opacity: 0.5;
  cursor: default;
}

.run-error {
  margin: 0;
  padding: 0.5rem 0.75rem;
  border-radius: 6px;
  background: var(--color-warning-muted);
  color: var(--color-warning);
  font-size: 0.8125rem;
}

.transcript-notice {
  margin: 0;
  padding: 0.5rem 0.75rem;
  border-radius: 6px;
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border);
  color: var(--color-text-muted);
  font-size: 0.8125rem;
}

.link-btn {
  margin-left: 0.5rem;
  background: none;
  border: none;
  color: var(--color-accent);
  cursor: pointer;
  text-decoration: underline;
  font-size: inherit;
}

.transcript {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  max-height: 20rem;
  overflow-y: auto;
  padding: 0.25rem;
}

.bubble {
  max-width: 85%;
  padding: 0.5rem 0.75rem;
  border-radius: 8px;
}

.bubble.user {
  align-self: flex-end;
  background: var(--color-accent-muted);
}

.bubble.assistant {
  align-self: flex-start;
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border);
}

.bubble-text {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
  font-family: inherit;
  font-size: 0.875rem;
}

.composer {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.composer-input {
  width: 100%;
  padding: 0.5rem 0.625rem;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-surface);
  color: var(--color-text-primary);
  font-family: inherit;
  font-size: 0.875rem;
  resize: vertical;
}

.composer-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
}

.stop-btn {
  border-color: var(--color-error, #dc3545);
  color: var(--color-error, #dc3545);
}

.read-only-hint {
  margin: 0;
  font-size: 0.8125rem;
  color: var(--color-text-muted);
}

.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 40;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.45);
}

.modal {
  width: min(24rem, calc(100vw - 2rem));
  padding: 1.25rem;
  border-radius: 8px;
  border: 1px solid var(--color-border);
  background: var(--color-surface-raised);
}

.modal-title {
  margin: 0 0 0.5rem;
}

.modal-body {
  margin: 0 0 1rem;
  color: var(--color-text-muted);
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
}
</style>
