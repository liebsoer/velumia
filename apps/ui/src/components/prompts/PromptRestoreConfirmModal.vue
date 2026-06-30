<script setup lang="ts">
defineProps<{
  versionLabel: string;
}>();

const emit = defineEmits<{
  confirm: [];
  cancel: [];
}>();
</script>

<template>
  <div class="modal-backdrop" data-testid="restore-confirm-modal" @click.self="emit('cancel')">
    <div
      class="modal"
      role="dialog"
      aria-modal="true"
      aria-labelledby="restore-confirm-title"
      @click.stop
    >
      <h3 id="restore-confirm-title" class="modal-title">Restore as new version</h3>
      <p class="modal-body">
        Copy <strong>{{ versionLabel }}</strong> as a new head version? Existing history is
        preserved.
      </p>
      <div class="actions">
        <button type="button" @click="emit('cancel')">Cancel</button>
        <button type="button" data-testid="restore-confirm" @click="emit('confirm')">
          Restore as new version
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 40;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.45);
}

.modal {
  width: min(28rem, calc(100vw - 2rem));
  padding: 1.25rem;
  border-radius: 8px;
  border: 1px solid var(--color-border);
  background: var(--color-surface-raised);
}

.modal-title {
  margin: 0 0 0.75rem;
}

.modal-body {
  margin: 0 0 1rem;
  color: var(--color-text-muted);
  line-height: 1.45;
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
}
</style>
