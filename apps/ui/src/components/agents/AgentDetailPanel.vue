<script setup lang="ts">
import { computed, ref } from "vue";
import { api, type AgentDetail, type AgentSummary, type PromptSummary } from "../../lib/api";
import EditIcon from "../library/EditIcon.vue";
import AgentUnsavedChangesModal from "./AgentUnsavedChangesModal.vue";

const props = defineProps<{
  agent: AgentDetail;
  allAgents: AgentSummary[];
  availablePrompts: PromptSummary[];
}>();

const emit = defineEmits<{
  saved: [agent: AgentDetail];
  error: [message: string];
}>();

const isEditing = ref(false);
const saving = ref(false);
const showUnsavedModal = ref(false);

const editTitle = ref("");
const savedTitle = ref("");
const editInstructions = ref("");
const savedInstructions = ref("");
const editModel = ref("");
const savedModel = ref("");
const editWebSearch = ref(false);
const savedWebSearch = ref(false);
const editPromptIds = ref<string[]>([]);
const savedPromptIds = ref<string[]>([]);
const editSubagentIds = ref<string[]>([]);
const savedSubagentIds = ref<string[]>([]);

const isDirty = computed(() => {
  return (
    editTitle.value !== savedTitle.value ||
    editInstructions.value !== savedInstructions.value ||
    editModel.value !== savedModel.value ||
    editWebSearch.value !== savedWebSearch.value ||
    editPromptIds.value.join() !== savedPromptIds.value.join() ||
    editSubagentIds.value.join() !== savedSubagentIds.value.join()
  );
});

function syncFromAgent() {
  editTitle.value = props.agent.title;
  savedTitle.value = props.agent.title;
  editInstructions.value = props.agent.instructions;
  savedInstructions.value = props.agent.instructions;
  editModel.value = props.agent.model;
  savedModel.value = props.agent.model;
  editWebSearch.value = props.agent.web_search;
  savedWebSearch.value = props.agent.web_search;
  editPromptIds.value = props.agent.prompts.map((p: { prompt_id: string }) => p.prompt_id);
  savedPromptIds.value = [...editPromptIds.value];
  editSubagentIds.value = props.agent.subagents.map((s: { agent_id: string }) => s.agent_id);
  savedSubagentIds.value = [...editSubagentIds.value];
}

function toggleEdit() {
  if (!isEditing.value) {
    syncFromAgent();
    isEditing.value = true;
    return;
  }
  if (isDirty.value) {
    showUnsavedModal.value = true;
    return;
  }
  isEditing.value = false;
}

async function save() {
  if (!isDirty.value) return;
  saving.value = true;
  try {
    let detail = await api.updateAgent(props.agent.id, {
      title: editTitle.value,
      instructions: editInstructions.value,
      model: editModel.value,
      webSearch: editWebSearch.value,
    });
    detail = await api.setAgentPrompts(props.agent.id, editPromptIds.value);
    detail = await api.setAgentSubagents(props.agent.id, editSubagentIds.value);
    savedTitle.value = editTitle.value;
    savedInstructions.value = editInstructions.value;
    savedModel.value = editModel.value;
    savedWebSearch.value = editWebSearch.value;
    savedPromptIds.value = [...editPromptIds.value];
    savedSubagentIds.value = [...editSubagentIds.value];
    isEditing.value = false;
    showUnsavedModal.value = false;
    emit("saved", detail);
  } catch (e) {
    emit("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

function discard() {
  syncFromAgent();
  isEditing.value = false;
  showUnsavedModal.value = false;
}

function togglePrompt(promptId: string) {
  const idx = editPromptIds.value.indexOf(promptId);
  if (idx >= 0) {
    editPromptIds.value.splice(idx, 1);
  } else {
    editPromptIds.value.push(promptId);
  }
}

function toggleSubagent(agentId: string) {
  const idx = editSubagentIds.value.indexOf(agentId);
  if (idx >= 0) {
    editSubagentIds.value.splice(idx, 1);
  } else {
    editSubagentIds.value.push(agentId);
  }
}

const subagentCandidates = computed(() =>
  props.allAgents.filter((a) => a.id !== props.agent.id),
);

syncFromAgent();
</script>

<template>
  <section class="agent-detail" data-testid="agent-detail-panel">
    <header class="detail-header">
      <div class="title-row">
        <h1 v-if="!isEditing">{{ agent.title }}</h1>
        <input
          v-else
          v-model="editTitle"
          class="title-input"
          data-testid="agent-title-input"
          type="text"
        />
        <button type="button" class="icon-btn" :aria-label="isEditing ? 'Cancel edit' : 'Edit'" @click="toggleEdit">
          <EditIcon />
        </button>
      </div>
      <p class="meta">{{ agent.slug }} · {{ agent.model }}</p>
    </header>

    <div class="sections">
      <section>
        <h2>Instructions</h2>
        <textarea
          v-if="isEditing"
          v-model="editInstructions"
          class="instructions-input"
          data-testid="agent-instructions-input"
          rows="10"
        />
        <pre v-else class="instructions-read">{{ agent.instructions || "No instructions yet." }}</pre>
      </section>

      <section v-if="isEditing" class="model-row">
        <label>
          Model
          <input v-model="editModel" type="text" data-testid="agent-model-input" />
        </label>
        <label class="checkbox">
          <input v-model="editWebSearch" type="checkbox" />
          Web search
        </label>
      </section>

      <section>
        <h2>Prompts</h2>
        <ul v-if="isEditing" class="picker-list">
          <li v-for="prompt in availablePrompts" :key="prompt.id">
            <label>
              <input
                type="checkbox"
                :checked="editPromptIds.includes(prompt.id)"
                @change="togglePrompt(prompt.id)"
              />
              {{ prompt.title }}
            </label>
          </li>
        </ul>
        <ul v-else class="attachment-list">
          <li v-for="prompt in agent.prompts" :key="prompt.prompt_id">{{ prompt.title }}</li>
          <li v-if="agent.prompts.length === 0" class="muted">No prompts attached.</li>
        </ul>
      </section>

      <section>
        <h2>Sub-agents</h2>
        <ul v-if="isEditing" class="picker-list">
          <li v-for="candidate in subagentCandidates" :key="candidate.id">
            <label>
              <input
                type="checkbox"
                :checked="editSubagentIds.includes(candidate.id)"
                @change="toggleSubagent(candidate.id)"
              />
              {{ candidate.title }}
            </label>
          </li>
        </ul>
        <ul v-else class="attachment-list">
          <li v-for="sub in agent.subagents" :key="sub.agent_id">{{ sub.title }}</li>
          <li v-if="agent.subagents.length === 0" class="muted">No sub-agents linked.</li>
        </ul>
      </section>
    </div>

    <footer v-if="isEditing" class="detail-footer">
      <button
        type="button"
        class="primary"
        data-testid="agent-update-button"
        :disabled="!isDirty || saving"
        @click="save"
      >
        Update agent
      </button>
    </footer>

    <AgentUnsavedChangesModal
      v-if="showUnsavedModal"
      @save="save"
      @discard="discard"
      @cancel="showUnsavedModal = false"
    />
  </section>
</template>

<style scoped>
.agent-detail {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1.5rem 2rem;
  height: 100%;
  overflow: auto;
}

.detail-header .title-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.title-input {
  flex: 1;
  font-size: 1.5rem;
  font-weight: 700;
}

.meta {
  color: var(--color-text-muted);
  font-size: 0.875rem;
}

.sections section {
  margin-bottom: 1.25rem;
}

.sections h2 {
  font-size: 0.875rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--color-text-muted);
  margin-bottom: 0.5rem;
}

.instructions-input,
.instructions-read {
  width: 100%;
  font-family: inherit;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 0.75rem;
  background: var(--color-surface);
  white-space: pre-wrap;
}

.model-row {
  display: flex;
  gap: 1.5rem;
  align-items: end;
}

.model-row input[type="text"] {
  display: block;
  margin-top: 0.25rem;
  min-width: 16rem;
}

.picker-list,
.attachment-list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.picker-list li,
.attachment-list li {
  padding: 0.25rem 0;
}

.muted {
  color: var(--color-text-muted);
}

.detail-footer {
  margin-top: auto;
}

.primary {
  background: var(--color-accent);
  color: #fff;
  border: none;
  border-radius: 8px;
  padding: 0.5rem 1rem;
  cursor: pointer;
}

.primary:disabled {
  opacity: 0.5;
  cursor: default;
}

.icon-btn {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--color-text-muted);
}
</style>
