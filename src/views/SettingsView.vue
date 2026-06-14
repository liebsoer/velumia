<script setup lang="ts">
import { onMounted, ref } from "vue";
import { api, type LangdockProfile } from "../lib/api";

const profiles = ref<LangdockProfile[]>([]);
const editingId = ref<string | null>(null);
const name = ref("");
const baseUrl = ref("");
const apiKey = ref("");
const message = ref("");

async function load() {
  profiles.value = await api.listProfiles();
  if (profiles.value.length && !editingId.value) {
    const p = profiles.value.find((x) => x.is_default) ?? profiles.value[0];
    editingId.value = p.id;
    name.value = p.name;
    baseUrl.value = p.base_url;
    apiKey.value = "";
  }
}

async function save() {
  message.value = "";
  const saved = await api.saveProfile(
    {
      name: name.value,
      base_url: baseUrl.value,
      api_key: apiKey.value || undefined,
      is_default: true,
    },
    editingId.value ?? undefined,
    !!apiKey.value || !editingId.value,
  );
  editingId.value = saved.id;
  message.value = "Profile saved.";
  await load();
}

async function testConnection() {
  if (!editingId.value) return;
  const updated = await api.testConnection(editingId.value);
  message.value = `Test complete: ${updated.connection_status}`;
  await load();
}

onMounted(load);
</script>

<template>
  <div class="settings">
    <h1>Settings</h1>
    <section>
      <h2>LangDock setup</h2>
      <label>Name<input v-model="name" /></label>
      <label>Base URL<input v-model="baseUrl" /></label>
      <label>API key (leave empty to keep existing)<input v-model="apiKey" type="password" placeholder="••••••••" /></label>
      <div class="actions">
        <button type="button" @click="save">Save</button>
        <button type="button" class="secondary" @click="testConnection">Test connection</button>
      </div>
      <p v-if="message" class="message">{{ message }}</p>
    </section>
  </div>
</template>

<style scoped>
.settings {
  flex: 1;
  padding: 2rem;
}

label {
  display: block;
  margin-bottom: 1rem;
}

input {
  display: block;
  width: 100%;
  max-width: 420px;
  margin-top: 0.35rem;
  padding: 0.5rem;
  border-radius: 6px;
  border: 1px solid var(--color-border);
  background: var(--color-surface);
  color: var(--color-text-primary);
}

.actions {
  display: flex;
  gap: 0.5rem;
}

button {
  padding: 0.5rem 1rem;
  border-radius: 6px;
  border: 1px solid var(--color-border);
  background: var(--color-accent-muted);
  color: var(--color-accent);
  cursor: pointer;
}

button.secondary {
  background: var(--color-surface-raised);
  color: var(--color-text-primary);
}

.message {
  margin-top: 1rem;
  color: var(--color-text-muted);
}
</style>
