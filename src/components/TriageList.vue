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
const selectedSample = ref<Sample | null>(null); // For Modal

const fetchSamples = async () => {
  try {
    const data = await invoke('get_samples') as Sample[];
    
    // Smart merge to avoid UI jump and state reset
    if (samples.value.length === 0) {
      samples.value = data;
    } else {
      // Add new samples to the beginning
      const existingIds = new Set(samples.value.map(s => s.id));
      const newItems = data.filter(s => !existingIds.has(s.id));
      if (newItems.length > 0) {
        samples.value = [...newItems, ...samples.value];
      }
      // Remove deleted samples
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
    await invoke('delete_sample', { id });
    samples.value = samples.value.filter(s => s.id !== id);
    if (selectedSample.value?.id === id) selectedSample.value = null;
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

const getResonanceLabel = (distance: number) => {
  const score = 1 - distance;
  if (score > 0.9) return { text: 'Core Resonance', class: 'high' };
  if (score > 0.75) return { text: 'Strong Link', class: 'mid' };
  return { text: 'Semantic Echo', class: 'low' };
};

const renderMarkdown = (text: string | null) => {
  if (!text) return '';
  return marked.parse(text);
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
    </div>

    <div v-else class="samples-stack">
      <div v-for="sample in samples" :key="sample.id" class="sample-card">
        <div class="card-header">
          <div class="app-info">
            <span class="app-dot"></span>
            <span class="app-name">{{ sample.source_app }}</span>
          </div>
          
          <div class="resonance-meta" :class="getResonanceLabel(sample.distance).class">
            <span class="percent">{{ Math.max(0, (1 - sample.distance) * 100).toFixed(0) }}%</span>
          </div>

          <button class="delete-icon" @click="deleteSample(sample.id)">✕</button>
        </div>
        
        <div class="content-preview-flow" @click="selectedSample = sample">
          <div class="preview-box captured">
            <div class="text" v-html="renderMarkdown(sample.content.substring(0, 300) + (sample.content.length > 300 ? '...' : ''))"></div>
          </div>
          <div class="preview-box matched" v-if="sample.matched_content">
            <div class="text" v-html="renderMarkdown(sample.matched_content.substring(0, 300) + (sample.matched_content.length > 300 ? '...' : ''))"></div>
          </div>
          <div class="click-hint">Click to read full Markdown</div>
        </div>

        <div class="card-footer">
          <span class="time">{{ formatDate(sample.created_at) }}</span>
          <button class="synthesis-btn" @click="openDeepBridge(sample)">
            Synthesize
          </button>
        </div>
      </div>
    </div>

    <!-- Modal Dialog -->
    <Transition name="modal">
      <div v-if="selectedSample" class="modal-overlay" @click.self="selectedSample = null">
        <div class="modal-content">
          <div class="modal-header">
            <h3>Resonance Detail</h3>
            <button class="close-btn" @click="selectedSample = null">Close</button>
          </div>
          <div class="modal-body">
            <div class="modal-section">
              <label>Captured from {{ selectedSample.source_app }}</label>
              <div class="markdown-body" v-html="renderMarkdown(selectedSample.content)"></div>
            </div>
            <div class="modal-connector">
              <div class="line"></div>
              <div class="node">{{ ( (1-selectedSample.distance)*100 ).toFixed(0) }}%</div>
              <div class="line"></div>
            </div>
            <div class="modal-section matched" v-if="selectedSample.matched_content">
              <label>Matched from Knowledge Base</label>
              <div class="markdown-body" v-html="renderMarkdown(selectedSample.matched_content)"></div>
            </div>
          </div>
          <div class="modal-footer">
            <button class="synthesis-btn large" @click="openDeepBridge(selectedSample)">
              Synthesize with Gemini
            </button>
          </div>
        </div>
      </div>
    </Transition>
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

h2 { margin: 0; font-size: 1.8rem; font-weight: 800; color: #fff; }

.count-badge { font-size: 0.8rem; color: #666; }

.refresh-btn {
  background: #1a1a1a; border: 1px solid #333; color: #aaa;
  padding: 8px 16px; border-radius: 8px; cursor: pointer;
}

/* Toast */
.toast {
  position: fixed; top: 20px; left: 50%; transform: translateX(-50%);
  background: #646cff; color: white; padding: 12px 24px;
  border-radius: 50px; font-weight: 600; z-index: 1000;
}

.samples-stack {
  display: flex; flex-direction: column; gap: 32px;
  max-width: 800px; margin: 0 auto;
}

.sample-card {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 20px; padding: 24px; transition: border-color 0.3s;
}

.sample-card:hover { border-color: rgba(100, 108, 255, 0.3); }

.card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }

.app-info { display: flex; align-items: center; gap: 8px; }
.app-dot { width: 8px; height: 8px; background: #646cff; border-radius: 50%; box-shadow: 0 0 10px #646cff; }
.app-name { font-weight: 700; color: #fff; font-size: 0.9rem; }

.resonance-meta {
  background: rgba(0, 0, 0, 0.3); padding: 2px 10px; border-radius: 50px;
  font-family: 'JetBrains Mono', monospace; font-weight: 800; font-size: 0.8rem;
}
.resonance-meta.high { color: #b794ff; }
.resonance-meta.mid { color: #64d2ff; }
.resonance-meta.low { color: #888; }

.delete-icon { background: transparent; border: none; color: #333; cursor: pointer; }
.delete-icon:hover { color: #ff4d4d; }

.content-preview-flow {
  cursor: pointer; display: flex; flex-direction: column; gap: 8px;
  position: relative;
}

.preview-box {
  padding: 12px; border-radius: 10px; background: rgba(255, 255, 255, 0.02);
  font-size: 0.9rem; max-height: 120px; overflow: hidden;
  position: relative;
}

.preview-box.matched { border-left: 2px solid rgba(76, 175, 80, 0.3); }

.click-hint {
  font-size: 0.7rem; color: #444; text-align: center; margin-top: 4px;
}

.card-footer {
  margin-top: 16px; display: flex; justify-content: space-between; align-items: center;
}

.time { font-size: 0.75rem; color: #444; }

.synthesis-btn {
  background: #646cff; color: white; border: none;
  padding: 6px 16px; border-radius: 6px; font-weight: 700; font-size: 0.8rem;
}

/* Modal Styling */
.modal-overlay {
  position: fixed; top: 0; left: 0; width: 100vw; height: 100vh;
  background: rgba(0, 0, 0, 0.85); backdrop-filter: blur(10px);
  display: flex; justify-content: center; align-items: center; z-index: 2000;
  padding: 40px;
}

.modal-content {
  background: #1a1a1a; width: 100%; max-width: 1000px; height: 90vh;
  border-radius: 24px; border: 1px solid #333; display: flex; flex-direction: column;
  overflow: hidden; box-shadow: 0 20px 60px rgba(0,0,0,0.5);
}

.modal-header {
  padding: 20px 32px; border-bottom: 1px solid #333;
  display: flex; justify-content: space-between; align-items: center;
}

.modal-body {
  flex-grow: 1; overflow-y: auto; padding: 32px;
  display: flex; flex-direction: column; gap: 24px;
}

.modal-section label {
  display: block; font-size: 0.7rem; font-weight: 800; color: #646cff;
  text-transform: uppercase; margin-bottom: 12px; letter-spacing: 1px;
}

.markdown-body {
  color: #ccc; line-height: 1.6; font-size: 1rem;
}

/* Base Markdown Styles */
:deep(.markdown-body h1), :deep(.markdown-body h2) { color: #fff; margin-top: 1.5em; }
:deep(.markdown-body code) { background: #333; padding: 2px 6px; border-radius: 4px; font-family: monospace; }
:deep(.markdown-body pre) { background: #000; padding: 16px; border-radius: 12px; overflow-x: auto; border: 1px solid #222; }
:deep(.markdown-body blockquote) { border-left: 4px solid #646cff; padding-left: 16px; color: #888; }

.modal-connector {
  display: flex; align-items: center; gap: 16px; color: #444;
}
.modal-connector .line { flex-grow: 1; height: 1px; background: #333; }
.modal-connector .node {
  padding: 4px 12px; border-radius: 20px; border: 1px solid #333;
  font-size: 0.8rem; font-weight: 800; color: #646cff;
}

.modal-footer {
  padding: 24px 32px; border-top: 1px solid #333; text-align: center;
}

.synthesis-btn.large { padding: 12px 40px; font-size: 1rem; border-radius: 12px; }

.close-btn {
  background: transparent; border: 1px solid #333; color: #888;
  padding: 6px 12px; border-radius: 6px; cursor: pointer;
}

/* Transitions */
.modal-enter-active, .modal-leave-active { transition: all 0.3s ease; }
.modal-enter-from, .modal-leave-to { opacity: 0; transform: scale(0.95); }

.fade-enter-active, .fade-leave-active { transition: opacity 0.5s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

.state-msg { text-align: center; margin-top: 100px; color: #444; }
.empty-icon { font-size: 4rem; opacity: 0.2; }
</style>
