<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { api, type PromptVersionSummary } from "../../lib/api";
import PromptRestoreConfirmModal from "./PromptRestoreConfirmModal.vue";
import PromptVersionDiffView from "./PromptVersionDiffView.vue";

const props = defineProps<{
  promptId: string;
}>();

const emit = defineEmits<{
  close: [];
  restored: [];
}>();

const versions = ref<PromptVersionSummary[]>([]);
const loading = ref(false);
const error = ref("");
const selectedVersionId = ref<string | null>(null);
const showRestoreConfirm = ref(false);
const headContent = ref("");
const selectedContent = ref("");

const headVersion = computed(() => versions.value.find((v) => v.is_head) ?? null);

const selectedVersion = computed(
  () => versions.value.find((v) => v.id === selectedVersionId.value) ?? null,
);

const canRestore = computed(() => {
  if (!selectedVersion.value || !headVersion.value) return false;
  if (selectedVersion.value.id === headVersion.value.id) return false;
  return headContent.value !== selectedContent.value;
});

const selectedLabel = computed(() => {
  const v = selectedVersion.value;
  if (!v) return "";
  return `v${v.version_number}`;
});

async function loadVersions() {
  loading.value = true;
  error.value = "";
  try {
    versions.value = await api.listPromptVersions(props.promptId);
    const head = versions.value.find((v) => v.is_head);
    if (head) {
      headContent.value = await api.getPromptVersionContent(head.id);
      if (!selectedVersionId.value || !versions.value.some((v) => v.id === selectedVersionId.value)) {
        const nonHead = versions.value.find((v) => !v.is_head);
        selectedVersionId.value = nonHead?.id ?? head.id;
      }
    } else {
      selectedVersionId.value = null;
      headContent.value = "";
    }
    await loadSelectedContent();
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
  }
}

async function loadSelectedContent() {
  if (!selectedVersionId.value) {
    selectedContent.value = "";
    return;
  }
  selectedContent.value = await api.getPromptVersionContent(selectedVersionId.value);
}

async function onSelectVersion(versionId: string) {
  selectedVersionId.value = versionId;
  try {
    await loadSelectedContent();
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}

function openRestoreConfirm() {
  if (!canRestore.value) return;
  showRestoreConfirm.value = true;
}

async function confirmRestore() {
  if (!selectedVersionId.value) return;
  showRestoreConfirm.value = false;
  error.value = "";
  try {
    await api.restorePromptVersion(props.promptId, selectedVersionId.value);
    emit("restored");
    await loadVersions();
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}

onMounted(() => {
  void loadVersions();
});

watch(
  () => props.promptId,
  () => {
    selectedVersionId.value = null;
    void loadVersions();
  },
);

defineExpose({ refresh: loadVersions });
</script>

<template>
  <section class="version-panel" data-testid="version-history-panel">
    <header class="version-panel-header">
      <h3>Version history</h3>
      <button type="button" class="text-btn" aria-label="Close version history" @click="emit('close')">
        Close
      </button>
    </header>

    <p v-if="loading" class="muted">Loading versions…</p>
    <p v-else-if="error" class="error">{{ error }}</p>

    <template v-else>
      <ul class="version-list" data-testid="version-list">
        <li v-for="v in versions" :key="v.id">
          <button
            type="button"
            class="version-row"
            :class="{ active: selectedVersionId === v.id, head: v.is_head }"
            :data-testid="v.is_head ? 'version-head' : 'version-row'"
            @click="onSelectVersion(v.id)"
          >
            <span class="version-num">v{{ v.version_number }}</span>
            <span class="version-date">{{ new Date(v.created_at).toLocaleString() }}</span>
            <span v-if="v.is_head" class="head-badge">Head</span>
          </button>
        </li>
      </ul>

      <PromptVersionDiffView
        v-if="headVersion && selectedVersion"
        left-label="Selected"
        right-label="Head"
        :left-content="selectedContent"
        :right-content="headContent"
      />

      <div class="version-actions">
        <button
          type="button"
          data-testid="restore-version"
          :disabled="!canRestore"
          :title="!canRestore ? 'Select a different version to restore' : undefined"
          @click="openRestoreConfirm"
        >
          Restore as new version
        </button>
      </div>
    </template>

    <PromptRestoreConfirmModal
      v-if="showRestoreConfirm && selectedVersion"
      :version-label="selectedLabel"
      @confirm="confirmRestore"
      @cancel="showRestoreConfirm = false"
    />
  </section>
</template>

<style scoped>
.version-panel {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding: 0.75rem;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  background: var(--color-surface);
}

.version-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
}

.version-panel-header h3 {
  margin: 0;
  font-size: 1rem;
}

.text-btn {
  background: transparent;
  border: none;
  color: var(--color-accent, #6ea8fe);
  cursor: pointer;
  font-size: 0.875rem;
  padding: 0.25rem 0.5rem;
}

.version-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  max-height: 10rem;
  overflow-y: auto;
}

.version-row {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.625rem;
  text-align: left;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  color: var(--color-text-primary);
  cursor: pointer;
}

.version-row:hover,
.version-row.active {
  border-color: var(--color-border);
  background: var(--color-surface-raised);
}

.version-num {
  font-weight: 600;
  min-width: 2rem;
}

.version-date {
  flex: 1;
  font-size: 0.8125rem;
  color: var(--color-text-muted);
}

.head-badge {
  font-size: 0.6875rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  padding: 0.125rem 0.375rem;
  border-radius: 4px;
  background: var(--color-accent-muted, rgba(110, 168, 254, 0.2));
  color: var(--color-accent, #6ea8fe);
}

.version-actions {
  display: flex;
  justify-content: flex-end;
}

.muted {
  color: var(--color-text-muted);
  margin: 0;
}

.error {
  color: var(--color-error, #dc3545);
  margin: 0;
}
</style>
