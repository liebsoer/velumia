<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import {
  api,
  type PromptFolder,
  type PromptSummary,
  type TagSummary,
} from "../lib/api";
import FolderAddIcon from "../components/library/FolderAddIcon.vue";
import FolderIcon from "../components/library/FolderIcon.vue";
import LibraryTreeNode, {
  type TreeRowKind,
} from "../components/library/LibraryTreeNode.vue";
import PromptAddIcon from "../components/library/PromptAddIcon.vue";
import PromptFavoriteIcon from "../components/library/PromptFavoriteIcon.vue";
import PromptIcon from "../components/library/PromptIcon.vue";
import StarFilterIcon from "../components/library/StarFilterIcon.vue";
import PromptDetailPanel from "../components/prompts/PromptDetailPanel.vue";

const ALL_NODE_ID = "all";
const UNFILED_NODE_ID = "unfiled";

const props = defineProps<{ startCreate?: boolean }>();

const emit = defineEmits<{ openSettings: [] }>();

const allPrompts = ref<PromptSummary[]>([]);
const folders = ref<PromptFolder[]>([]);
const tags = ref<TagSummary[]>([]);
const selectedTagIds = ref<Set<string>>(new Set());
const favoritesOnly = ref(false);
const selectedNavKey = ref<string>(ALL_NODE_ID);
const selectedPromptId = ref<string | null>(null);
const viewMode = ref<"idle" | "folder" | "detail">("idle");
const expandedKeys = ref<Set<string>>(new Set());
const error = ref("");
const loading = ref(false);

const showCreatePrompt = ref(false);
const newPromptTitle = ref("");
const newPromptFolderId = ref<string | null>(null);
const showCreateFolder = ref(false);
const newFolderTitle = ref("");
const newFolderParentId = ref<string | null>(null);
const showMoveModal = ref(false);
const moveTargetFolderId = ref<string | null>(null);
const showDeleteConfirm = ref(false);
const showTagFilterMenu = ref(false);
const tagFilterAnchorRef = ref<HTMLElement | null>(null);
const detailPanelRef = ref<InstanceType<typeof PromptDetailPanel> | null>(null);

interface TreeRow {
  key: string;
  kind: TreeRowKind;
  label: string;
  depth: number;
  expandable: boolean;
  promptId?: string;
  favorite?: boolean;
  isEmpty?: boolean;
}

interface FolderChildItem {
  key: string;
  kind: "folder" | "prompt" | "unfiled";
  label: string;
  favorite?: boolean;
}

const rootFolders = computed(() => folders.value.filter((f) => !f.parent_id));
const childFolders = (parentId: string) =>
  folders.value.filter((f) => f.parent_id === parentId);

const filteredPrompts = computed(() =>
  allPrompts.value.filter((p) => {
    if (favoritesOnly.value && !p.is_favorite) return false;
    if (selectedTagIds.value.size > 0) {
      const matchesTag = p.tags.some((t) => selectedTagIds.value.has(t.id));
      if (!matchesTag) return false;
    }
    return true;
  }),
);

const sortedTags = computed(() =>
  [...tags.value].sort((a, b) => a.name.localeCompare(b.name)),
);

const tagFilterActive = computed(() => selectedTagIds.value.size > 0);

const tagFilterLabel = computed(() => {
  const count = selectedTagIds.value.size;
  if (count === 0) return "All tags";
  if (count === 1) {
    const id = [...selectedTagIds.value][0];
    return tags.value.find((t) => t.id === id)?.name ?? "1 tag";
  }
  return `${count} tags`;
});

const promptsByFolder = computed(() => {
  const map = new Map<string | null, PromptSummary[]>();
  for (const p of filteredPrompts.value) {
    const key = p.folder_id;
    if (!map.has(key)) map.set(key, []);
    map.get(key)!.push(p);
  }
  for (const list of map.values()) {
    list.sort((a, b) => a.title.localeCompare(b.title));
  }
  return map;
});

const selectedPrompt = computed(
  () => allPrompts.value.find((p) => p.id === selectedPromptId.value) ?? null,
);

function countInFolder(folderId: string): number {
  return filteredPrompts.value.filter((p) => p.folder_id === folderId).length;
}

function countUnfiled(): number {
  return filteredPrompts.value.filter((p) => !p.folder_id).length;
}

function folderSubtreeCount(folderId: string): number {
  let n = countInFolder(folderId);
  for (const child of childFolders(folderId)) {
    n += countInFolder(child.id);
  }
  return n;
}

function promptRow(prompt: PromptSummary, depth: number): TreeRow {
  return {
    key: `prompt-${prompt.id}`,
    kind: "prompt",
    label: prompt.title,
    depth,
    expandable: false,
    promptId: prompt.id,
    favorite: prompt.is_favorite,
  };
}

function buildFolderRows(rows: TreeRow[], folder: PromptFolder, depth: number) {
  const children = childFolders(folder.id);
  const hasChildFolders = children.length > 0;
  const promptCount = (promptsByFolder.value.get(folder.id) ?? []).length;
  const subtreeCount = folderSubtreeCount(folder.id);

  rows.push({
    key: folder.id,
    kind: "folder",
    label: folder.title,
    depth,
    expandable: hasChildFolders || promptCount > 0 || children.length > 0,
    isEmpty: subtreeCount === 0,
  });

  if (!expandedKeys.value.has(folder.id)) return;

  for (const p of promptsByFolder.value.get(folder.id) ?? []) {
    rows.push(promptRow(p, depth + 1));
  }
  for (const child of children) {
    buildFolderRows(rows, child, depth + 1);
  }
}

const treeRows = computed(() => {
  const rows: TreeRow[] = [];
  const allEmpty = filteredPrompts.value.length === 0;

  rows.push({
    key: ALL_NODE_ID,
    kind: "root",
    label: "All prompts",
    depth: 0,
    expandable: true,
    isEmpty: allEmpty,
  });

  if (!expandedKeys.value.has(ALL_NODE_ID)) return rows;

  for (const root of rootFolders.value) {
    buildFolderRows(rows, root, 1);
  }

  const unfiledEmpty = countUnfiled() === 0;
  rows.push({
    key: UNFILED_NODE_ID,
    kind: "unfiled",
    label: "Unfiled",
    depth: 1,
    expandable: true,
    isEmpty: unfiledEmpty,
  });

  if (expandedKeys.value.has(UNFILED_NODE_ID)) {
    for (const p of promptsByFolder.value.get(null) ?? []) {
      rows.push(promptRow(p, 2));
    }
  }

  return rows;
});

const folderOverviewTitle = computed(() => {
  if (selectedNavKey.value === ALL_NODE_ID) return "All prompts";
  if (selectedNavKey.value === UNFILED_NODE_ID) return "Unfiled";
  return folders.value.find((f) => f.id === selectedNavKey.value)?.title ?? "Folder";
});

const folderOverviewCount = computed(() => {
  if (selectedNavKey.value === ALL_NODE_ID) return filteredPrompts.value.length;
  if (selectedNavKey.value === UNFILED_NODE_ID) return countUnfiled();
  return folderSubtreeCount(selectedNavKey.value);
});

const activeFolderChildren = computed((): FolderChildItem[] => {
  const key = selectedNavKey.value;
  const items: FolderChildItem[] = [];

  if (key === ALL_NODE_ID) {
    for (const folder of rootFolders.value) {
      items.push({ key: folder.id, kind: "folder", label: folder.title });
    }
    items.push({ key: UNFILED_NODE_ID, kind: "unfiled", label: "Unfiled" });
    return items;
  }

  if (key === UNFILED_NODE_ID) {
    for (const prompt of promptsByFolder.value.get(null) ?? []) {
      items.push({
        key: prompt.id,
        kind: "prompt",
        label: prompt.title,
        favorite: prompt.is_favorite,
      });
    }
    return items;
  }

  for (const folder of childFolders(key)) {
    items.push({ key: folder.id, kind: "folder", label: folder.title });
  }
  for (const prompt of promptsByFolder.value.get(key) ?? []) {
    items.push({
      key: prompt.id,
      kind: "prompt",
      label: prompt.title,
      favorite: prompt.is_favorite,
    });
  }
  return items;
});

const locationBreadcrumb = computed(() => {
  const prompt = selectedPrompt.value;
  if (!prompt) return [];
  if (!prompt.folder_id) return ["Unfiled"];
  const folder = folders.value.find((f) => f.id === prompt.folder_id);
  if (!folder) return [];
  const parts: string[] = [];
  if (folder.parent_id) {
    const parent = folders.value.find((f) => f.id === folder.parent_id);
    if (parent) parts.push(parent.title);
  }
  parts.push(folder.title);
  return parts;
});

function syncExpandedDefaults() {
  const next = new Set<string>([ALL_NODE_ID, UNFILED_NODE_ID]);
  for (const f of folders.value) next.add(f.id);
  expandedKeys.value = next;
}

function defaultCreateFolderId(): string | null {
  if (
    selectedNavKey.value !== ALL_NODE_ID &&
    selectedNavKey.value !== UNFILED_NODE_ID
  ) {
    return selectedNavKey.value;
  }
  return null;
}

function resetCreatePromptForm() {
  newPromptTitle.value = "";
  newPromptFolderId.value = defaultCreateFolderId();
}

function resetCreateFolderForm() {
  newFolderTitle.value = "";
  newFolderParentId.value = null;
}

function openCreatePrompt() {
  resetCreatePromptForm();
  showCreatePrompt.value = true;
}

function closeCreatePrompt() {
  resetCreatePromptForm();
  showCreatePrompt.value = false;
}

function openCreateFolder() {
  resetCreateFolderForm();
  showCreateFolder.value = true;
}

function closeCreateFolder() {
  resetCreateFolderForm();
  showCreateFolder.value = false;
}

function ensureTreeExpandedForFolderKey(key: string) {
  const next = new Set(expandedKeys.value);
  next.add(ALL_NODE_ID);
  if (key === ALL_NODE_ID) {
    expandedKeys.value = next;
    return;
  }
  if (key === UNFILED_NODE_ID) {
    next.add(UNFILED_NODE_ID);
    expandedKeys.value = next;
    return;
  }
  const folder = folders.value.find((f) => f.id === key);
  if (folder?.parent_id) next.add(folder.parent_id);
  next.add(key);
  expandedKeys.value = next;
}

function ensureTreeExpandedForPrompt(promptId: string) {
  const prompt = allPrompts.value.find((p) => p.id === promptId);
  if (!prompt) return;
  const next = new Set(expandedKeys.value);
  next.add(ALL_NODE_ID);
  if (!prompt.folder_id) {
    next.add(UNFILED_NODE_ID);
  } else {
    const folder = folders.value.find((f) => f.id === prompt.folder_id);
    if (folder?.parent_id) next.add(folder.parent_id);
    if (folder) next.add(folder.id);
  }
  expandedKeys.value = next;
}

function openMoveModal() {
  moveTargetFolderId.value = selectedPrompt.value?.folder_id ?? null;
  showMoveModal.value = true;
}

function closeMoveModal() {
  showMoveModal.value = false;
  moveTargetFolderId.value = null;
}

async function openPromptDetail(promptId: string) {
  if (
    selectedPromptId.value &&
    selectedPromptId.value !== promptId &&
    detailPanelRef.value
  ) {
    const ok = await detailPanelRef.value.confirmLeaveIfDirty();
    if (!ok) return;
  }
  ensureTreeExpandedForPrompt(promptId);
  selectedPromptId.value = promptId;
  viewMode.value = "detail";
}

async function selectFolderNav(key: string) {
  if (viewMode.value === "detail" && detailPanelRef.value) {
    const ok = await detailPanelRef.value.confirmLeaveIfDirty();
    if (!ok) return;
  }
  ensureTreeExpandedForFolderKey(key);
  selectedNavKey.value = key;
  selectedPromptId.value = null;
  viewMode.value = "folder";
}

function onFolderChildClick(item: FolderChildItem) {
  if (item.kind === "prompt") {
    openPromptDetail(item.key);
    return;
  }
  selectFolderNav(item.key);
}

function isFolderChildActive(item: FolderChildItem): boolean {
  if (item.kind === "prompt") {
    return viewMode.value === "detail" && selectedPromptId.value === item.key;
  }
  return viewMode.value === "folder" && selectedNavKey.value === item.key;
}

async function clearDetailSelection() {
  if (detailPanelRef.value) {
    const ok = await detailPanelRef.value.confirmLeaveIfDirty();
    if (!ok) return;
  }
  selectedPromptId.value = null;
  viewMode.value = "idle";
}

function openDeleteConfirm() {
  showDeleteConfirm.value = true;
}

function closeDeleteConfirm() {
  showDeleteConfirm.value = false;
}

function closeTagFilterMenu() {
  showTagFilterMenu.value = false;
}

function toggleTagFilterMenu() {
  if (showTagFilterMenu.value) closeTagFilterMenu();
  else showTagFilterMenu.value = true;
}

function isNavTagSelected(tagId: string) {
  return selectedTagIds.value.has(tagId);
}

function toggleNavTagFilter(tagId: string) {
  const next = new Set(selectedTagIds.value);
  if (next.has(tagId)) next.delete(tagId);
  else next.add(tagId);
  selectedTagIds.value = next;
}

function clearNavTagFilters() {
  selectedTagIds.value = new Set();
}

function onDocumentPointerDown(e: PointerEvent) {
  const target = e.target as Node;
  if (showTagFilterMenu.value) {
    const anchor = tagFilterAnchorRef.value;
    if (anchor && !anchor.contains(target)) closeTagFilterMenu();
  }
}

function toggleExpand(key: string) {
  const next = new Set(expandedKeys.value);
  if (next.has(key)) next.delete(key);
  else next.add(key);
  expandedKeys.value = next;
}

function isRowActive(row: TreeRow): boolean {
  if (row.kind === "prompt") {
    return viewMode.value === "detail" && selectedPromptId.value === row.promptId;
  }
  if (viewMode.value === "folder") return selectedNavKey.value === row.key;
  return false;
}

function onTreeRowClick(row: TreeRow) {
  if (row.kind === "prompt" && row.promptId) {
    openPromptDetail(row.promptId);
    return;
  }
  selectFolderNav(row.key);
}

async function onEscapeKey(e: KeyboardEvent) {
  if (e.key !== "Escape") return;
  if (showCreatePrompt.value) closeCreatePrompt();
  else if (showCreateFolder.value) closeCreateFolder();
  else if (showMoveModal.value) closeMoveModal();
  else if (showDeleteConfirm.value) closeDeleteConfirm();
  else if (showTagFilterMenu.value) closeTagFilterMenu();
  else if (viewMode.value === "detail" && detailPanelRef.value?.handleEscape()) return;
  else if (viewMode.value === "detail") await clearDetailSelection();
}

async function refresh() {
  loading.value = true;
  error.value = "";
  try {
    [allPrompts.value, folders.value, tags.value] = await Promise.all([
      api.listPrompts({}),
      api.listPromptFolders(),
      api.listTags(),
    ]);
    syncExpandedDefaults();
    if (
      selectedPromptId.value &&
      !allPrompts.value.some((p) => p.id === selectedPromptId.value)
    ) {
      await clearDetailSelection();
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
  }
}

async function createPrompt() {
  if (!newPromptTitle.value.trim()) {
    error.value = "Title is required";
    return;
  }
  try {
    const id = await api.createPrompt(
      newPromptTitle.value.trim(),
      newPromptFolderId.value,
    );
    closeCreatePrompt();
    await refresh();
    if (newPromptFolderId.value) {
      expandedKeys.value = new Set([...expandedKeys.value, newPromptFolderId.value]);
    } else {
      expandedKeys.value = new Set([...expandedKeys.value, UNFILED_NODE_ID]);
    }
    openPromptDetail(id);
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}

async function createFolder() {
  if (!newFolderTitle.value.trim()) {
    error.value = "Folder title is required";
    return;
  }
  try {
    await api.createPromptFolder(newFolderTitle.value.trim(), newFolderParentId.value);
    closeCreateFolder();
    await refresh();
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}

async function confirmTrashSelected() {
  if (!selectedPromptId.value) return;
  try {
    await api.trashPrompt(selectedPromptId.value);
    closeDeleteConfirm();
    await clearDetailSelection();
    await refresh();
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}

async function confirmMove() {
  if (!selectedPromptId.value) return;
  try {
    await api.movePromptToFolder(selectedPromptId.value, moveTargetFolderId.value);
    closeMoveModal();
    await refresh();
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}

async function toggleFavorite() {
  const prompt = selectedPrompt.value;
  if (!prompt) return;
  try {
    if (prompt.is_favorite) {
      await api.unsetPromptFavorite(prompt.id);
    } else {
      await api.setPromptFavorite(prompt.id);
    }
    await refresh();
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}

onMounted(async () => {
  window.addEventListener("keydown", onEscapeKey);
  document.addEventListener("pointerdown", onDocumentPointerDown);
  await refresh();
  if (props.startCreate) openCreatePrompt();
});

onUnmounted(() => {
  window.removeEventListener("keydown", onEscapeKey);
  document.removeEventListener("pointerdown", onDocumentPointerDown);
});

defineExpose({ refresh });
</script>

<template>
  <main class="library" data-testid="prompt-library">
    <p v-if="error" class="error">{{ error }}</p>
    <p v-if="loading" class="hint">Loading…</p>

    <div class="panels">
      <aside class="library-nav">
        <h1 class="nav-title">Prompts</h1>

        <div class="nav-toolbar">
          <button
            type="button"
            class="icon-btn"
            data-testid="new-prompt"
            aria-label="New prompt"
            title="New prompt"
            @click="openCreatePrompt"
          >
            <PromptAddIcon />
          </button>
          <button
            type="button"
            class="icon-btn"
            aria-label="New folder"
            title="New folder"
            @click="openCreateFolder"
          >
            <FolderAddIcon />
          </button>
          <button
            type="button"
            class="icon-btn"
            :class="{ active: favoritesOnly }"
            data-testid="favorites-filter"
            :aria-label="favoritesOnly ? 'Show all prompts' : 'Show favorites only'"
            :aria-pressed="favoritesOnly"
            :title="favoritesOnly ? 'Show all prompts' : 'Show favorites only'"
            @click="favoritesOnly = !favoritesOnly"
          >
            <StarFilterIcon :active="favoritesOnly" />
          </button>
          <span ref="tagFilterAnchorRef" class="tag-filter-anchor">
            <button
              type="button"
              class="tag-filter"
              :class="{ active: tagFilterActive }"
              data-testid="tag-filter"
              aria-haspopup="listbox"
              :aria-expanded="showTagFilterMenu"
              @click.stop="toggleTagFilterMenu"
            >
              <span class="tag-filter-label">{{ tagFilterLabel }}</span>
            </button>
            <div
              v-if="showTagFilterMenu"
              class="tag-filter-menu"
              role="listbox"
              aria-label="Filter by tags"
              aria-multiselectable="true"
              @click.stop
            >
              <button
                v-if="tagFilterActive"
                type="button"
                class="tag-filter-clear"
                @click="clearNavTagFilters"
              >
                Clear filters
              </button>
              <label
                v-for="tag in sortedTags"
                :key="tag.id"
                class="tag-filter-option"
                role="option"
                :aria-selected="isNavTagSelected(tag.id)"
              >
                <input
                  type="checkbox"
                  :checked="isNavTagSelected(tag.id)"
                  @change="toggleNavTagFilter(tag.id)"
                />
                <span>{{ tag.name }}</span>
              </label>
              <p v-if="sortedTags.length === 0" class="tag-filter-empty">No tags yet</p>
            </div>
          </span>
        </div>

        <div class="tree-scroll" data-testid="library-tree">
          <LibraryTreeNode
            v-for="row in treeRows"
            :key="row.key"
            :kind="row.kind"
            :label="row.label"
            :depth="row.depth"
            :expandable="row.expandable"
            :expanded="expandedKeys.has(row.key)"
            :active="isRowActive(row)"
            :dimmed="!!row.isEmpty && (favoritesOnly || tagFilterActive)"
            :favorite="row.favorite"
            @click="onTreeRowClick(row)"
            @toggle="toggleExpand(row.key)"
          />
        </div>
      </aside>

      <section class="main-panel">
        <PromptDetailPanel
          v-if="viewMode === 'detail' && selectedPrompt"
          ref="detailPanelRef"
          :prompt="selectedPrompt"
          :tags="tags"
          :location-breadcrumb="locationBreadcrumb"
          @refresh="refresh"
          @error="error = $event"
          @open-move="openMoveModal"
          @open-delete="openDeleteConfirm"
          @favorite-toggle="toggleFavorite"
          @open-settings="emit('openSettings')"
        />

        <div v-else-if="viewMode === 'folder'" class="folder-overview" data-testid="folder-overview">
          <h2>{{ folderOverviewTitle }}</h2>
          <p class="muted">{{ folderOverviewCount }} prompt(s)</p>
          <ul v-if="activeFolderChildren.length" class="folder-children">
            <li v-for="item in activeFolderChildren" :key="item.key">
              <button
                type="button"
                class="folder-child-row"
                :class="{ active: isFolderChildActive(item) }"
                :data-testid="item.kind === 'prompt' ? 'folder-child-prompt' : 'folder-child-folder'"
                @click="onFolderChildClick(item)"
              >
                <FolderIcon v-if="item.kind === 'folder' || item.kind === 'unfiled'" />
                <PromptFavoriteIcon v-else-if="item.favorite" />
                <PromptIcon v-else />
                <span class="folder-child-label">{{ item.label }}</span>
              </button>
            </li>
          </ul>
          <p v-else class="hint">No items in this folder</p>
        </div>

        <div v-else class="empty" data-testid="empty-library">
          <p>Select a folder or prompt from the tree</p>
        </div>
      </section>
    </div>

    <div v-if="showCreatePrompt" class="modal-backdrop" @click.self="closeCreatePrompt">
      <div
        class="modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="new-prompt-title"
        @click.stop
      >
        <form @submit.prevent="createPrompt">
          <h3 id="new-prompt-title" class="modal-title">New prompt</h3>
          <label class="field">
            Title
            <input v-model="newPromptTitle" placeholder="Title" autofocus />
          </label>
          <label class="field">
            Folder
            <select v-model="newPromptFolderId" data-testid="new-prompt-folder">
              <option :value="null">Unfiled (no folder)</option>
              <option v-for="folder in rootFolders" :key="folder.id" :value="folder.id">
                {{ folder.title }}
              </option>
              <template v-for="root in rootFolders" :key="`child-group-${root.id}`">
                <option
                  v-for="child in childFolders(root.id)"
                  :key="child.id"
                  :value="child.id"
                >
                  — {{ child.title }}
                </option>
              </template>
            </select>
          </label>
          <div class="actions">
            <button type="button" @click="closeCreatePrompt">Cancel</button>
            <button type="submit">Create</button>
          </div>
        </form>
      </div>
    </div>

    <div v-if="showCreateFolder" class="modal-backdrop" @click.self="closeCreateFolder">
      <div
        class="modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="new-folder-title"
        @click.stop
      >
        <form @submit.prevent="createFolder">
          <h3 id="new-folder-title" class="modal-title">New folder</h3>
          <label class="field">
            Folder title
            <input v-model="newFolderTitle" placeholder="Folder title" autofocus />
          </label>
          <label class="field">
            Parent folder
            <select v-model="newFolderParentId">
              <option :value="null">Root</option>
              <option v-for="folder in rootFolders" :key="folder.id" :value="folder.id">
                Under {{ folder.title }}
              </option>
            </select>
          </label>
          <div class="actions">
            <button type="button" @click="closeCreateFolder">Cancel</button>
            <button type="submit">Create</button>
          </div>
        </form>
      </div>
    </div>

    <div v-if="showDeleteConfirm" class="modal-backdrop" @click.self="closeDeleteConfirm">
      <div
        class="modal"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="delete-title"
        aria-describedby="delete-desc"
        @click.stop
      >
        <h3 id="delete-title" class="modal-title">Move to trash?</h3>
        <p id="delete-desc" class="confirm-text">
          “{{ selectedPrompt?.title }}” will be moved to trash. This action cannot be undone
          from here.
        </p>
        <div class="actions">
          <button type="button" @click="closeDeleteConfirm">Cancel</button>
          <button type="button" class="danger" data-testid="confirm-trash" @click="confirmTrashSelected">
            Move to trash
          </button>
        </div>
      </div>
    </div>

    <div v-if="showMoveModal" class="modal-backdrop" @click.self="closeMoveModal">
      <div class="modal" role="dialog" aria-modal="true" aria-labelledby="move-title" @click.stop>
        <form @submit.prevent="confirmMove">
          <h3 id="move-title" class="modal-title">Move to folder</h3>
          <label class="field">
            Destination
            <select v-model="moveTargetFolderId">
              <option :value="null">Unfiled (no folder)</option>
              <option v-for="folder in rootFolders" :key="folder.id" :value="folder.id">
                {{ folder.title }}
              </option>
              <template v-for="root in rootFolders" :key="`move-child-${root.id}`">
                <option
                  v-for="child in childFolders(root.id)"
                  :key="child.id"
                  :value="child.id"
                >
                  — {{ child.title }}
                </option>
              </template>
            </select>
          </label>
          <div class="actions">
            <button type="button" @click="closeMoveModal">Cancel</button>
            <button type="submit">Move</button>
          </div>
        </form>
      </div>
    </div>
  </main>
</template>

<style scoped>
.library {
  flex: 1;
  min-width: 0;
  min-height: 0;
  height: 100%;
  padding: 1.5rem 2rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.panels {
  display: grid;
  grid-template-columns: 300px minmax(0, 1fr);
  gap: 1rem;
  flex: 1;
  min-height: 0;
}

.library-nav {
  width: 300px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  min-height: 0;
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 0.75rem;
  overflow: hidden;
}

.nav-title {
  margin: 0;
  font-size: 1.25rem;
}

.nav-toolbar {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 0.375rem;
}

.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 2rem;
  height: 2rem;
  padding: 0;
  color: var(--color-text-muted);
}

.icon-btn:hover {
  color: var(--color-text-primary);
  border-color: var(--color-text-muted);
}

.icon-btn.active {
  color: var(--color-warning);
  background: var(--color-warning-muted);
  border-color: var(--color-warning);
}

.field input,
.field select,
.tag-filter {
  box-sizing: border-box;
  min-height: 2rem;
  height: 2rem;
  padding: 0 1.75rem 0 0.625rem;
  line-height: 1.25;
  font-size: 0.8125rem;
  background-color: var(--color-surface);
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%238b949e' d='M3 4.5 6 7.5 9 4.5'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 0.5rem center;
  background-size: 0.75rem;
}

.tag-filter {
  flex: 1;
  min-width: 0;
  width: auto;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  color: var(--color-text-primary);
  cursor: pointer;
  text-align: left;
}

.tag-filter.active {
  color: var(--color-accent);
  border-color: var(--color-accent);
  background-color: var(--color-accent-muted);
}

.tag-filter-label {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tag-filter-anchor {
  position: relative;
  flex: 1;
  min-width: 0;
}

.tag-filter-menu {
  position: absolute;
  top: calc(100% + 0.35rem);
  left: 0;
  right: 0;
  z-index: 20;
  max-height: 14rem;
  overflow-y: auto;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  background: var(--color-surface-raised);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
  padding: 0.35rem;
}

.tag-filter-clear {
  display: block;
  width: 100%;
  padding: 0.4rem 0.5rem;
  margin-bottom: 0.25rem;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--color-accent);
  font-size: 0.75rem;
  text-align: left;
}

.tag-filter-clear:hover {
  background: var(--color-accent-muted);
}

.tag-filter-option {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.4rem 0.5rem;
  border-radius: 6px;
  font-size: 0.8125rem;
  color: var(--color-text-primary);
  cursor: pointer;
}

.tag-filter-option:hover {
  background: var(--color-accent-muted);
}

.tag-filter-option input {
  flex-shrink: 0;
  margin: 0;
  accent-color: var(--color-accent);
}

.tag-filter-empty {
  margin: 0.25rem 0.5rem;
  font-size: 0.8125rem;
  color: var(--color-text-muted);
}

.nav-toolbar .tag-filter {
  padding: 0 1.75rem 0 0.625rem;
}

.field input {
  -webkit-appearance: none;
  appearance: none;
  background-image: none;
  padding: 0.625rem 0.875rem;
  height: auto;
  min-height: 2.75rem;
  font-size: 1rem;
}

.field select {
  width: 100%;
  -webkit-appearance: none;
  appearance: none;
}

.field input,
.field select {
  -webkit-appearance: none;
  appearance: none;
}

.tree-scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.main-panel {
  min-width: 0;
  width: 100%;
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 1rem;
  overflow-y: auto;
}

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

.detail h2,
.folder-overview h2 {
  margin: 0;
  color: var(--color-text-primary);
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

.detail-toolbar .icon-btn {
  padding: 0;
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

.prompt-content-editor {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.prompt-content-editor .field-label {
  display: block;
  margin-bottom: 0.35rem;
  font-size: 0.8125rem;
  color: var(--color-text-muted);
}

.prompt-content-textarea {
  width: 100%;
  min-height: 12rem;
  padding: 0.625rem 0.75rem;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-surface);
  color: var(--color-text-primary);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 0.8125rem;
  line-height: 1.45;
  resize: vertical;
}

.content-actions {
  display: flex;
  justify-content: flex-end;
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
}

.detail-menu-item:hover {
  background: var(--color-accent-muted);
  color: var(--color-accent);
}

.detail-menu-item-danger {
  color: var(--color-error, #f87171);
}

.detail-menu-item-danger:hover {
  background: rgba(248, 81, 73, 0.12);
  color: #fca5a5;
}


.detail-toolbar .icon-btn.active {
  color: var(--color-warning);
  background: var(--color-warning-muted);
  border-color: var(--color-warning);
}

.tag-badges {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.25rem;
}

.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.15rem;
  font-size: 0.625rem;
  font-weight: 500;
  line-height: 1;
  padding: 0.15rem 0.4rem;
  border-radius: 999px;
  background: var(--color-accent-muted);
  color: var(--color-accent);
  border: 1px solid rgba(108, 182, 255, 0.35);
}

.status-badge-remove {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 0.75rem;
  height: 0.75rem;
  padding: 0;
  border: none;
  background: transparent;
  color: inherit;
  opacity: 0.75;
  font-size: 0.6875rem;
  line-height: 1;
}

.status-badge-remove:hover {
  opacity: 1;
}

.status-badge-add {
  cursor: pointer;
  padding: 0.12rem 0.3rem;
  border-style: dashed;
  background: transparent;
  color: var(--color-text-muted);
}

.status-badge-add :deep(.lib-icon) {
  width: 10px;
  height: 10px;
}

.status-badge-add:hover,
.status-badge-add.active {
  color: var(--color-accent);
  border-color: var(--color-accent);
  background: var(--color-accent-muted);
}

.tag-picker-anchor {
  position: relative;
}

.tag-picker {
  position: absolute;
  top: calc(100% + 0.35rem);
  left: 0;
  z-index: 20;
  min-width: min(16rem, 70vw);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  background: var(--color-surface-raised);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
  padding: 0.5rem;
}

.tag-picker-search {
  width: 100%;
  box-sizing: border-box;
  min-height: 2rem;
  padding: 0.45rem 0.625rem;
  font-size: 0.8125rem;
  margin-bottom: 0.35rem;
}

.tag-picker-list {
  list-style: none;
  margin: 0;
  padding: 0;
  max-height: 10rem;
  overflow-y: auto;
}

.tag-picker-list li button {
  width: 100%;
  text-align: left;
  padding: 0.45rem 0.5rem;
  border: none;
  background: transparent;
  color: var(--color-text-primary);
  font-size: 0.8125rem;
}

.tag-picker-list li button:hover {
  background: var(--color-accent-muted);
  color: var(--color-accent);
}

.tag-picker-empty {
  margin: 0.25rem 0.5rem;
  font-size: 0.8125rem;
  color: var(--color-text-muted);
}

.tag-picker-create {
  width: 100%;
  text-align: left;
  padding: 0.45rem 0.5rem;
  border: none;
  background: transparent;
  color: var(--color-accent);
  font-size: 0.8125rem;
}

.tag-picker-create:hover {
  background: var(--color-accent-muted);
}

.confirm-text {
  margin: 0 0 1rem;
  color: var(--color-text-muted);
  font-size: 0.875rem;
  line-height: 1.5;
}

.folder-overview .muted {
  color: var(--color-text-muted);
  margin: 0.25rem 0 0.75rem;
}

.folder-children {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.folder-child-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.5rem 0.625rem;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--color-text-primary);
  font-size: 0.875rem;
  text-align: left;
  cursor: pointer;
}

.folder-child-row:hover {
  background: var(--color-accent-muted);
}

.folder-child-row.active {
  background: var(--color-accent-muted);
  color: var(--color-accent);
}

.folder-child-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}

.row {
  display: flex;
  gap: 0.5rem;
}

.empty {
  padding: 2rem;
  text-align: center;
  color: var(--color-text-muted);
}

.error {
  color: var(--color-error, #f87171);
}

.hint {
  color: var(--color-text-muted);
}

.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.55);
  padding: 1rem;
}

.modal {
  width: min(768px, 100%);
  max-height: calc(100vh - 2rem);
  overflow: auto;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 1.25rem;
  background: var(--color-surface-raised);
}

.modal-title {
  color: var(--color-text-primary);
  margin: 0 0 1rem;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  margin-bottom: 0.75rem;
  color: var(--color-text-primary);
  font-size: 0.875rem;
}

.actions {
  display: flex;
  gap: 0.5rem;
  margin-top: 0.75rem;
}

.danger {
  color: #f87171;
}

button.danger {
  border-color: rgba(248, 81, 73, 0.45);
}

button.danger:hover {
  background: rgba(248, 81, 73, 0.12);
}

input,
select,
button {
  border-radius: 6px;
  border: 1px solid var(--color-border);
  background: var(--color-surface);
  color: var(--color-text-primary);
}

button,
.row input {
  padding: 0.5rem 0.75rem;
}

.nav-toolbar .icon-btn {
  padding: 0;
}

</style>
