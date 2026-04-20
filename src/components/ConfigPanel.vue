<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

interface Config {
  kb_sources: string[];
  app_whitelist: string[];
  threshold: number;
}

const config = ref<Config>({
  kb_sources: [],
  app_whitelist: [],
  threshold: 0.7
});

const app_whitelist_str = ref('');

const loadConfig = async () => {
  const c = await invoke('get_config') as Config;
  config.value = c;
  app_whitelist_str.value = c.app_whitelist.join(', ');
};

const saveConfig = async () => {
  config.value.app_whitelist = app_whitelist_str.value.split(',').map(s => s.trim()).filter(s => s !== '');
  await invoke('save_config', { config: config.value });
};

const addSource = () => {
  config.value.kb_sources.push('');
};

const removeSource = (index: number) => {
  config.value.kb_sources.splice(index, 1);
};

onMounted(loadConfig);
</script>

<template>
  <div class="config-panel">
    <h2>Configuration</h2>
    <div class="section">
      <h3>Knowledge Sources</h3>
      <div v-for="(_, index) in config.kb_sources" :key="index" class="input-group">
        <input v-model="config.kb_sources[index]" placeholder="/path/to/folder" />
        <button @click="removeSource(index)">Remove</button>
      </div>
      <button @click="addSource">Add Source</button>
    </div>

    <div class="section">
      <h3>App Whitelist</h3>
      <p>Comma separated apps</p>
      <textarea v-model="app_whitelist_str"></textarea>
    </div>

    <div class="section">
      <h3>Threshold: {{ config.threshold }}</h3>
      <input type="range" v-model.number="config.threshold" min="0" max="1" step="0.05" />
    </div>

    <button class="save-btn" @click="saveConfig">Save Configuration</button>
  </div>
</template>

<style scoped>
.config-panel {
  padding: 20px;
  max-width: 600px;
  margin: 0 auto;
}

.section {
  margin-bottom: 25px;
}

.input-group {
  display: flex;
  margin-bottom: 10px;
}

input[type="text"], input:not([type]) {
  flex-grow: 1;
  padding: 8px;
  background: #1a1a1a;
  border: 1px solid #333;
  color: white;
  border-radius: 4px;
}

textarea {
  width: 100%;
  height: 80px;
  background: #1a1a1a;
  border: 1px solid #333;
  color: white;
  border-radius: 4px;
  padding: 8px;
}

.save-btn {
  width: 100%;
  padding: 12px;
  background: #4CAF50;
  font-weight: bold;
}

.save-btn:hover {
  background: #45a049;
}
</style>
