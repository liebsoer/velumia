<script setup lang="ts">
defineEmits<{
  save: [];
  discard: [];
  saveDraft: [];
}>();
</script>

<template>
  <div class="modal-backdrop" data-testid="unsaved-changes-modal" @click.self="$emit('discard')">
    <div
      class="modal"
      role="dialog"
      aria-modal="true"
      aria-labelledby="unsaved-changes-title"
      @click.stop
    >
      <h3 id="unsaved-changes-title" class="modal-title">Unsaved changes</h3>
      <p class="modal-body">You have unsaved edits. What would you like to do?</p>
      <div class="actions">
        <button type="button" data-testid="unsaved-discard" @click="$emit('discard')">
          Discard
        </button>
        <button type="button" data-testid="unsaved-save-draft" @click="$emit('saveDraft')">
          Save draft
        </button>
        <button type="button" data-testid="unsaved-save" @click="$emit('save')">Save</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 50;
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
  flex-wrap: wrap;
}
</style>
