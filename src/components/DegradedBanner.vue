<script setup lang="ts">
defineProps<{ visible: boolean }>();
defineEmits<{ dismiss: [] }>();
</script>

<template>
  <div v-if="visible" class="banner">
    <span>LangDock connection needs attention.</span>
    <button type="button" @click="$emit('dismiss')">Dismiss</button>
  </div>
</template>

<style scoped>
.banner {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem 1rem;
  background: var(--color-warning-muted);
  color: var(--color-warning);
  border-bottom: 1px solid var(--color-border);
}

button {
  background: transparent;
  border: 1px solid currentColor;
  color: inherit;
  border-radius: 4px;
  padding: 0.25rem 0.5rem;
  cursor: pointer;
}
</style>
