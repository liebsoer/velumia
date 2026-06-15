<script setup lang="ts">
import { ref } from "vue";
import { api } from "../lib/api";

const emit = defineEmits<{ done: [] }>();

const step = ref(1);
const name = ref("Default");
const baseUrl = ref("https://api.langdock.com");
const apiKey = ref("");
const addSamples = ref(false);
const error = ref("");

async function skip() {
  await api.completeWizard(true);
  emit("done");
}

async function complete() {
  error.value = "";
  if (!apiKey.value.trim()) {
    error.value = "API key is required";
    return;
  }
  try {
    await api.saveProfile(
      {
        name: name.value,
        base_url: baseUrl.value,
        api_key: apiKey.value,
        is_default: true,
      },
      undefined,
      true,
    );
    if (addSamples.value) {
      await api.seedSamples();
    }
    await api.completeWizard(false);
    emit("done");
  } catch (e) {
    error.value = String(e);
  }
}
</script>

<template>
  <div class="wizard">
    <h1>LangDock setup</h1>
    <p class="subtitle">Step {{ step }} of 2</p>

    <div v-if="step === 1" class="panel">
      <label>Name<input v-model="name" /></label>
      <label>Base URL<input v-model="baseUrl" /></label>
      <label>API key<input v-model="apiKey" type="password" /></label>
      <p v-if="error" class="error">{{ error }}</p>
      <div class="actions">
        <button type="button" class="ghost" @click="skip">Skip</button>
        <button type="button" @click="step = 2">Next</button>
      </div>
    </div>

    <div v-else class="panel">
      <label class="checkbox">
        <input v-model="addSamples" type="checkbox" />
        Add starter samples to the library
      </label>
      <div class="actions">
        <button type="button" class="ghost" @click="step = 1">Back</button>
        <button type="button" @click="complete">Finish</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.wizard {
  max-width: 480px;
  margin: 4rem auto;
  padding: 2rem;
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border);
  border-radius: 12px;
}

.subtitle {
  color: var(--color-text-muted);
}

label {
  display: block;
  margin-bottom: 1rem;
}

input[type="text"],
input[type="password"] {
  display: block;
  width: 100%;
  margin-top: 0.35rem;
  padding: 0.5rem;
  border-radius: 6px;
  border: 1px solid var(--color-border);
  background: var(--color-surface);
  color: var(--color-text-primary);
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  margin-top: 1.5rem;
}

button {
  padding: 0.5rem 1rem;
  border-radius: 6px;
  border: 1px solid var(--color-border);
  background: var(--color-accent-muted);
  color: var(--color-accent);
  cursor: pointer;
}

button.ghost {
  background: transparent;
  color: var(--color-text-muted);
}

.error {
  color: var(--color-error);
}

.checkbox {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
</style>
