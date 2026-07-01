<script setup lang="ts">
import { ref, watch } from "vue";
import { api, type AgentDetail, type AgentSummary, type PromptSummary } from "../lib/api";
import AgentDetailPanel from "../components/agents/AgentDetailPanel.vue";

const props = defineProps<{
  startCreate?: boolean;
}>();

defineEmits<{
  openSettings: [];
}>();

const agents = ref<AgentSummary[]>([]);
const availablePrompts = ref<PromptSummary[]>([]);
const selectedAgentId = ref<string | null>(null);
const selectedDetail = ref<AgentDetail | null>(null);
const loading = ref(false);
const error = ref("");
const showCreate = ref(false);
const newTitle = ref("");

async function loadAgents() {
  loading.value = true;
  error.value = "";
  try {
    agents.value = await api.listAgents();
    availablePrompts.value = await api.listPrompts();
    if (selectedAgentId.value) {
      selectedDetail.value = await api.getAgent(selectedAgentId.value);
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
  }
}

async function selectAgent(id: string) {
  selectedAgentId.value = id;
  try {
    selectedDetail.value = await api.getAgent(id);
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}

async function createAgent() {
  const title = newTitle.value.trim();
  if (!title) return;
  try {
    const id = await api.createAgent(title);
    showCreate.value = false;
    newTitle.value = "";
    await loadAgents();
    await selectAgent(id);
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}

function onSaved(detail: AgentDetail) {
  selectedDetail.value = detail;
  void loadAgents();
}

watch(
  () => props.startCreate,
  (create) => {
    if (create) showCreate.value = true;
  },
  { immediate: true },
);

defineExpose({ refresh: loadAgents });

void loadAgents();
</script>

<template>
  <div class="agent-library" data-testid="agent-library-view">
    <aside class="list-pane">
      <header class="list-header">
        <h1>Agents</h1>
        <button type="button" data-testid="new-agent-button" @click="showCreate = true">New agent</button>
      </header>
      <p v-if="error" class="error">{{ error }}</p>
      <p v-if="loading" class="muted">Loading…</p>
      <ul v-else class="agent-list" data-testid="agent-list">
        <li
          v-for="agent in agents"
          :key="agent.id"
          :class="{ active: agent.id === selectedAgentId }"
        >
          <button type="button" @click="selectAgent(agent.id)">
            <span class="title">{{ agent.title }}</span>
            <span class="model">{{ agent.model }}</span>
          </button>
        </li>
        <li v-if="agents.length === 0" class="muted empty">No agents yet.</li>
      </ul>
    </aside>

    <main class="detail-pane">
      <AgentDetailPanel
        v-if="selectedDetail"
        :agent="selectedDetail"
        :all-agents="agents"
        :available-prompts="availablePrompts"
        @saved="onSaved"
        @error="(msg) => (error = msg)"
      />
      <div v-else class="empty-detail muted">Select an agent or create one.</div>
    </main>

    <div v-if="showCreate" class="modal-overlay" role="dialog" data-testid="create-agent-dialog">
      <div class="modal">
        <h2>New agent</h2>
        <input v-model="newTitle" type="text" placeholder="Agent title" data-testid="create-agent-title" />
        <div class="modal-actions">
          <button type="button" @click="showCreate = false">Cancel</button>
          <button type="button" class="primary" data-testid="create-agent-submit" @click="createAgent">
            Create
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.agent-library {
  display: flex;
  min-height: 100vh;
}

.list-pane {
  width: 280px;
  border-right: 1px solid var(--color-border);
  padding: 1rem;
  background: var(--color-surface-raised);
}

.list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1rem;
}

.list-header h1 {
  font-size: 1.125rem;
}

.agent-list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.agent-list button {
  width: 100%;
  text-align: left;
  border: none;
  background: transparent;
  padding: 0.5rem 0.75rem;
  border-radius: 6px;
  cursor: pointer;
  color: inherit;
}

.agent-list li.active button {
  background: var(--color-accent-muted);
}

.agent-list .title {
  display: block;
  font-weight: 600;
}

.agent-list .model {
  display: block;
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.detail-pane {
  flex: 1;
  min-width: 0;
}

.empty-detail {
  padding: 2rem;
}

.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  display: grid;
  place-items: center;
}

.modal {
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  padding: 1.25rem;
  min-width: 20rem;
}

.modal input {
  width: 100%;
  margin: 0.75rem 0 1rem;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
}

.primary {
  background: var(--color-accent);
  color: #fff;
  border: none;
  border-radius: 8px;
  padding: 0.4rem 0.75rem;
}

.error {
  color: var(--color-danger, #f87171);
}

.muted {
  color: var(--color-text-muted);
}
</style>
