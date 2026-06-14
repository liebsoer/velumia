<script setup lang="ts">
import { onMounted, ref } from "vue";
import { api, type ConnectionWidgetState } from "../lib/api";
import ConnectionWidget from "../components/ConnectionWidget.vue";
import DegradedBanner from "../components/DegradedBanner.vue";

const emit = defineEmits<{ openSettings: [] }>();

const connection = ref<ConnectionWidgetState>({ status: "not_configured" });
const counts = ref({ prompts: 0, agents: 0, skills: 0 });
const runBlocked = ref(false);
const bannerDismissed = ref(false);
const newPromptTitle = ref("");

async function refresh() {
  connection.value = await api.getConnectionWidget();
  counts.value = await api.libraryCounts();
  runBlocked.value = !(await api.canRunPrompt());
}

async function createPrompt() {
  if (!newPromptTitle.value.trim()) return;
  await api.createPrompt(newPromptTitle.value.trim());
  newPromptTitle.value = "";
  await refresh();
}

async function tryRun() {
  if (await api.canRunPrompt()) {
    alert("Run would start (LangDock connected).");
  } else {
    alert("LangDock is not connected. Open LangDock setup to fix your connection.");
  }
}

onMounted(refresh);

defineExpose({ refresh });
</script>

<template>
  <DegradedBanner
    :visible="
      !bannerDismissed &&
      (connection.status === 'configuration_error' || connection.status === 'offline')
    "
    @dismiss="bannerDismissed = true"
  />
  <main class="dashboard">
    <header>
      <h1>Dashboard</h1>
      <button type="button" class="link" @click="emit('openSettings')">Settings</button>
    </header>

    <ConnectionWidget :state="connection" />

    <section class="card">
      <h2>Library</h2>
      <p>Prompts: {{ counts.prompts }} · Agents: {{ counts.agents }} · Skills: {{ counts.skills }}</p>
      <div class="row">
        <input v-model="newPromptTitle" placeholder="New prompt title" />
        <button type="button" @click="createPrompt">Create prompt</button>
      </div>
      <button type="button" class="secondary" @click="tryRun">Run prompt</button>
      <p v-if="runBlocked" class="hint">Runs blocked until LangDock is connected.</p>
    </section>
  </main>
</template>

<style scoped>
.dashboard {
  flex: 1;
  padding: 1.5rem 2rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

h1 {
  margin: 0;
}

.link {
  background: none;
  border: none;
  color: var(--color-accent);
  cursor: pointer;
}

.card {
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 1rem 1.25rem;
}

.row {
  display: flex;
  gap: 0.5rem;
  margin: 0.75rem 0;
}

input,
button {
  border-radius: 6px;
  border: 1px solid var(--color-border);
  padding: 0.5rem 0.75rem;
  background: var(--color-surface);
  color: var(--color-text-primary);
}

button.secondary {
  margin-top: 0.5rem;
}

.hint {
  color: var(--color-text-muted);
  font-size: 0.875rem;
}
</style>
