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
const toastMsg = ref('');

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
  await invoke('delete_sample', { id });
  await fetchSamples();
};

const openDeepBridge = async (sample: Sample) => {
  try {
    const msg = await invoke('open_deep_bridge', { 
      content: sample.content, 
      matchedContent: sample.matched_content 
    }) as string;
    showToast(msg);
  } catch (error) {
    console.error('Failed to open deep bridge:', error);
  }
};

const showToast = (msg: string) => {
  toastMsg.value = msg;
  setTimeout(() => { toastMsg.value = ''; }, 4000);
};

const formatDate = (dateStr: string) => {
  try {
    const date = new Date(dateStr + 'Z');
    return date.toLocaleString(undefined, { 
      month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' 
    });
  } catch {
    return dateStr;
  }
};

// Map score to a human-friendly resonance level
const getResonanceLabel = (distance: number) => {
  const score = 1 - distance;
  if (score > 0.9) return { text: 'Core Resonance', class: 'high' };
  if (score > 0.75) return { text: 'Strong Link', class: 'mid' };
  return { text: 'Semantic Echo', class: 'low' };
};

onMounted(() => {
  fetchSamples();
  setInterval(fetchSamples, 5000);
});
</script>

<template>
  <div class="triage-hub">
    <div class="header">
      <div class="title-group">
        <h2>Triage Hub</h2>
        <span class="count-badge">{{ samples.length }} Resonances</span>
      </div>
      <button class="refresh-btn" @click="fetchSamples">
        <span class="icon">🔄</span> Refresh
      </button>
    </div>

    <!-- Toast Notification -->
    <Transition name="fade">
      <div v-if="toastMsg" class="toast">{{ toastMsg }}</div>
    </Transition>

    <div v-if="is_loading && samples.length === 0" class="state-msg">
      <div class="loader"></div>
      <p>Synchronizing with Knowledge Base...</p>
    </div>
    
    <div v-else-if="samples.length === 0" class="state-msg">
      <div class="empty-icon">📡</div>
      <p>Your library is silent.</p>
      <p class="sub">Copy text from whitelisted apps to trigger a resonance.</p>
    </div>

    <div v-else class="samples-stack">
      <div v-for="sample in samples" :key="sample.id" class="sample-card">
        <div class="card-header">
          <div class="app-info">
            <span class="app-dot"></span>
            <span class="app-name">{{ sample.source_app }}</span>
          </div>
          
          <div class="resonance-meta" :class="getResonanceLabel(sample.distance).class">
            <span class="label">{{ getResonanceLabel(sample.distance).text }}</span>
            <span class="percent">{{ ((1 - sample.distance) * 100).toFixed(0) }}%</span>
          </div>

          <button class="delete-icon" @click="deleteSample(sample.id)">✕</button>
        </div>
        
        <div class="content-flow">
          <!-- Captured Section -->
          <div class="box captured">
            <div class="box-header">Captured Fragment</div>
            <div class="text" :class="{ 'expanded': sample.expanded }">{{ sample.content }}</div>
          </div>

          <!-- Connecting Icon -->
          <div class="flow-connector">
            <div class="line"></div>
            <div class="pulse-node"></div>
            <div class="line"></div>
          </div>
          
          <!-- Matched Knowledge Section -->
          <div v-if="sample.matched_content" class="box matched">
            <div class="box-header">Associated Knowledge</div>
            <div class="text" :class="{ 'expanded': sample.expanded }">{{ sample.matched_content }}</div>
          </div>
        </div>

        <div class="card-footer">
          <span class="time">{{ formatDate(sample.created_at) }}</span>
          <div class="actions">
            <button class="expand-btn" @click="sample.expanded = !sample.expanded">
              {{ sample.expanded ? 'Show Less' : 'Full Text' }}
            </button>
            <button class="synthesis-btn" @click="openDeepBridge(sample)">
              Synthesize with Gemini
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.triage-hub {
  padding: 32px;
  flex-grow: 1;
  overflow-y: auto;
  background: radial-gradient(circle at top right, #161625, #0f0f0f);
  position: relative;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  margin-bottom: 32px;
  max-width: 800px;
  margin-left: auto;
  margin-right: auto;
}

h2 {
  margin: 0;
  font-size: 1.8rem;
  font-weight: 800;
  background: linear-gradient(to right, #fff, #888);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.count-badge {
  font-size: 0.8rem;
  color: #666;
  font-weight: 500;
}

.refresh-btn {
  background: #1a1a1a;
  border: 1px solid #333;
  color: #aaa;
  padding: 8px 16px;
  border-radius: 8px;
  cursor: pointer;
  font-size: 0.85rem;
  transition: all 0.2s;
}

.refresh-btn:hover {
  border-color: #646cff;
  color: #fff;
}

/* Toast Styling */
.toast {
  position: fixed;
  top: 20px;
  left: 50%;
  transform: translateX(-50%);
  background: #646cff;
  color: white;
  padding: 12px 24px;
  border-radius: 50px;
  font-weight: 600;
  box-shadow: 0 10px 25px rgba(100, 108, 255, 0.4);
  z-index: 1000;
}

.samples-stack {
  display: flex;
  flex-direction: column;
  gap: 32px;
  max-width: 800px;
  margin: 0 auto;
}

.sample-card {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 20px;
  padding: 24px;
  position: relative;
  transition: border-color 0.3s, transform 0.3s;
}

.sample-card:hover {
  border-color: rgba(100, 108, 255, 0.3);
  background: rgba(255, 255, 255, 0.04);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.app-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.app-dot {
  width: 8px;
  height: 8px;
  background: #646cff;
  border-radius: 50%;
  box-shadow: 0 0 10px #646cff;
}

.app-name {
  font-weight: 700;
  color: #fff;
  font-size: 0.9rem;
}

.resonance-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  background: rgba(0, 0, 0, 0.3);
  padding: 4px 12px;
  border-radius: 50px;
}

.resonance-meta.high { color: #b794ff; }
.resonance-meta.mid { color: #64d2ff; }
.resonance-meta.low { color: #888; }

.resonance-meta .percent {
  font-family: 'JetBrains Mono', monospace;
  font-weight: 800;
}

.delete-icon {
  background: transparent;
  border: none;
  color: #333;
  cursor: pointer;
  font-size: 1.1rem;
}

.delete-icon:hover { color: #ff4d4d; }

.content-flow {
  display: flex;
  flex-direction: column;
}

.box {
  padding: 16px;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.02);
}

.box-header {
  font-size: 0.65rem;
  font-weight: 800;
  text-transform: uppercase;
  color: #555;
  margin-bottom: 8px;
  letter-spacing: 1px;
}

.text {
  font-size: 1rem;
  line-height: 1.6;
  color: #ccc;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.text.expanded {
  display: block;
  -webkit-line-clamp: unset;
}

.flow-connector {
  display: flex;
  flex-direction: column;
  align-items: center;
  height: 40px;
}

.line {
  width: 1px;
  flex-grow: 1;
  background: rgba(255, 255, 255, 0.1);
}

.pulse-node {
  width: 6px;
  height: 6px;
  background: #646cff;
  border-radius: 50%;
  margin: 4px 0;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0% { transform: scale(1); opacity: 1; }
  50% { transform: scale(1.5); opacity: 0.5; }
  100% { transform: scale(1); opacity: 1; }
}

.matched {
  background: rgba(76, 175, 80, 0.05);
  border-left: 2px solid rgba(76, 175, 80, 0.3);
}

.card-footer {
  margin-top: 24px;
  padding-top: 20px;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.time {
  font-size: 0.75rem;
  color: #444;
}

.actions {
  display: flex;
  gap: 12px;
}

.expand-btn {
  background: transparent;
  border: 1px solid #333;
  color: #666;
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 0.8rem;
  cursor: pointer;
}

.synthesis-btn {
  background: #646cff;
  color: white;
  border: none;
  padding: 8px 20px;
  border-radius: 8px;
  font-weight: 700;
  font-size: 0.85rem;
  cursor: pointer;
  box-shadow: 0 4px 15px rgba(100, 108, 255, 0.2);
}

.synthesis-btn:hover {
  background: #535bf2;
  transform: translateY(-1px);
}

/* Transitions */
.fade-enter-active, .fade-leave-active { transition: opacity 0.5s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

.state-msg {
  text-align: center;
  margin-top: 100px;
  color: #444;
}

.empty-icon { font-size: 4rem; margin-bottom: 20px; opacity: 0.2; }
</style>
