<script setup lang="ts">
import { ref, onMounted } from "vue";
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import TriageList from "./components/TriageList.vue";
import ConfigPanel from "./components/ConfigPanel.vue";
import ResonanceBubble from "./components/ResonanceBubble.vue";

const windowLabel = getCurrentWebviewWindow().label;
const currentTab = ref('triage');
const isMain = windowLabel === 'main';

onMounted(() => {
  // Always let the root be transparent so rounded corners work on borderless windows
  document.documentElement.style.backgroundColor = 'transparent';
  document.body.style.backgroundColor = 'transparent';
});
</script>

<template>
  <div class="app-root" :class="[windowLabel]">
    <!-- Main Window Layout -->
    <main v-if="isMain" class="main-window">
      <nav class="tabs">
        <button :class="{ active: currentTab === 'triage' }" @click="currentTab = 'triage'">Triage Hub</button>
        <button :class="{ active: currentTab === 'config' }" @click="currentTab = 'config'">Configuration</button>
      </nav>

      <div class="main-body">
        <TriageList v-if="currentTab === 'triage'" />
        <ConfigPanel v-if="currentTab === 'config'" />
      </div>
    </main>

    <!-- Bubble Window Layout -->
    <ResonanceBubble v-if="!isMain" />
  </div>
</template>

<style>
/* Reset everything to be transparent at the OS level */
html, body, #app {
  margin: 0; padding: 0; width: 100%; height: 100%;
  background: transparent !important;
  overflow: hidden;
}
* { box-sizing: border-box; }
</style>

<style scoped>
.app-root { width: 100%; height: 100%; background: transparent; }

/* ONLY the Main window gets a solid background */
.main.app-root { background: #0f0f0f; color: white; }

.main-window { display: flex; flex-direction: column; height: 100vh; }
.main-body { flex-grow: 1; overflow: hidden; display: flex; flex-direction: column; }

.tabs { display: flex; background: #1a1a1a; padding: 10px; gap: 10px; border-bottom: 1px solid #333; flex-shrink: 0; }
.tabs button { background: transparent; color: #888; border: none; padding: 8px 16px; cursor: pointer; font-weight: bold; }
.tabs button.active { color: #646cff; border-bottom: 2px solid #646cff; }
</style>
