<script setup lang="ts">
defineProps<{ active?: boolean }>();
</script>

<template>
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="16"
    height="16"
    viewBox="0 0 16 16"
    aria-hidden="true"
    class="lib-icon"
    :class="{ active }"
  >
    <path
      v-if="active"
      d="M8 2.2 9.76 5.77l3.94.57-2.85 2.78.67 3.92L8 11.27l-3.52 1.85.67-3.92L2.3 6.34l3.94-.57L8 2.2Z"
      fill="currentColor"
    />
    <path
      v-else
      d="M8 2.2 9.76 5.77l3.94.57-2.85 2.78.67 3.92L8 11.27l-3.52 1.85.67-3.92L2.3 6.34l3.94-.57L8 2.2Z"
      fill="none"
      stroke="currentColor"
      stroke-width="1.1"
      stroke-linejoin="round"
    />
  </svg>
</template>

<style scoped>
.lib-icon {
  flex-shrink: 0;
  color: var(--color-text-muted);
}

.lib-icon.active {
  color: var(--color-warning);
}
</style>
