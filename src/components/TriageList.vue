<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

interface Sample {
  content: string;
  source_app: string;
  distance: number;
  created_at: string;
}

const samples = ref<Sample[]>([]);

const fetchSamples = async () => {
  try {
    samples.value = await invoke('get_samples');
  } catch (error) {
    console.error('Failed to fetch samples:', error);
  }
};

const openDeepBridge = async (content: string) => {
  try {
    await invoke('open_deep_bridge', { content });
  } catch (error) {
    console.error('Failed to open deep bridge:', error);
  }
};

onMounted(() => {
  fetchSamples();
  // Poll every 5 seconds for new resonances
  setInterval(fetchSamples, 5000);
});
</script>

<template>
  <div class="triage-list">
    <h2>Triage Hub</h2>
    <div v-if="samples.length === 0" class="empty">No resonances found yet.</div>
    <div v-for="sample in samples" :key="sample.created_at" class="sample-card">
      <div class="card-header">
        <span class="app-name">{{ sample.source_app }}</span>
        <span class="distance">Match: {{ (1 - sample.distance).toFixed(2) }}</span>
      </div>
      <p class="content">{{ sample.content.substring(0, 200) }}...</p>
      <div class="card-footer">
        <span class="time">{{ sample.created_at }}</span>
        <button @click="openDeepBridge(sample.content)">Deep Synthesis</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.triage-list {
  padding: 20px;
  max-width: 600px;
  margin: 0 auto;
}

.sample-card {
  background: #2a2a2a;
  border-radius: 8px;
  padding: 15px;
  margin-bottom: 15px;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

.card-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 10px;
}

.app-name {
  font-weight: bold;
  color: #646cff;
}

.distance {
  font-size: 0.9em;
  color: #888;
}

.content {
  font-size: 0.95em;
  line-height: 1.4;
  margin-bottom: 10px;
}

.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.time {
  font-size: 0.8em;
  color: #666;
}

button {
  background: #646cff;
  color: white;
  border: none;
  padding: 5px 12px;
  border-radius: 4px;
  cursor: pointer;
}

button:hover {
  background: #535bf2;
}

.empty {
  text-align: center;
  color: #666;
  margin-top: 50px;
}
</style>
