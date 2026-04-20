<script setup lang="ts">
import { ref } from "vue";
import TriageList from "./components/TriageList.vue";
import ConfigPanel from "./components/ConfigPanel.vue";

const currentTab = ref('triage');
</script>

<template>
  <main class="container">
    <nav class="tabs">
      <button :class="{ active: currentTab === 'triage' }" @click="currentTab = 'triage'">Triage Hub</button>
      <button :class="{ active: currentTab === 'config' }" @click="currentTab = 'config'">Configuration</button>
    </nav>

    <TriageList v-if="currentTab === 'triage'" />
    <ConfigPanel v-if="currentTab === 'config'" />
  </main>
</template>

<style scoped>
.container {
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  height: 100vh;
  background-color: #0f0f0f;
  color: white;
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
}

.tabs {
  display: flex;
  background: #1a1a1a;
  padding: 10px;
  gap: 10px;
  border-bottom: 1px solid #333;
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
