<script setup lang="ts">
import { onMounted, ref } from "vue";
import { api } from "./lib/api";
import DashboardView from "./views/DashboardView.vue";
import WizardView from "./views/WizardView.vue";
import SettingsView from "./views/SettingsView.vue";
import PromptLibraryView from "./views/PromptLibraryView.vue";

type View = "wizard" | "dashboard" | "prompts" | "settings";

const view = ref<View>("dashboard");
const showWizard = ref(false);
const version = ref("");
const startPromptCreate = ref(false);

const dashboardRef = ref<InstanceType<typeof DashboardView> | null>(null);
const promptsRef = ref<InstanceType<typeof PromptLibraryView> | null>(null);

async function init() {
  version.value = await api.ping();
  showWizard.value = await api.isFirstLaunch();
  view.value = showWizard.value ? "wizard" : "dashboard";
}

async function onWizardDone() {
  showWizard.value = false;
  view.value = "dashboard";
  await dashboardRef.value?.refresh();
}

function openSettings() {
  view.value = "settings";
}

function backToDashboard() {
  view.value = "dashboard";
  dashboardRef.value?.refresh();
}

function openPrompts(create = false) {
  startPromptCreate.value = create;
  view.value = "prompts";
}

async function onPromptsNav() {
  startPromptCreate.value = false;
  view.value = "prompts";
  await promptsRef.value?.refresh();
}

onMounted(init);
</script>

<template>
  <div class="layout">
    <aside class="sidebar">
      <div class="brand">Velumia</div>
      <nav>
        <button
          type="button"
          class="nav-item"
          :class="{ active: view === 'dashboard' }"
          @click="view = 'dashboard'"
        >
          Dashboard
        </button>
        <button
          type="button"
          class="nav-item"
          :class="{ active: view === 'prompts' }"
          @click="onPromptsNav"
        >
          Prompts
        </button>
        <button type="button" class="nav-item muted" disabled>Agents</button>
        <button type="button" class="nav-item muted" disabled>Skills</button>
        <button
          type="button"
          class="nav-item"
          :class="{ active: view === 'settings' }"
          @click="openSettings"
        >
          Settings
        </button>
      </nav>
      <p class="version">v{{ version }}</p>
    </aside>

    <WizardView v-if="view === 'wizard'" @done="onWizardDone" />
    <DashboardView
      v-else-if="view === 'dashboard'"
      ref="dashboardRef"
      @open-settings="openSettings"
      @open-prompts="openPrompts(true)"
    />
    <PromptLibraryView
      v-else-if="view === 'prompts'"
      ref="promptsRef"
      :start-create="startPromptCreate"
    />
    <div v-else class="settings-wrap">
      <button type="button" class="back" @click="backToDashboard">← Dashboard</button>
      <SettingsView />
    </div>
  </div>
</template>

<style scoped>
.layout {
  display: flex;
  min-height: 100vh;
  width: 100%;
}

.layout > :not(.sidebar) {
  flex: 1;
  min-width: 0;
}

.sidebar {
  width: 220px;
  padding: 1.5rem 1rem;
  background: var(--color-surface-raised);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
}

.brand {
  font-weight: 700;
  font-size: 1.25rem;
  margin-bottom: 2rem;
}

.nav-item {
  display: block;
  width: 100%;
  text-align: left;
  padding: 0.5rem 0.75rem;
  margin-bottom: 0.25rem;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: var(--color-text-primary);
  cursor: pointer;
}

.nav-item.active {
  background: var(--color-accent-muted);
  color: var(--color-accent);
}

.nav-item.muted {
  color: var(--color-text-muted);
  cursor: default;
}

.version {
  margin-top: auto;
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.settings-wrap {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.back {
  align-self: flex-start;
  margin: 1rem 2rem 0;
  background: none;
  border: none;
  color: var(--color-accent);
  cursor: pointer;
}
</style>
