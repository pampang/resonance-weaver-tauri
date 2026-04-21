<script setup lang="ts">
import { ref } from "vue";
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import TriageList from "./components/TriageList.vue";
import ConfigPanel from "./components/ConfigPanel.vue";
import ResonanceBubble from "./components/ResonanceBubble.vue";

const windowLabel = getCurrentWebviewWindow().label;
const currentTab = ref('triage');

const isMain = windowLabel === 'main';
const isBubble = windowLabel === 'resonance-bubble';
</script>

<template>
  <div class="app-root" :class="{ 'bubble-mode': isBubble }">
    <!-- Main Window Layout -->
    <main v-if="isMain" class="container">
      <nav class="tabs">
        <button :class="{ active: currentTab === 'triage' }" @click="currentTab = 'triage'">Triage Hub</button>
        <button :class="{ active: currentTab === 'config' }" @click="currentTab = 'config'">Configuration</button>
      </nav>

      <TriageList v-if="currentTab === 'triage'" />
      <ConfigPanel v-if="currentTab === 'config'" />
    </main>

    <!-- Bubble Window Layout -->
    <ResonanceBubble v-if="isBubble" />
  </div>
</template>

<style>
* {
  box-sizing: border-box;
}

body {
  margin: 0;
  padding: 0;
  background-color: transparent; /* Essential for transparent windows */
}
</style>

<style scoped>
.app-root {
  width: 100%;
  height: 100%;
}

.bubble-mode {
  background: transparent;
}

.container {
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  background-color: #0f0f0f;
  color: white;
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  overflow: hidden;
}

.tabs {
  display: flex;
  background: #1a1a1a;
  padding: 10px;
  gap: 10px;
  border-bottom: 1px solid #333;
  flex-shrink: 0;
}

.tabs button {
  background: transparent;
  color: #888;
  border: none;
  padding: 8px 16px;
  cursor: pointer;
  font-weight: bold;
}

.tabs button.active {
  color: #646cff;
  border-bottom: 2px solid #646cff;
}
</style>
