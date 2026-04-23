<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { marked } from 'marked';
import IconButton from './IconButton.vue';

interface Sample {
  id: number;
  content: string;
  matched_content: string | null;
  source_app: string;
  distance: number;
  created_at: string;
  metadata?: string; // Add metadata field
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
    return date.toLocaleString(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
  } catch {
    return dateStr;
  }
};

const renderMarkdown = (text: string | null): string => {
  if (!text) return '';
  try {
    return marked.parse(text) as string;
  } catch (e) {
    return text;
  }
};

const getSourceFile = (metadata?: string) => {
  if (!metadata) return null;
  // Format is "file:Filename.md|part:X"
  const match = metadata.match(/file:(.*?)\|/);
  return match ? match[1] : null;
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
      <div class="header-btns">
        <button class="debug-btn" @click="invoke('ping_test')">Ping Test Event</button>
        <button class="refresh-btn" @click="fetchSamples">
          <span class="icon">🔄</span> Refresh
        </button>
      </div>
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

          <IconButton 
            variant="danger" 
            size="md" 
            title="Delete Resonance" 
            @click="deleteSample(sample.id)" 
          />
        </div>
        
        <div class="card-content-area" @click="selectedSample = sample">
          <div class="content-block captured">
            <label>Captured</label>
            <div class="preview-wrap">
              <div class="markdown-content" v-html="renderMarkdown(sample.content)"></div>
              <div class="fade-overlay"></div>
            </div>
          </div>

          <div class="connector-v">
            <div class="dot"></div>
          </div>

          <div class="content-block matched" v-if="sample.matched_content">
            <div class="matched-header">
              <label>Knowledge Match</label>
              <span v-if="getSourceFile(sample.metadata)" class="source-tag">
                📄 {{ getSourceFile(sample.metadata) }}
              </span>
            </div>
            <div class="preview-wrap">
              <div class="markdown-content" v-html="renderMarkdown(sample.matched_content)"></div>
              <div class="fade-overlay"></div>
            </div>
          </div>
          <div class="expand-hint">Click card to open full immersive view</div>
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
            <div class="modal-title">
              <h3>Resonance Analysis</h3>
              <span class="modal-source" v-if="getSourceFile(selectedSample.metadata)">
                Source: {{ getSourceFile(selectedSample.metadata) }}
              </span>
            </div>
            <IconButton size="lg" @click="selectedSample = null" />
          </div>
          <div class="modal-body">
            <div class="modal-pane">
              <label>Captured Content</label>
              <div class="markdown-view" v-html="renderMarkdown(selectedSample.content)"></div>
            </div>
            
            <div class="modal-divider">
              <div class="line"></div>
              <div class="score-node">{{ ((1-selectedSample.distance)*100).toFixed(0) }}% Similarity</div>
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

.header-btns {
  display: flex;
  gap: 12px;
}

.debug-btn {
  background: rgba(255, 255, 0, 0.1);
  border: 1px solid rgba(255, 255, 0, 0.3);
  color: #ffcc00;
  padding: 8px 16px;
  border-radius: 8px;
  cursor: pointer;
  font-size: 0.8rem;
  font-weight: bold;
}

.toast {
  position: fixed; top: 24px; left: 50%; transform: translateX(-50%);
  background: #646cff; color: white; padding: 12px 32px;
  border-radius: 50px; font-weight: 700; z-index: 3000;
  box-shadow: 0 10px 30px rgba(100, 108, 255, 0.4);
}

.samples-list {
  display: flex; flex-direction: column; gap: 40px;
  max-width: 800px; margin: 0 auto;
}

.sample-card {
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 24px; padding: 28px;
  transition: all 0.3s;
}

.sample-card:hover { 
  border-color: rgba(100, 108, 255, 0.4);
  background: rgba(255, 255, 255, 0.03);
}

.card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px; }
.app-dot { width: 10px; height: 10px; background: #646cff; border-radius: 50%; box-shadow: 0 0 10px #646cff; }
.app-name { font-weight: 700; font-size: 1rem; margin-left: 10px; color: #eee; }

.resonance-score {
  font-family: 'JetBrains Mono', monospace; font-weight: 800; color: #b794ff; font-size: 0.9rem;
  background: rgba(183, 148, 255, 0.1); padding: 4px 12px; border-radius: 20px;
}

.card-content-area { cursor: pointer; display: flex; flex-direction: column; }

.content-block {
  display: flex; flex-direction: column; gap: 8px;
}

.content-block label {
  font-size: 0.65rem; font-weight: 900; text-transform: uppercase; color: #444; letter-spacing: 1px;
}

.matched-header {
  display: flex; justify-content: space-between; align-items: center;
}

.source-tag {
  font-size: 0.7rem; color: #646cff; background: rgba(100, 108, 255, 0.1);
  padding: 2px 8px; border-radius: 4px; font-weight: 600;
}

.preview-wrap {
  position: relative; max-height: 140px; overflow: hidden;
  border-radius: 12px; background: rgba(255, 255, 255, 0.015);
  padding: 12px;
}

.fade-overlay {
  position: absolute; bottom: 0; left: 0; width: 100%; height: 40px;
  background: linear-gradient(to top, #141414, transparent);
}

.markdown-content { font-size: 0.95rem; color: #bbb; line-height: 1.5; }

.connector-v {
  height: 30px; display: flex; align-items: center; justify-content: center;
}
.connector-v .dot {
  width: 4px; height: 4px; background: #333; border-radius: 50%;
}

.expand-hint { font-size: 0.7rem; color: #333; text-align: center; margin-top: 12px; font-style: italic; }

.card-footer {
  margin-top: 24px; padding-top: 20px; border-top: 1px solid rgba(255, 255, 255, 0.05);
  display: flex; justify-content: space-between; align-items: center;
}
.time { font-size: 0.75rem; color: #444; }

.synthesis-btn {
  background: #646cff; color: white; border: none;
  padding: 10px 24px; border-radius: 10px; font-weight: 700; cursor: pointer;
  box-shadow: 0 4px 15px rgba(100, 108, 255, 0.2);
}

/* Modal */
.modal-overlay {
  position: fixed; top: 0; left: 0; width: 100vw; height: 100vh;
  background: rgba(0, 0, 0, 0.9); backdrop-filter: blur(16px);
  display: flex; justify-content: center; align-items: center; z-index: 2000;
  padding: 40px;
}

.modal-container {
  background: #0f0f12; width: 100%; max-width: 1000px; height: 90vh;
  border-radius: 28px; border: 1px solid #222; display: flex; flex-direction: column;
  overflow: hidden; box-shadow: 0 30px 80px rgba(0,0,0,0.8);
}

.modal-header {
  padding: 28px 40px; border-bottom: 1px solid #222;
  display: flex; justify-content: space-between; align-items: center;
}

.modal-title h3 { margin: 0; font-size: 1.5rem; color: #fff; }
.modal-source { font-size: 0.8rem; color: #646cff; margin-top: 4px; display: block; font-weight: 600; }

.modal-body { flex-grow: 1; overflow-y: auto; padding: 40px; display: flex; flex-direction: column; gap: 40px; }
.modal-pane label { color: #646cff; font-weight: 800; font-size: 0.75rem; text-transform: uppercase; display: block; margin-bottom: 20px; letter-spacing: 1px; }

.markdown-view { color: #ccc; line-height: 1.7; font-size: 1.05rem; }

/* Base Markdown Styles */
:deep(.markdown-view h1), :deep(.markdown-view h2) { color: #fff; margin-top: 1.2em; font-size: 1.2rem; border-bottom: 1px solid #333; padding-bottom: 8px; }
:deep(.markdown-view code) { background: rgba(100, 108, 255, 0.2); color: #b794ff; padding: 2px 6px; border-radius: 4px; font-family: 'JetBrains Mono', monospace; }
:deep(.markdown-view pre) { background: #000; padding: 20px; border-radius: 12px; overflow-x: auto; border: 1px solid #222; margin: 16px 0; }
:deep(.markdown-view blockquote) { border-left: 4px solid #646cff; padding-left: 20px; color: #999; background: rgba(255,255,255,0.02); padding: 12px 20px; border-radius: 0 8px 8px 0; }

.modal-divider { display: flex; align-items: center; gap: 20px; }
.modal-divider .line { flex-grow: 1; height: 1px; background: #222; }
.score-node { 
  background: #1a1a2a; border: 1px solid #646cff; color: #646cff;
  padding: 6px 18px; border-radius: 30px; font-weight: 800; font-size: 1rem; 
}

.modal-footer { padding: 32px 40px; border-top: 1px solid #222; text-align: center; background: rgba(0,0,0,0.2); }
.synthesis-btn-large {
  background: #646cff; color: white; border: none; padding: 18px 80px;
  border-radius: 20px; font-weight: 800; font-size: 1.2rem; cursor: pointer;
  transition: all 0.3s;
}
.synthesis-btn-large:hover { transform: scale(1.02); box-shadow: 0 8px 25px rgba(100, 108, 255, 0.4); }

/* Transitions */
.modal-enter-active, .modal-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.modal-enter-from, .modal-leave-to { opacity: 0; transform: translateY(40px) scale(0.95); }

.state-container { text-align: center; margin-top: 120px; color: #444; }
.empty-icon { font-size: 4rem; opacity: 0.1; }
</style>
