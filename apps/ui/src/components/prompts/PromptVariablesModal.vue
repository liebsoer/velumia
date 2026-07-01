<script setup lang="ts">
import { computed, ref } from "vue";

const props = defineProps<{
  placeholders: string[];
}>();

const emit = defineEmits<{
  confirm: [variables: Record<string, string>, allowEmpty: boolean];
  cancel: [];
}>();

const values = ref<Record<string, string>>(
  Object.fromEntries(props.placeholders.map((name) => [name, ""])),
);
const allowEmpty = ref(false);

const hasEmptyValues = computed(() =>
  props.placeholders.some((name) => !values.value[name]?.trim()),
);

const runDisabled = computed(() => hasEmptyValues.value && !allowEmpty.value);

function onConfirm() {
  const out: Record<string, string> = {};
  for (const name of props.placeholders) {
    out[name] = values.value[name]?.trim() ?? "";
  }
  emit("confirm", out, allowEmpty.value);
}
</script>

<template>
  <div class="modal-backdrop" data-testid="prompt-variables-modal" @click.self="emit('cancel')">
    <div
      class="modal"
      role="dialog"
      aria-modal="true"
      aria-labelledby="variables-modal-title"
      @click.stop
    >
      <h3 id="variables-modal-title" class="modal-title">Prompt variables</h3>
      <p class="modal-hint">Fill in values for placeholders in the prompt content.</p>

      <label
        v-for="name in placeholders"
        :key="name"
        class="field"
        :data-testid="`prompt-variable-field-${name}`"
      >
        {{ name }}
        <input
          v-model="values[name]"
          type="text"
          :data-testid="`prompt-variable-input-${name}`"
          :aria-label="`Variable ${name}`"
        />
      </label>

      <label class="allow-empty">
        <input
          v-model="allowEmpty"
          type="checkbox"
          data-testid="prompt-variables-continue-empty"
        />
        Continue with empty values
      </label>

      <div class="actions">
        <button type="button" @click="emit('cancel')">Cancel</button>
        <button
          type="button"
          data-testid="prompt-variables-run"
          :disabled="runDisabled"
          @click="onConfirm"
        >
          Run
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
  margin: 0 0 0.5rem;
}

.modal-hint {
  margin: 0 0 1rem;
  color: var(--color-text-muted);
  font-size: 0.875rem;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  margin-bottom: 0.75rem;
  font-size: 0.8125rem;
  color: var(--color-text-muted);
}

.field input {
  padding: 0.4rem 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-surface);
  color: var(--color-text-primary);
}

.allow-empty {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 1rem;
  font-size: 0.8125rem;
  color: var(--color-text-primary);
  cursor: pointer;
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
}
</style>
