<script setup lang="ts">
import { computed, ref } from "vue";
import { api } from "../lib/api";

const emit = defineEmits<{ done: [] }>();

const step = ref(1);
const name = ref("Default");
const baseUrl = ref("https://api.langdock.com");
const apiKey = ref("");
const addSamples = ref(false);
const error = ref("");
const busy = ref(false);

const canAdvance = computed(() => name.value.trim() && baseUrl.value.trim() && apiKey.value.trim());

function formatError(e: unknown): string {
  if (e instanceof Error) return e.message;
  if (typeof e === "string") return e;
  return String(e);
}

function goToStep2() {
  error.value = "";
  if (!name.value.trim()) {
    error.value = "Profile name is required";
    return;
  }
  if (!baseUrl.value.trim()) {
    error.value = "Base URL is required";
    return;
  }
  if (!apiKey.value.trim()) {
    error.value = "API key is required";
    return;
  }
  step.value = 2;
}

async function skip() {
  if (busy.value) return;
  busy.value = true;
  error.value = "";
  try {
    await api.completeWizard(true);
    emit("done");
  } catch (e) {
    error.value = formatError(e);
  } finally {
    busy.value = false;
  }
}

async function complete() {
  if (busy.value) return;
  error.value = "";
  if (!apiKey.value.trim()) {
    error.value = "API key is required";
    step.value = 1;
    return;
  }
  busy.value = true;
  try {
    await api.saveProfile(
      {
        name: name.value.trim(),
        base_url: baseUrl.value.trim(),
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
    error.value = formatError(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <main class="wizard-page">
    <div class="wizard">
      <header class="wizard-header">
        <h1>LangDock setup</h1>
        <p class="lead">Connect Velumia to your LangDock workspace so you can run prompts and agents.</p>
      </header>

      <ol class="steps" aria-label="Setup progress">
        <li :class="{ active: step === 1, done: step > 1 }">
          <span class="step-num">1</span>
          <span class="step-label">Credentials</span>
        </li>
        <li class="step-sep" aria-hidden="true" />
        <li :class="{ active: step === 2 }">
          <span class="step-num">2</span>
          <span class="step-label">Starter library</span>
        </li>
      </ol>

      <div v-if="step === 1" class="panel">
        <p class="panel-intro">Add your LangDock API key. You can create multiple profiles later in Settings.</p>

        <label class="field">
          <span class="field-label">Name</span>
          <input v-model="name" type="text" autocomplete="off" placeholder="Default" />
        </label>

        <label class="field">
          <span class="field-label">Base URL</span>
          <input
            v-model="baseUrl"
            type="url"
            autocomplete="off"
            placeholder="https://api.langdock.com"
          />
          <span class="field-hint">Use the cloud default or your dedicated deployment URL.</span>
        </label>

        <label class="field">
          <span class="field-label">API key</span>
          <input
            v-model="apiKey"
            type="password"
            autocomplete="off"
            placeholder="Paste your LangDock API key"
          />
        </label>
      </div>

      <div v-else class="panel">
        <p class="panel-intro">
          Your profile <strong>{{ name }}</strong> will be saved and tested against
          <code>{{ baseUrl }}</code>.
        </p>

        <label class="sample-option">
          <input v-model="addSamples" type="checkbox" :disabled="busy" />
          <span class="sample-copy">
            <span class="sample-title">Add starter samples to the library</span>
            <span class="sample-desc">A few example prompts to explore Velumia — you can remove them anytime.</span>
          </span>
        </label>
      </div>

      <p v-if="error" class="error" role="alert">{{ error }}</p>
      <p v-if="busy" class="busy">Testing connection…</p>

      <div class="actions">
        <template v-if="step === 1">
          <button type="button" class="ghost" :disabled="busy" @click="skip">Skip for now</button>
          <button type="button" class="primary" :disabled="!canAdvance || busy" @click="goToStep2">
            Next
          </button>
        </template>
        <template v-else>
          <button type="button" class="ghost" :disabled="busy" @click="step = 1">Back</button>
          <button type="button" class="primary" :disabled="busy" @click="complete">
            {{ busy ? "Saving…" : "Finish" }}
          </button>
        </template>
      </div>
    </div>
  </main>
</template>

<style scoped>
.wizard-page {
  flex: 1;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding: 2.5rem 1.5rem;
}

.wizard {
  width: 100%;
  max-width: 520px;
  padding: 2rem;
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border);
  border-radius: 12px;
}

.wizard-header h1 {
  margin: 0 0 0.5rem;
  font-size: 1.5rem;
}

.lead {
  margin: 0;
  color: var(--color-text-muted);
  font-size: 0.9375rem;
  line-height: 1.5;
}

.steps {
  display: flex;
  align-items: center;
  gap: 0;
  margin: 1.75rem 0 1.5rem;
  padding: 0;
  list-style: none;
}

.steps li {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--color-text-muted);
  font-size: 0.875rem;
}

.steps li.active {
  color: var(--color-text-primary);
}

.steps li.done {
  color: var(--color-accent);
}

.step-num {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.5rem;
  height: 1.5rem;
  border-radius: 50%;
  border: 1px solid var(--color-border);
  font-size: 0.75rem;
  font-weight: 600;
}

.steps li.active .step-num,
.steps li.done .step-num {
  border-color: var(--color-accent);
  background: var(--color-accent-muted);
  color: var(--color-accent);
}

.step-sep {
  flex: 1;
  min-width: 1.5rem;
  height: 1px;
  margin: 0 0.75rem;
  background: var(--color-border);
}

.panel-intro {
  margin: 0 0 1.25rem;
  color: var(--color-text-muted);
  font-size: 0.9375rem;
  line-height: 1.5;
}

.panel-intro code {
  font-size: 0.8125rem;
  color: var(--color-text-primary);
}

.field {
  display: block;
  margin-bottom: 1.125rem;
}

.field-label {
  display: block;
  margin-bottom: 0.35rem;
  font-size: 0.875rem;
  font-weight: 500;
}

.field-hint {
  display: block;
  margin-top: 0.35rem;
  font-size: 0.8125rem;
  color: var(--color-text-muted);
}

.field input:not([type="checkbox"]) {
  display: block;
  width: 100%;
  padding: 0.55rem 0.75rem;
  border-radius: 8px;
  border: 1px solid var(--color-border);
  background: var(--color-surface);
  color: var(--color-text-primary);
  font: inherit;
}

.field input:not([type="checkbox"]):focus {
  outline: none;
  border-color: var(--color-accent);
  box-shadow: 0 0 0 2px var(--color-accent-muted);
}

.sample-option {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
  padding: 1rem;
  border-radius: 8px;
  border: 1px solid var(--color-border);
  background: var(--color-surface);
  cursor: pointer;
}

.sample-option:has(input:checked) {
  border-color: var(--color-accent);
  background: var(--color-accent-muted);
}

.sample-option input[type="checkbox"] {
  margin-top: 0.2rem;
  width: 1rem;
  height: 1rem;
  accent-color: var(--color-accent);
  cursor: pointer;
}

.sample-copy {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.sample-title {
  font-weight: 500;
}

.sample-desc {
  font-size: 0.875rem;
  color: var(--color-text-muted);
  line-height: 1.45;
}

.error {
  margin: 1rem 0 0;
  padding: 0.65rem 0.75rem;
  border-radius: 8px;
  background: rgba(248, 81, 73, 0.1);
  color: var(--color-error);
  font-size: 0.875rem;
}

.busy {
  margin: 1rem 0 0;
  color: var(--color-text-muted);
  font-size: 0.875rem;
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  margin-top: 1.5rem;
}

button {
  padding: 0.55rem 1.1rem;
  border-radius: 8px;
  border: 1px solid var(--color-border);
  background: var(--color-surface);
  color: var(--color-text-primary);
  font: inherit;
  cursor: pointer;
}

button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

button.primary {
  background: var(--color-accent-muted);
  border-color: var(--color-accent);
  color: var(--color-accent);
  font-weight: 500;
}

button.ghost {
  background: transparent;
  color: var(--color-text-muted);
}

button:not(:disabled):hover {
  filter: brightness(1.08);
}
</style>
