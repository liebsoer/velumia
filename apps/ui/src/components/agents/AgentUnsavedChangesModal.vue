<script setup lang="ts">
defineEmits<{
  save: [];
  discard: [];
  cancel: [];
}>();
</script>

<template>
  <div class="overlay" role="alertdialog" data-testid="agent-unsaved-dialog">
    <div class="dialog">
      <h2>Unsaved changes</h2>
      <p>You have unsaved agent edits.</p>
      <div class="actions">
        <button type="button" @click="$emit('save')">Save</button>
        <button type="button" @click="$emit('discard')">Discard</button>
        <button type="button" class="muted" @click="$emit('cancel')">Cancel</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  display: grid;
  place-items: center;
  z-index: 100;
}

.dialog {
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  padding: 1.25rem;
  min-width: 20rem;
}

.actions {
  display: flex;
  gap: 0.5rem;
  margin-top: 1rem;
}

.muted {
  margin-left: auto;
}
</style>
