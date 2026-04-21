<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

interface Sample {
  id: number;
  content: string;
  matched_content: string | null;
  source_app: string;
  distance: number;
  created_at: string;
  expanded?: boolean;
}

const samples = ref<Sample[]>([]);
const is_loading = ref(true);

const fetchSamples = async () => {
  try {
    const data = await invoke('get_samples') as Sample[];
    samples.value = data.map(s => ({ ...s, expanded: false }));
  } catch (error) {
    console.error('Failed to fetch samples:', error);
  } finally {
    is_loading.value = false;
  }
};

const deleteSample = async (id: number) => {
  if (confirm('Delete this resonance?')) {
    await invoke('delete_sample', { id });
    await fetchSamples();
  }
};

const openDeepBridge = async (sample: Sample) => {
  try {
    await invoke('open_deep_bridge', { 
      content: sample.content, 
      matchedContent: sample.matched_content 
    });
  } catch (error) {
    console.error('Failed to open deep bridge:', error);
  }
};

const formatDate = (dateStr: string) => {
  try {
    const date = new Date(dateStr + 'Z');
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
    </div>

    <div v-else class="samples-list">
      <div v-for="sample in samples" :key="sample.id" class="sample-card">
        <div class="card-top">
          <div class="app-badge">{{ sample.source_app }}</div>
          <div class="resonance-score">
            <div class="score-label">{{ Math.max(0, (1 - sample.distance) * 100).toFixed(0) }}% Resonance</div>
            <div class="score-bar-bg">
              <div class="score-bar-fill" :style="{ width: `${Math.max(0, (1 - sample.distance) * 100)}%` }"></div>
            </div>
          </div>
          <button class="delete-btn" @click="deleteSample(sample.id)" title="Delete">✕</button>
        </div>
        
        <div class="content-sections" :class="{ 'is-expanded': sample.expanded }">
          <div class="content-box captured">
            <label>CAPTURED</label>
            <div class="text">{{ sample.content }}</div>
          </div>
          
          <div v-if="sample.matched_content" class="content-box matched">
            <label>MATCHED KNOWLEDGE</label>
            <div class="text">{{ sample.matched_content }}</div>
          </div>
        </div>

        <button class="expand-toggle" @click="sample.expanded = !sample.expanded">
          {{ sample.expanded ? 'Show Less ↑' : 'Read Full Context ↓' }}
        </button>

        <div class="card-bottom">
          <span class="timestamp">{{ formatDate(sample.created_at) }}</span>
          <button class="synthesis-btn" @click="openDeepBridge(sample)">
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

.samples-list {
  display: flex;
  flex-direction: column;
  gap: 20px;
  max-width: 900px;
  margin: 0 auto;
}

.sample-card {
  background: #1a1a1a;
  border: 1px solid #333;
  border-radius: 12px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  position: relative;
}

.delete-btn {
  background: transparent;
  border: none;
  color: #444;
  font-size: 1.2rem;
  cursor: pointer;
  padding: 4px 8px;
  margin-left: 12px;
}

.delete-btn:hover {
  color: #ff4d4d;
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
}

.resonance-score {
  flex-grow: 1;
  margin-left: 20px;
}

.score-label {
  font-size: 0.7rem;
  color: #666;
  margin-bottom: 4px;
}

.score-bar-bg {
  height: 4px;
  background: #333;
  border-radius: 2px;
}

.score-bar-fill {
  height: 100%;
  background: #646cff;
  border-radius: 2px;
}

.content-sections {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  margin-bottom: 12px;
}

.is-expanded .text {
  display: block !important;
  -webkit-line-clamp: unset !important;
  max-height: unset !important;
}

.content-box {
  padding: 12px;
  border-radius: 8px;
  font-size: 0.9rem;
  background: #222;
}

.matched {
  background: #1e251e;
  border-left: 3px solid #4CAF50;
}

.text {
  line-height: 1.5;
  color: #ccc;
  display: -webkit-box;
  -webkit-line-clamp: 5;
  -webkit-box-orient: vertical;
  overflow: hidden;
  max-height: 7.5em;
}

.expand-toggle {
  background: transparent;
  border: none;
  color: #646cff;
  font-size: 0.8rem;
  cursor: pointer;
  padding: 8px 0;
  text-align: center;
  width: 100%;
  border-bottom: 1px solid #2a2a2a;
  margin-bottom: 12px;
}

.card-bottom {
  display: flex;
  justify-content: space-between;
  align-items: center;
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
  font-weight: 600;
  cursor: pointer;
}
</style>
