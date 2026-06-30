<script setup lang="ts">
import { computed } from "vue";
import { computeLineDiff } from "../../lib/line-diff";

const props = defineProps<{
  leftLabel: string;
  rightLabel: string;
  leftContent: string;
  rightContent: string;
}>();

const lines = computed(() => computeLineDiff(props.leftContent, props.rightContent));
</script>

<template>
  <div class="diff-view" data-testid="version-diff">
    <div class="diff-labels">
      <span>{{ leftLabel }}</span>
      <span>{{ rightLabel }}</span>
    </div>
    <pre class="diff-body"><code
      ><span
        v-for="(line, idx) in lines"
        :key="idx"
        class="diff-line"
        :class="`diff-${line.type}`"
        >{{ line.type === "remove" ? "- " : line.type === "add" ? "+ " : "  "
        }}{{ line.text }}
</span></code></pre>
  </div>
</template>

<style scoped>
.diff-view {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  min-height: 0;
}

.diff-labels {
  display: flex;
  justify-content: space-between;
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.diff-body {
  margin: 0;
  padding: 0.75rem;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-surface);
  overflow: auto;
  max-height: 16rem;
  font-size: 0.8125rem;
  line-height: 1.4;
}

.diff-line {
  display: block;
  white-space: pre-wrap;
  word-break: break-word;
}

.diff-remove {
  background: rgba(220, 53, 69, 0.15);
  color: var(--color-text-primary);
}

.diff-add {
  background: rgba(40, 167, 69, 0.15);
  color: var(--color-text-primary);
}
</style>
