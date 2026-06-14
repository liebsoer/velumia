<script setup lang="ts">
import { statusClass, statusLabel, type ConnectionWidgetState } from "../lib/api";

defineProps<{
  state: ConnectionWidgetState;
}>();
</script>

<template>
  <div class="connection-widget" :class="statusClass(state.status)">
    <span class="dot" />
    <span class="label">{{ statusLabel(state.status) }}</span>
    <p v-if="state.message" class="message">{{ state.message }}</p>
  </div>
</template>

<style scoped>
.connection-widget {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  border-radius: 8px;
  border: 1px solid var(--color-border);
  background: var(--color-surface-raised);
}

.dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: currentColor;
}

.label {
  font-weight: 600;
}

.message {
  flex-basis: 100%;
  margin: 0.25rem 0 0;
  font-size: 0.875rem;
  color: var(--color-text-muted);
}

.status-success {
  color: var(--color-success);
}

.status-error {
  color: var(--color-error);
}

.status-warning {
  color: var(--color-warning);
}

.status-muted {
  color: var(--color-text-muted);
}
</style>
