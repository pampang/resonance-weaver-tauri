<script setup lang="ts">
import { ref } from "vue";
import TriageList from "./components/TriageList.vue";
import ConfigPanel from "./components/ConfigPanel.vue";

const currentTab = ref('triage');
</script>

<template>
  <div class="app-root">
    <div class="brand-line"></div>
    <main class="main-window">
      <nav class="tabs">
        <button :class="{ active: currentTab === 'triage' }" @click="currentTab = 'triage'">Triage Hub</button>
        <button :class="{ active: currentTab === 'config' }" @click="currentTab = 'config'">Configuration</button>
      </nav>
      <div class="main-body">
        <Transition name="tab-fade" mode="out-in">
          <TriageList v-if="currentTab === 'triage'" key="triage" />
          <ConfigPanel v-else-if="currentTab === 'config'" key="config" />
        </Transition>
      </div>
    </main>
  </div>
</template>

<style>
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800;900&display=swap');

/* Reset */
html, body, #app {
  margin: 0; padding: 0; width: 100%; height: 100%;
  font-family: 'Inter', system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}
* { box-sizing: border-box; }
</style>

<style scoped>
.app-root { width: 100%; height: 100%; background: #0f0f0f; color: white; display: flex; flex-direction: column; }

.brand-line {
  height: 2px;
  flex-shrink: 0;
  background: linear-gradient(90deg, #646cff, #b794ff);
}

.main-window { display: flex; flex-direction: column; flex: 1; min-height: 0; }
.main-body { flex-grow: 1; overflow: hidden; display: flex; flex-direction: column; }

.tabs {
  display: flex;
  background: #1a1a1a;
  padding: 10px 16px;
  gap: 6px;
  border-bottom: 1px solid #2a2a2a;
  flex-shrink: 0;
}

.tabs button {
  background: transparent;
  color: #888;
  border: none;
  padding: 12px 20px;
  cursor: pointer;
  font-weight: 700;
  font-size: 0.9rem;
  border-radius: 8px;
  transition: color 0.2s, background 0.2s;
  position: relative;
}

.tabs button:hover {
  color: #ccc;
  background: rgba(255, 255, 255, 0.05);
}

.tabs button.active {
  color: #b794ff;
  background: rgba(100, 108, 255, 0.08);
}

.tabs button.active::after {
  content: '';
  position: absolute;
  bottom: -1px;
  left: 20%;
  right: 20%;
  height: 2px;
  background: linear-gradient(90deg, #646cff, #b794ff);
  border-radius: 2px;
}

/* Tab transition */
.tab-fade-enter-active,
.tab-fade-leave-active {
  transition: opacity 0.15s ease;
}

.tab-fade-enter-from,
.tab-fade-leave-to {
  opacity: 0;
}
</style>
