<script setup lang="ts">
import { ref } from "vue";
import TriageList from "./components/TriageList.vue";
import ConfigPanel from "./components/ConfigPanel.vue";

const currentTab = ref('triage');
</script>

<template>
  <div class="app-root">
    <main class="main-window">
      <nav class="tabs">
        <button :class="{ active: currentTab === 'triage' }" @click="currentTab = 'triage'">Triage Hub</button>
        <button :class="{ active: currentTab === 'config' }" @click="currentTab = 'config'">Configuration</button>
      </nav>
      <div class="main-body">
        <TriageList v-if="currentTab === 'triage'" />
        <ConfigPanel v-if="currentTab === 'config'" />
      </div>
    </main>
  </div>
</template>

<style>
/* Reset */
html, body, #app {
  margin: 0; padding: 0; width: 100%; height: 100%;
}
* { box-sizing: border-box; }
</style>

<style scoped>
.app-root { width: 100%; height: 100%; background: #0f0f0f; color: white; }

.main-window { display: flex; flex-direction: column; height: 100vh; }
.main-body { flex-grow: 1; overflow: hidden; display: flex; flex-direction: column; }

.tabs { display: flex; background: #1a1a1a; padding: 10px; gap: 10px; border-bottom: 1px solid #333; }
.tabs button { background: transparent; color: #888; border: none; padding: 8px 16px; cursor: pointer; font-weight: bold; transition: color 0.2s; }
.tabs button:hover { color: #ccc; }
.tabs button.active { color: #646cff; border-bottom: 2px solid #646cff; }
</style>
