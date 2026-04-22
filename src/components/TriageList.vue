<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { marked } from 'marked';

interface Sample {
  id: number;
  content: string;
  matched_content: string | null;
  source_app: string;
  distance: number;
  created_at: string;
}

const samples = ref<Sample[]>([]);
const is_loading = ref(true);
const toastMsg = ref('');
const selectedSample = ref<Sample | null>(null);

const fetchSamples = async () => {
  try {
    const data = await invoke('get_samples') as Sample[];
    if (samples.value.length === 0) {
      samples.value = data;
    } else {
      const existingIds = new Set(samples.value.map(s => s.id));
      const newItems = data.filter(s => !existingIds.has(s.id));
      if (newItems.length > 0) {
        samples.value = [...newItems, ...samples.value];
      }
      const currentIds = new Set(data.map(s => s.id));
      samples.value = samples.value.filter(s => currentIds.has(s.id));
    }
  } catch (error) {
    console.error('Failed to fetch samples:', error);
  } finally {
    is_loading.value = false;
  }
};

const deleteSample = async (id: number) => {
  if (confirm('Delete this resonance?')) {
    try {
      await invoke('delete_sample', { id });
      samples.value = samples.value.filter(s => s.id !== id);
      if (selectedSample.value?.id === id) selectedSample.value = null;
    } catch (e) {
      console.error(e);
    }
  }
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
    showToast('Failed to synthesize.');
  }
};

const showToast = (msg: string) => {
  toastMsg.value = msg;
  setTimeout(() => { toastMsg.value = ''; }, 4000);
};

const formatDate = (dateStr: string) => {
  try {
    const date = new Date(dateStr + 'Z');
    return date.toLocaleString();
  } catch {
    return dateStr;
  }
};

const renderMarkdown = (text: string | null): string => {
  if (!text) return '';
  try {
    // marked.parse can be synchronous or asynchronous depending on configuration.
    // In current version it is sync by default.
    return marked.parse(text) as string;
  } catch (e) {
    console.error('Markdown parse error:', e);
    return text; // Fallback to raw text
  }
};

onMounted(() => {
  fetchSamples();
  setInterval(fetchSamples, 5000);
});
</script>

<template>
  <div class="triage-hub-root">
    <div class="header">
      <div class="title-group">
        <h2>Triage Hub</h2>
        <span class="count-badge">{{ samples.length }} Resonances</span>
      </div>
      <button class="refresh-btn" @click="fetchSamples">
        <span class="icon">🔄</span> Refresh
      </button>
    </div>

    <!-- Toast -->
    <Transition name="fade">
      <div v-if="toastMsg" class="toast">{{ toastMsg }}</div>
    </Transition>

    <!-- State Messages -->
    <div v-if="is_loading && samples.length === 0" class="state-container">
      <div class="loader"></div>
      <p>Syncing Knowledge Base...</p>
    </div>
    
    <div v-else-if="samples.length === 0" class="state-container">
      <div class="empty-icon">📡</div>
      <p>Silence in the library.</p>
    </div>

    <!-- Main List -->
    <div v-else class="samples-list">
      <div v-for="sample in samples" :key="sample.id" class="sample-card">
        <div class="card-header">
          <div class="app-info">
            <span class="app-dot"></span>
            <span class="app-name">{{ sample.source_app }}</span>
          </div>
          
          <div class="resonance-score">
            {{ Math.max(0, (1 - sample.distance) * 100).toFixed(0) }}% Resonance
          </div>

          <button class="delete-icon" @click="deleteSample(sample.id)" title="Delete">✕</button>
        </div>
        
        <div class="card-preview-area" @click="selectedSample = sample">
          <div class="preview-box captured" v-html="renderMarkdown(sample.content.substring(0, 200) + '...')"></div>
          <div class="preview-box matched" v-if="sample.matched_content" v-html="renderMarkdown(sample.matched_content.substring(0, 200) + '...')"></div>
          <div class="expand-hint">Click to read full association ↘</div>
        </div>

        <div class="card-footer">
          <span class="time">{{ formatDate(sample.created_at) }}</span>
          <button class="synthesis-btn" @click="openDeepBridge(sample)">Synthesize</button>
        </div>
      </div>
    </div>

    <!-- Modal Dialog -->
    <Transition name="modal">
      <div v-if="selectedSample" class="modal-overlay" @click.self="selectedSample = null">
        <div class="modal-container">
          <div class="modal-header">
            <h3>Resonance Analysis</h3>
            <button class="modal-close-btn" @click="selectedSample = null">✕</button>
          </div>
          <div class="modal-body">
            <div class="modal-pane">
              <label>Captured Content</label>
              <div class="markdown-view" v-html="renderMarkdown(selectedSample.content)"></div>
            </div>
            
            <div class="modal-divider">
              <div class="line"></div>
              <div class="score-node">{{ ((1-selectedSample.distance)*100).toFixed(0) }}%</div>
              <div class="line"></div>
            </div>

            <div class="modal-pane association" v-if="selectedSample.matched_content">
              <label>Associated Knowledge</label>
              <div class="markdown-view" v-html="renderMarkdown(selectedSample.matched_content)"></div>
            </div>
          </div>
          <div class="modal-footer">
            <button class="synthesis-btn-large" @click="openDeepBridge(selectedSample)">
              ✨ Synthesize with Gemini
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.triage-hub-root {
  padding: 32px;
  background: transparent;
  flex-grow: 1;
  overflow-y: auto;
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

h2 { margin: 0; font-size: 1.8rem; font-weight: 800; color: #fff; }
.count-badge { font-size: 0.8rem; color: #555; }

.refresh-btn {
  background: #1a1a1a; border: 1px solid #333; color: #888;
  padding: 8px 16px; border-radius: 8px; cursor: pointer;
}

.toast {
  position: fixed; top: 24px; left: 50%; transform: translateX(-50%);
  background: #646cff; color: white; padding: 12px 32px;
  border-radius: 50px; font-weight: 700; z-index: 3000;
  box-shadow: 0 10px 30px rgba(100, 108, 255, 0.4);
}

.samples-list {
  display: flex; flex-direction: column; gap: 32px;
  max-width: 800px; margin: 0 auto;
}

.sample-card {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 20px; padding: 24px;
}

.sample-card:hover { border-color: rgba(100, 108, 255, 0.3); }

.card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
.app-dot { width: 8px; height: 8px; background: #646cff; border-radius: 50%; box-shadow: 0 0 10px #646cff; }
.app-name { font-weight: 700; font-size: 0.9rem; margin-left: 8px; }

.resonance-score {
  font-family: 'JetBrains Mono', monospace; font-weight: 800; color: #b794ff; font-size: 0.85rem;
}

.delete-icon {
  background: rgba(255, 255, 255, 0.05); border: none; color: #555;
  width: 32px; height: 32px; border-radius: 50%; cursor: pointer;
}
.delete-icon:hover { background: rgba(255, 77, 77, 0.2); color: #ff4d4d; }

.card-preview-area { cursor: pointer; display: flex; flex-direction: column; gap: 10px; }
.preview-box {
  padding: 12px; border-radius: 10px; background: rgba(255, 255, 255, 0.02);
  font-size: 0.9rem; color: #aaa; max-height: 100px; overflow: hidden;
}
.preview-box.matched { border-left: 3px solid #4CAF5022; }

.expand-hint { font-size: 0.7rem; color: #444; text-align: right; }

.card-footer {
  margin-top: 20px; padding-top: 16px; border-top: 1px solid rgba(255, 255, 255, 0.05);
  display: flex; justify-content: space-between; align-items: center;
}
.time { font-size: 0.75rem; color: #444; }

.synthesis-btn {
  background: #646cff; color: white; border: none;
  padding: 8px 20px; border-radius: 8px; font-weight: 700; cursor: pointer;
}

/* Modal */
.modal-overlay {
  position: fixed; top: 0; left: 0; width: 100vw; height: 100vh;
  background: rgba(0, 0, 0, 0.9); backdrop-filter: blur(12px);
  display: flex; justify-content: center; align-items: center; z-index: 2000;
  padding: 40px;
}

.modal-container {
  background: #16161a; width: 100%; max-width: 1000px; height: 90vh;
  border-radius: 24px; border: 1px solid #333; display: flex; flex-direction: column;
  overflow: hidden;
}

.modal-header {
  padding: 24px 32px; border-bottom: 1px solid #333;
  display: flex; justify-content: space-between; align-items: center;
}
.modal-close-btn {
  background: #333; border: none; color: #fff; width: 40px; height: 40px; border-radius: 50%; cursor: pointer;
}

.modal-body { flex-grow: 1; overflow-y: auto; padding: 32px; display: flex; flex-direction: column; gap: 32px; }
.modal-pane label { color: #646cff; font-weight: 800; font-size: 0.7rem; text-transform: uppercase; display: block; margin-bottom: 16px; }

.markdown-view { color: #ccc; line-height: 1.6; }

.modal-divider { display: flex; align-items: center; gap: 16px; }
.modal-divider .line { flex-grow: 1; height: 1px; background: #222; }
.score-node { color: #646cff; font-weight: 800; font-size: 0.9rem; }

.modal-footer { padding: 32px; border-top: 1px solid #333; text-align: center; }
.synthesis-btn-large {
  background: #646cff; color: white; border: none; padding: 16px 64px;
  border-radius: 16px; font-weight: 800; font-size: 1.1rem; cursor: pointer;
}

/* Transitions */
.modal-enter-active, .modal-leave-active { transition: all 0.3s ease; }
.modal-enter-from, .modal-leave-to { opacity: 0; transform: scale(0.98); }

.state-container { text-align: center; margin-top: 120px; color: #444; }
.empty-icon { font-size: 4rem; opacity: 0.2; }
</style>
