<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import {
  api,
  type PromptContentSyntax,
  type PromptSummary,
  type TagSummary,
} from "../../lib/api";
import { can } from "../../lib/authz";
import { PROMPT_SYNTAX_OPTIONS } from "../../lib/prompt-syntax";
import EditIcon from "../library/EditIcon.vue";
import MoreMenuIcon from "../library/MoreMenuIcon.vue";
import MoveIcon from "../library/MoveIcon.vue";
import PlusIcon from "../library/PlusIcon.vue";
import StarFilterIcon from "../library/StarFilterIcon.vue";
import TrashIcon from "../library/TrashIcon.vue";
import { RunPanel } from "@velumia/ui-run-panel";
import { promptRunApi } from "../../lib/prompt-run-adapter";
import PromptCodeEditor from "./PromptCodeEditor.vue";
import PromptUnsavedChangesModal from "./PromptUnsavedChangesModal.vue";
import PromptVersionHistoryPanel from "./PromptVersionHistoryPanel.vue";

const props = defineProps<{
  prompt: PromptSummary;
  tags: TagSummary[];
  locationBreadcrumb: string[];
}>();

const emit = defineEmits<{
  refresh: [];
  error: [message: string];
  openMove: [];
  openDelete: [];
  favoriteToggle: [];
  openSettings: [];
}>();

const isEditing = ref(false);
const saving = ref(false);
const showVersionHistory = ref(false);
const showRunPanel = ref(false);
const showDetailMenu = ref(false);
const showTagPicker = ref(false);
const showUnsavedModal = ref(false);
const tagSearchQuery = ref("");
const detailMenuAnchorRef = ref<HTMLElement | null>(null);
const tagPickerAnchorRef = ref<HTMLElement | null>(null);
const tagSearchInputRef = ref<HTMLInputElement | null>(null);
const versionHistoryPanelRef = ref<InstanceType<typeof PromptVersionHistoryPanel> | null>(
  null,
);
const runPanelRef = ref<InstanceType<typeof RunPanel> | null>(null);

const canWrite = ref(true);
const canExecute = ref(true);

const lifecycleStatus = computed(
  () => props.prompt.lifecycle_status ?? "active",
);
const isArchivedOrTrashed = computed(
  () => lifecycleStatus.value === "archived" || lifecycleStatus.value === "trashed",
);
const detailReadOnly = computed(() => isArchivedOrTrashed.value || !canWrite.value);
const runAllowed = computed(
  () => lifecycleStatus.value === "active" && canExecute.value,
);

const editTitle = ref("");
const savedTitle = ref("");
const editTagNames = ref<string[]>([]);
const savedTagNames = ref<string[]>([]);
const promptContent = ref("");
const contentSaved = ref("");
const editSyntax = ref<PromptContentSyntax>("auto");
const savedSyntax = ref<PromptContentSyntax>("auto");

let pendingExitResolve: ((proceed: boolean) => void) | null = null;
let pendingExitAction: "leaveEdit" | "allowNavigation" | null = null;

const editTagNameSet = computed(() => new Set(editTagNames.value.map((n) => n.toLowerCase())));

const filteredPickerTags = computed(() => {
  const query = tagSearchQuery.value.trim().toLowerCase();
  return props.tags
    .filter((t) => !editTagNameSet.value.has(t.name.toLowerCase()))
    .filter((t) => !query || t.name.toLowerCase().includes(query))
    .sort((a, b) => a.name.localeCompare(b.name));
});

const showCreateTagOption = computed(() => {
  const name = tagSearchQuery.value.trim();
  if (!name) return false;
  return !editTagNameSet.value.has(name.toLowerCase());
});

const createTagLabel = computed(() => `Create tag “${tagSearchQuery.value.trim()}”`);

const titleDirty = computed(() => editTitle.value !== savedTitle.value);
const contentDirty = computed(() => promptContent.value !== contentSaved.value);
const syntaxDirty = computed(() => editSyntax.value !== savedSyntax.value);
const tagsDirty = computed(() => {
  const a = [...savedTagNames.value].sort();
  const b = [...editTagNames.value].sort();
  return a.length !== b.length || a.some((v, i) => v !== b[i]);
});
const isDirty = computed(
  () => titleDirty.value || contentDirty.value || syntaxDirty.value || tagsDirty.value,
);

const displayTags = computed(() => {
  if (isEditing.value) {
    return editTagNames.value.map((name) => {
      const existing = props.tags.find((t) => t.name.toLowerCase() === name.toLowerCase());
      return { id: existing?.id ?? name, name };
    });
  }
  return props.prompt.tags;
});

function syncSnapshotsFromPrompt() {
  editTitle.value = props.prompt.title;
  savedTitle.value = props.prompt.title;
  const names = props.prompt.tags.map((t) => t.name);
  editTagNames.value = [...names];
  savedTagNames.value = [...names];
  editSyntax.value = props.prompt.content_syntax;
  savedSyntax.value = props.prompt.content_syntax;
}

async function loadPromptContent() {
  try {
    const versions = await api.listPromptVersions(props.prompt.id);
    const head = versions.find((v) => v.is_head);
    if (head) {
      const body = await api.getPromptVersionContent(head.id);
      promptContent.value = body;
      contentSaved.value = body;
    } else {
      promptContent.value = "";
      contentSaved.value = "";
    }
  } catch (e) {
    emit("error", e instanceof Error ? e.message : String(e));
  }
}

function enterEditMode() {
  if (detailReadOnly.value) return;
  closeDetailMenu();
  closeVersionHistory();
  closeRunPanel();
  syncSnapshotsFromPrompt();
  isEditing.value = true;
}

function exitEditMode() {
  isEditing.value = false;
  closeTagPicker();
  closeDetailMenu();
}

function revertSnapshots() {
  syncSnapshotsFromPrompt();
  void loadPromptContent();
}

async function updatePrompt(options: { saveDraft?: boolean } = {}) {
  if (!isDirty.value) return;
  saving.value = true;
  try {
    if (titleDirty.value || syntaxDirty.value) {
      await api.updatePrompt(props.prompt.id, {
        title: titleDirty.value ? editTitle.value.trim() : undefined,
        contentSyntax: syntaxDirty.value ? editSyntax.value : undefined,
      });
    }
    if (tagsDirty.value) {
      await api.setPromptTags(props.prompt.id, editTagNames.value);
    }
    if (contentDirty.value) {
      await api.savePromptContent(props.prompt.id, promptContent.value);
      await versionHistoryPanelRef.value?.refresh();
    }
    savedTitle.value = editTitle.value;
    savedTagNames.value = [...editTagNames.value];
    savedSyntax.value = editSyntax.value;
    contentSaved.value = promptContent.value;
    emit("refresh");
    if (!options.saveDraft) exitEditMode();
  } catch (e) {
    emit("error", e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

function requestLeaveEdit(): Promise<boolean> {
  if (!isEditing.value || !isDirty.value) return Promise.resolve(true);
  pendingExitAction = "leaveEdit";
  showUnsavedModal.value = true;
  return new Promise((resolve) => {
    pendingExitResolve = resolve;
  });
}

function confirmLeaveIfDirty(): Promise<boolean> {
  if (showRunPanel.value && runPanelRef.value?.isStreaming) {
    return runPanelRef.value.confirmLeaveIfActive();
  }
  if (!isEditing.value || !isDirty.value) return Promise.resolve(true);
  pendingExitAction = "allowNavigation";
  showUnsavedModal.value = true;
  return new Promise((resolve) => {
    pendingExitResolve = resolve;
  });
}

function finishPendingExit(proceed: boolean) {
  showUnsavedModal.value = false;
  pendingExitResolve?.(proceed);
  pendingExitResolve = null;
  if (proceed && pendingExitAction === "leaveEdit") exitEditMode();
  pendingExitAction = null;
}

async function onUnsavedSave() {
  await updatePrompt();
  finishPendingExit(true);
}

async function onUnsavedSaveDraft() {
  await updatePrompt({ saveDraft: true });
  finishPendingExit(true);
}

function onUnsavedDiscard() {
  revertSnapshots();
  finishPendingExit(true);
}

async function toggleEditMode() {
  if (isEditing.value) {
    const ok = await requestLeaveEdit();
    if (ok && !isDirty.value) exitEditMode();
    return;
  }
  enterEditMode();
}

function closeDetailMenu() {
  showDetailMenu.value = false;
}

function toggleDetailMenu() {
  if (showDetailMenu.value) closeDetailMenu();
  else {
    closeTagPicker();
    showDetailMenu.value = true;
  }
}

function closeTagPicker() {
  showTagPicker.value = false;
  tagSearchQuery.value = "";
}

function openTagPicker() {
  tagSearchQuery.value = "";
  showTagPicker.value = true;
  void nextTick(() => tagSearchInputRef.value?.focus());
}

function toggleTagPicker() {
  if (showTagPicker.value) closeTagPicker();
  else {
    closeDetailMenu();
    openTagPicker();
  }
}

function addTagName(name: string) {
  const trimmed = name.trim();
  if (!trimmed) return;
  if (editTagNameSet.value.has(trimmed.toLowerCase())) return;
  editTagNames.value = [...editTagNames.value, trimmed];
  closeTagPicker();
}

function removeStagedTag(name: string) {
  editTagNames.value = editTagNames.value.filter(
    (t) => t.toLowerCase() !== name.toLowerCase(),
  );
}

async function onTagSearchEnter() {
  if (showCreateTagOption.value) {
    addTagName(tagSearchQuery.value);
    return;
  }
  const first = filteredPickerTags.value[0];
  if (first) addTagName(first.name);
}

function toggleVersionHistory() {
  closeDetailMenu();
  closeRunPanel();
  showVersionHistory.value = !showVersionHistory.value;
}

function closeVersionHistory() {
  showVersionHistory.value = false;
}

async function refreshAuthz() {
  canWrite.value = await can("prompt:write");
  canExecute.value = await can("prompt:execute");
}

async function onArchive() {
  closeDetailMenu();
  try {
    await api.archivePrompt(props.prompt.id);
    emit("refresh");
  } catch (e) {
    emit("error", e instanceof Error ? e.message : String(e));
  }
}

async function onUnarchive() {
  closeDetailMenu();
  try {
    await api.unarchivePrompt(props.prompt.id);
    emit("refresh");
  } catch (e) {
    emit("error", e instanceof Error ? e.message : String(e));
  }
}

async function onRestoreFromTrash() {
  closeDetailMenu();
  try {
    await api.restorePrompt(props.prompt.id);
    emit("refresh");
  } catch (e) {
    emit("error", e instanceof Error ? e.message : String(e));
  }
}

async function toggleRunPanel() {
  if (!runAllowed.value && !showRunPanel.value) return;
  closeDetailMenu();
  if (showRunPanel.value) {
    const ok = await runPanelRef.value?.confirmLeaveIfActive();
    if (!ok) return;
    showRunPanel.value = false;
    return;
  }
  if (isEditing.value) {
    const ok = await requestLeaveEdit();
    if (!ok) return;
    if (isDirty.value) return;
    exitEditMode();
  }
  closeVersionHistory();
  showRunPanel.value = true;
}

function closeRunPanel() {
  showRunPanel.value = false;
}

async function onVersionRestored() {
  await loadPromptContent();
  emit("refresh");
}

async function attemptPromptRun() {
  if (!showRunPanel.value) {
    if (isEditing.value) {
      const ok = await requestLeaveEdit();
      if (!ok) return;
      if (isDirty.value) return;
      exitEditMode();
    }
    closeVersionHistory();
    showRunPanel.value = true;
    await nextTick();
  }
  runPanelRef.value?.onAttemptRun();
}

function onDocumentPointerDown(e: PointerEvent) {
  const target = e.target as Node;
  if (showTagPicker.value) {
    const anchor = tagPickerAnchorRef.value;
    if (anchor && !anchor.contains(target)) closeTagPicker();
  }
  if (showDetailMenu.value) {
    const anchor = detailMenuAnchorRef.value;
    if (anchor && !anchor.contains(target)) closeDetailMenu();
  }
}

function handleEscape(): boolean {
  if (showUnsavedModal.value) {
    showUnsavedModal.value = false;
    pendingExitResolve?.(false);
    pendingExitResolve = null;
    pendingExitAction = null;
    return true;
  }
  if (showDetailMenu.value) {
    closeDetailMenu();
    return true;
  }
  if (showTagPicker.value) {
    closeTagPicker();
    return true;
  }
  if (showVersionHistory.value) {
    closeVersionHistory();
    return true;
  }
  if (showRunPanel.value) {
    void toggleRunPanel();
    return true;
  }
  if (isEditing.value && isDirty.value) {
    void requestLeaveEdit().then((ok) => {
      if (ok) exitEditMode();
    });
    return true;
  }
  if (isEditing.value) {
    exitEditMode();
    return true;
  }
  return false;
}

watch(
  () => props.prompt.id,
  () => {
    isEditing.value = false;
    showUnsavedModal.value = false;
    closeVersionHistory();
    closeRunPanel();
    closeTagPicker();
    closeDetailMenu();
    syncSnapshotsFromPrompt();
    void loadPromptContent();
    void refreshAuthz();
  },
  { immediate: true },
);

onMounted(() => {
  document.addEventListener("pointerdown", onDocumentPointerDown);
  void refreshAuthz();
});

onUnmounted(() => {
  document.removeEventListener("pointerdown", onDocumentPointerDown);
});

defineExpose({ confirmLeaveIfDirty, handleEscape, attemptPromptRun });
</script>

<template>
  <div class="detail" data-testid="prompt-detail">
    <div class="detail-header">
      <div class="detail-meta">
        <div class="title-row">
          <h2 v-if="!isEditing">{{ prompt.title }}</h2>
          <input
            v-else
            v-model="editTitle"
            type="text"
            class="title-input"
            data-testid="prompt-title-input"
            aria-label="Prompt title"
          />
          <button
            v-if="!detailReadOnly"
            type="button"
            class="icon-btn edit-toggle"
            data-testid="prompt-edit-toggle"
            :class="{ active: isEditing }"
            :aria-label="isEditing ? 'Exit edit mode' : 'Edit prompt'"
            :aria-pressed="isEditing"
            @click="toggleEditMode"
          >
            <EditIcon />
          </button>
        </div>
        <p v-if="isArchivedOrTrashed" class="lifecycle-hint" data-testid="prompt-lifecycle-readonly">
          {{
            lifecycleStatus === "archived"
              ? "Archived — read-only"
              : "In trash — read-only"
          }}
        </p>
        <p class="location" data-testid="prompt-location">
          {{ locationBreadcrumb.join(" / ") }}
        </p>
        <div class="tag-badges">
          <span
            v-for="tag in displayTags"
            :key="tag.id"
            class="status-badge"
            data-testid="prompt-tag"
          >
            {{ tag.name }}
            <button
              v-if="isEditing"
              type="button"
              class="status-badge-remove"
              :aria-label="`Remove tag ${tag.name}`"
              @click="removeStagedTag(tag.name)"
            >
              ×
            </button>
          </span>
          <span v-if="isEditing" ref="tagPickerAnchorRef" class="tag-picker-anchor">
            <button
              type="button"
              class="status-badge status-badge-add"
              :class="{ active: showTagPicker }"
              aria-label="Add tag"
              title="Add tag"
              data-testid="add-tag"
              @click.stop="toggleTagPicker"
            >
              <PlusIcon />
            </button>
            <div v-if="showTagPicker" class="tag-picker" @click.stop>
              <input
                ref="tagSearchInputRef"
                v-model="tagSearchQuery"
                type="search"
                class="tag-picker-search"
                placeholder="Search tags…"
                data-testid="tag-search"
                @keydown.enter.prevent="onTagSearchEnter"
              />
              <ul v-if="filteredPickerTags.length" class="tag-picker-list">
                <li v-for="tag in filteredPickerTags" :key="tag.id">
                  <button type="button" @click="addTagName(tag.name)">{{ tag.name }}</button>
                </li>
              </ul>
              <p v-else-if="!showCreateTagOption" class="tag-picker-empty">No matching tags</p>
              <button
                v-if="showCreateTagOption"
                type="button"
                class="tag-picker-create"
                data-testid="create-tag"
                @click="addTagName(tagSearchQuery)"
              >
                {{ createTagLabel }}
              </button>
            </div>
          </span>
        </div>
      </div>
      <div class="detail-toolbar">
        <button
          type="button"
          class="text-toolbar-btn"
          data-testid="prompt-run-toggle"
          :class="{ active: showRunPanel }"
          :disabled="!runAllowed"
          :title="runAllowed ? 'Run prompt' : 'Run not available'"
          @click="toggleRunPanel"
        >
          Run
        </button>
        <button
          type="button"
          class="text-toolbar-btn"
          data-testid="version-history-toggle"
          :class="{ active: showVersionHistory }"
          @click="toggleVersionHistory"
        >
          Version history
        </button>
        <button
          type="button"
          class="icon-btn"
          :class="{ active: prompt.is_favorite }"
          data-testid="favorite-toggle"
          :aria-label="prompt.is_favorite ? 'Unfavorite' : 'Favorite'"
          :aria-pressed="prompt.is_favorite"
          :title="prompt.is_favorite ? 'Unfavorite' : 'Favorite'"
          @click="emit('favoriteToggle')"
        >
          <StarFilterIcon :active="prompt.is_favorite" />
        </button>
        <span ref="detailMenuAnchorRef" class="detail-menu-anchor">
          <button
            type="button"
            class="icon-btn"
            aria-label="Prompt actions"
            title="Prompt actions"
            aria-haspopup="menu"
            :aria-expanded="showDetailMenu"
            @click.stop="toggleDetailMenu"
          >
            <MoreMenuIcon />
          </button>
          <div
            v-if="showDetailMenu"
            class="detail-menu"
            role="menu"
            aria-label="Prompt actions"
            @click.stop
          >
        <button
          v-if="lifecycleStatus === 'active' && canWrite"
          type="button"
          class="detail-menu-item"
          role="menuitem"
          data-testid="archive-prompt"
          @click="onArchive"
        >
              <span>Archive</span>
            </button>
        <button
          v-if="lifecycleStatus === 'archived' && canWrite"
          type="button"
          class="detail-menu-item"
          role="menuitem"
          data-testid="unarchive-prompt"
          @click="onUnarchive"
        >
              <span>Unarchive</span>
            </button>
        <button
          v-if="lifecycleStatus === 'trashed' && canWrite"
          type="button"
          class="detail-menu-item"
          role="menuitem"
          data-testid="restore-prompt"
          @click="onRestoreFromTrash"
        >
              <span>Restore from trash</span>
            </button>
        <button
          v-if="canWrite && lifecycleStatus !== 'trashed'"
          type="button"
          class="detail-menu-item"
          role="menuitem"
          data-testid="move-prompt"
          @click="closeDetailMenu(); emit('openMove')"
        >
              <MoveIcon />
              <span>Move to folder</span>
            </button>
        <button
          v-if="canWrite && lifecycleStatus !== 'trashed'"
          type="button"
          class="detail-menu-item detail-menu-item-danger"
          role="menuitem"
          @click="closeDetailMenu(); emit('openDelete')"
        >
              <TrashIcon />
              <span>Move to trash</span>
            </button>
          </div>
        </span>
      </div>
    </div>

    <div class="prompt-content-section">
      <div class="content-header">
        <span class="field-label">Content</span>
        <select
          v-if="isEditing"
          v-model="editSyntax"
          class="syntax-select"
          data-testid="prompt-syntax-select"
          aria-label="Content syntax"
        >
          <option v-for="opt in PROMPT_SYNTAX_OPTIONS" :key="opt.value" :value="opt.value">
            {{ opt.label }}
          </option>
        </select>
      </div>
      <PromptCodeEditor
        v-model="promptContent"
        :stored-syntax="isEditing ? editSyntax : savedSyntax"
        :read-only="!isEditing"
        test-id="prompt-content"
      />
      <div v-if="isEditing" class="content-actions">
        <button
          type="button"
          data-testid="update-prompt"
          :disabled="!isDirty || saving"
          @click="updatePrompt()"
        >
          {{ saving ? "Updating…" : "Update prompt" }}
        </button>
      </div>
    </div>

    <RunPanel
      v-if="showRunPanel"
      ref="runPanelRef"
      :entity-id="prompt.id"
      :head-content="contentSaved"
      :run-api="promptRunApi"
      @error="emit('error', $event)"
      @open-settings="emit('openSettings')"
    />

    <PromptVersionHistoryPanel
      v-if="showVersionHistory"
      ref="versionHistoryPanelRef"
      :prompt-id="prompt.id"
      @close="closeVersionHistory"
      @restored="onVersionRestored"
    />

    <PromptUnsavedChangesModal
      v-if="showUnsavedModal"
      @save="onUnsavedSave"
      @save-draft="onUnsavedSaveDraft"
      @discard="onUnsavedDiscard"
    />
  </div>
</template>

<style scoped>
.detail {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.detail-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.detail-meta {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  min-width: 0;
}

.title-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-width: 0;
}

.title-row h2 {
  margin: 0;
  color: var(--color-text-primary);
}

.title-input {
  flex: 1;
  min-width: 0;
  padding: 0.35rem 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-surface);
  color: var(--color-text-primary);
  font-size: 1.25rem;
  font-weight: 600;
}

.edit-toggle.active {
  color: var(--color-accent);
}

.lifecycle-hint {
  margin: 0.25rem 0 0;
  font-size: 0.8125rem;
  color: var(--color-text-muted);
}

.location {
  margin: 0;
  color: var(--color-text-muted);
  font-size: 0.8125rem;
  line-height: 1.35;
}

.detail-toolbar {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  flex-shrink: 0;
}

.text-toolbar-btn {
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: transparent;
  color: var(--color-text-muted);
  font-size: 0.75rem;
  padding: 0.35rem 0.625rem;
  cursor: pointer;
  white-space: nowrap;
}

.text-toolbar-btn:hover,
.text-toolbar-btn.active {
  border-color: var(--color-accent);
  color: var(--color-accent);
  background: var(--color-accent-muted);
}

.detail-toolbar .icon-btn {
  padding: 0;
}

.detail-menu-anchor {
  position: relative;
}

.detail-menu {
  position: absolute;
  top: calc(100% + 0.35rem);
  right: 0;
  z-index: 20;
  min-width: 11.5rem;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  background: var(--color-surface-raised);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
  padding: 0.35rem;
}

.detail-menu-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.5rem 0.625rem;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--color-text-primary);
  font-size: 0.8125rem;
  text-align: left;
  cursor: pointer;
}

.detail-menu-item:hover {
  background: var(--color-accent-muted);
  color: var(--color-accent);
}

.detail-menu-item-danger:hover {
  background: rgba(220, 53, 69, 0.15);
  color: var(--color-error, #dc3545);
}

.tag-badges {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
  align-items: center;
}

.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.2rem 0.5rem;
  border-radius: 999px;
  font-size: 0.75rem;
  background: var(--color-accent-muted);
  color: var(--color-accent);
}

.status-badge-remove {
  border: none;
  background: transparent;
  color: inherit;
  cursor: pointer;
  padding: 0;
  line-height: 1;
}

.tag-picker-anchor {
  position: relative;
}

.status-badge-add {
  padding: 0.2rem 0.35rem;
  cursor: pointer;
  border: 1px dashed var(--color-border);
  background: transparent;
}

.status-badge-add.active {
  border-color: var(--color-accent);
}

.tag-picker {
  position: absolute;
  top: calc(100% + 0.35rem);
  left: 0;
  z-index: 20;
  min-width: 12rem;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  background: var(--color-surface-raised);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
  padding: 0.5rem;
}

.tag-picker-search {
  width: 100%;
  padding: 0.4rem 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-surface);
  color: var(--color-text-primary);
  font-size: 0.8125rem;
}

.tag-picker-list {
  list-style: none;
  margin: 0.35rem 0 0;
  padding: 0;
  max-height: 10rem;
  overflow-y: auto;
}

.tag-picker-list button {
  width: 100%;
  text-align: left;
  padding: 0.4rem 0.5rem;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--color-text-primary);
  cursor: pointer;
  font-size: 0.8125rem;
}

.tag-picker-list button:hover {
  background: var(--color-accent-muted);
  color: var(--color-accent);
}

.tag-picker-empty {
  margin: 0.35rem 0 0;
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.tag-picker-create {
  margin-top: 0.35rem;
  width: 100%;
  text-align: left;
  padding: 0.4rem 0.5rem;
  border: none;
  border-radius: 4px;
  background: var(--color-accent-muted);
  color: var(--color-accent);
  cursor: pointer;
  font-size: 0.8125rem;
}

.prompt-content-section {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.content-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
}

.field-label {
  font-size: 0.8125rem;
  color: var(--color-text-muted);
}

.syntax-select {
  padding: 0.25rem 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-surface);
  color: var(--color-text-primary);
  font-size: 0.75rem;
}

.content-actions {
  display: flex;
  justify-content: flex-end;
}
</style>
