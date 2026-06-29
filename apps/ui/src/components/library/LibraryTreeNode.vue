<script setup lang="ts">
import FolderIcon from "./FolderIcon.vue";
import PromptIcon from "./PromptIcon.vue";
import PromptFavoriteIcon from "./PromptFavoriteIcon.vue";

export type TreeRowKind = "root" | "unfiled" | "folder" | "prompt";

defineProps<{
  kind: TreeRowKind;
  label: string;
  depth: number;
  expanded?: boolean;
  expandable?: boolean;
  active?: boolean;
  dimmed?: boolean;
  favorite?: boolean;
}>();

defineEmits<{
  click: [];
  toggle: [];
}>();
</script>

<template>
  <div
    class="tree-row"
    :class="{ active, dimmed }"
    :style="{ paddingLeft: `${0.5 + depth * 1}rem` }"
    :title="label"
    role="button"
    tabindex="0"
    @click="$emit('click')"
    @keydown.enter="$emit('click')"
  >
    <button
      v-if="expandable"
      type="button"
      class="chevron"
      :aria-expanded="expanded"
      @click.stop="$emit('toggle')"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="14"
        height="14"
        viewBox="0 0 14 14"
        fill="currentColor"
        aria-hidden="true"
        class="chevron-icon"
        :class="{ expanded }"
      >
        <path d="M5.25 3.5 9.75 7 5.25 10.5" />
      </svg>
    </button>
    <span v-else class="chevron-spacer" />
    <FolderIcon v-if="kind === 'root' || kind === 'unfiled' || kind === 'folder'" />
    <PromptFavoriteIcon v-else-if="favorite" />
    <PromptIcon v-else />
    <span class="tree-label">{{ label }}</span>
  </div>
</template>

<style scoped>
.tree-row {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  width: 100%;
  text-align: left;
  border: none;
  background: transparent;
  padding: 0.35rem 0.5rem;
  border-radius: 6px;
  cursor: pointer;
  color: var(--color-text-primary);
  min-width: 0;
}

.tree-row:hover {
  background: var(--color-accent-muted);
}

.tree-row.active {
  background: var(--color-accent-muted);
  color: var(--color-accent);
}

.tree-row.dimmed {
  opacity: 0.5;
  color: var(--color-text-muted);
}

.chevron {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.75rem;
  height: 1.75rem;
  flex-shrink: 0;
  border: none;
  background: transparent;
  color: inherit;
  cursor: pointer;
  padding: 0;
}

.chevron-icon {
  flex-shrink: 0;
  transition: transform 0.15s ease;
  transform: rotate(0deg);
  transform-origin: center;
}

.chevron-icon.expanded {
  transform: rotate(90deg);
}

.chevron-spacer {
  width: 1.75rem;
  flex-shrink: 0;
}

.tree-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}
</style>
