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
const is_loading = ref(true);

const fetchSamples = async () => {
  try {
    samples.value = await invoke('get_samples');
  } catch (error) {
    console.error('Failed to fetch samples:', error);
  } finally {
    is_loading.value = false;
  }
};

const openDeepBridge = async (content: string) => {
  try {
    await invoke('open_deep_bridge', { content });
  } catch (error) {
    console.error('Failed to open deep bridge:', error);
  }
};

const formatDate = (dateStr: string) => {
  try {
    const date = new Date(dateStr + 'Z'); // Assume UTC from SQLite
    return date.toLocaleString();
  } catch {
    return dateStr;
  }
};

onMounted(() => {
  fetchSamples();
  setInterval(fetchSamples, 5000);
});
</script>

<template>
  <div class="triage-hub">
    <div class="header">
      <h2>Triage Hub</h2>
      <button class="refresh-btn" @click="fetchSamples">Refresh</button>
    </div>

    <div v-if="is_loading && samples.length === 0" class="state-msg">Loading resonances...</div>
    <div v-else-if="samples.length === 0" class="state-msg">
      <div class="empty-icon">📡</div>
      <p>No resonances detected yet.</p>
      <p class="sub">Monitor whitelisted apps and copy text to see them here.</p>
    </div>

    <div v-else class="samples-grid">
      <div v-for="sample in samples" :key="sample.created_at" class="sample-card">
        <div class="card-top">
          <div class="app-badge">{{ sample.source_app }}</div>
          <div class="resonance-score">
            <div class="score-label">Resonance</div>
            <div class="score-bar-bg">
              <div class="score-bar-fill" :style="{ width: `${Math.max(0, (1 - sample.distance) * 100)}%` }"></div>
            </div>
          </div>
        </div>
        
        <div class="content-preview">
          {{ sample.content }}
        </div>

        <div class="card-bottom">
          <span class="timestamp">{{ formatDate(sample.created_at) }}</span>
          <button class="synthesis-btn" @click="openDeepBridge(sample.content)">
            Deep Synthesis
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.triage-hub {
  padding: 24px;
  flex-grow: 1;
  overflow-y: auto;
  background: #0f0f0f;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

h2 {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 700;
}

.refresh-btn {
  background: transparent;
  border: 1px solid #333;
  color: #888;
  padding: 6px 12px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.85rem;
}

.refresh-btn:hover {
  border-color: #646cff;
  color: #646cff;
}

.state-msg {
  text-align: center;
  margin-top: 80px;
  color: #666;
}

.empty-icon {
  font-size: 3rem;
  margin-bottom: 16px;
}

.sub {
  font-size: 0.9rem;
  max-width: 300px;
  margin: 8px auto;
}

.samples-grid {
  display: grid;
  gap: 20px;
  grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
}

.sample-card {
  background: #1a1a1a;
  border: 1px solid #333;
  border-radius: 12px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  transition: transform 0.2s, border-color 0.2s;
}

.sample-card:hover {
  border-color: #444;
  transform: translateY(-2px);
}

.card-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.app-badge {
  background: #646cff22;
  color: #747bff;
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.resonance-score {
  width: 100px;
}

.score-label {
  font-size: 0.7rem;
  color: #666;
  text-align: right;
  margin-bottom: 4px;
}

.score-bar-bg {
  height: 6px;
  background: #333;
  border-radius: 3px;
  overflow: hidden;
}

.score-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, #646cff, #9089ff);
  border-radius: 3px;
}

.content-preview {
  font-size: 0.95rem;
  line-height: 1.5;
  color: #ccc;
  margin-bottom: 20px;
  display: -webkit-box;
  -webkit-line-clamp: 4;
  -webkit-box-orient: vertical;
  overflow: hidden;
  flex-grow: 1;
}

.card-bottom {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 16px;
  border-top: 1px solid #2a2a2a;
}

.timestamp {
  font-size: 0.75rem;
  color: #666;
}

.synthesis-btn {
  background: #646cff;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 0.85rem;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.2s;
}

.synthesis-btn:hover {
  background: #535bf2;
}
</style>
