<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { marked } from 'marked';
import IconButton from './IconButton.vue';

interface Sample {
  id: number;
  content: string;
  matched_content: string | null;
  source_app: string;
  distance: number;
  created_at: string;
  metadata?: string;
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
  if (confirm('Delete this resonance from history?')) {
    try {
      await invoke('delete_sample', { id });
      samples.value = samples.value.filter(s => s.id !== id);
      if (selectedSample.value?.id === id) {
        selectedSample.value = null;
      }
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
    const now = new Date();
    if (date.toDateString() === now.toDateString()) {
      return date.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' });
    }
    return date.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
  } catch {
    return dateStr;
  }
};

const getScore = (distance: number) => Math.max(0, (1 - distance) * 100).toFixed(0);

const getResonanceClass = (distance: number) => {
  const score = 1 - distance;
  if (score > 0.9) return 'high';
  if (score > 0.75) return 'mid';
  return 'low';
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
  const match = metadata.match(/file:(.*?)\|/);
  return match ? match[1] : null;
};

onMounted(async () => {
  fetchSamples();
  
  // Real-time update when Funnel emits an event
  await listen('new-resonance', () => {
    fetchSamples(); // Fetch to get the new item with ID
  });

  // Background polling as fallback
  setInterval(fetchSamples, 10000);
});
</script>

<template>
  <div class="split-view-root">
    <!-- Toast -->
    <Transition name="fade">
      <div v-if="toastMsg" class="toast">{{ toastMsg }}</div>
    </Transition>

    <!-- LEFT PANE: Master List -->
    <div class="master-pane">
      <div class="pane-header">
        <div class="title-area">
          <h2>Triage Hub</h2>
          <span class="count">{{ samples.length }} captures</span>
        </div>
        <div class="actions">
          <button class="icon-btn" @click="invoke('ping_test')" title="Send Test Ping">⚡</button>
          <button class="icon-btn" @click="fetchSamples" title="Refresh">🔄</button>
        </div>
      </div>

      <div v-if="is_loading && samples.length === 0" class="list-state">
        <div class="loader"></div>
        <p>Loading...</p>
      </div>
      <div v-else-if="samples.length === 0" class="list-state">
        <p>No resonances recorded.</p>
        <span class="hint">Copy text to trigger.</span>
      </div>

      <div v-else class="list-content">
        <div 
          v-for="sample in samples" 
          :key="sample.id" 
          class="list-item"
          :class="{ 'selected': selectedSample?.id === sample.id }"
          @click="selectedSample = sample"
        >
          <div class="item-top">
            <span class="app-tag">{{ sample.source_app }}</span>
            <span class="score-tag" :class="getResonanceClass(sample.distance)">
              {{ getScore(sample.distance) }}% Match
            </span>
          </div>
          <div class="item-preview">{{ sample.content }}</div>
          <div class="item-bottom">
            <span class="time">{{ formatDate(sample.created_at) }}</span>
            <IconButton 
              size="sm" 
              variant="danger" 
              icon="🗑" 
              title="Delete" 
              @click.stop="deleteSample(sample.id)" 
            />
          </div>
        </div>
      </div>
    </div>

    <!-- RIGHT PANE: Detail View -->
    <div class="detail-pane">
      <div v-if="!selectedSample" class="detail-empty">
        <div class="empty-illustration">✧</div>
        <p>Select a resonance from the list to view its deep associations.</p>
      </div>
      
      <div v-else class="detail-content">
        <div class="detail-header">
          <div class="meta-info">
            <span class="score-badge">{{ getScore(selectedSample.distance) }}% Resonance</span>
            <span class="source-info" v-if="getSourceFile(selectedSample.metadata)">
              Found in: <strong>{{ getSourceFile(selectedSample.metadata) }}</strong>
            </span>
          </div>
          <button class="synth-btn" @click="openDeepBridge(selectedSample)">
            ✨ Synthesize in Gemini
          </button>
        </div>

        <div class="markdown-scroller">
          <div class="content-block captured-block">
            <div class="block-label">Captured from {{ selectedSample.source_app }}</div>
            <div class="markdown-view" v-html="renderMarkdown(selectedSample.content)"></div>
          </div>

          <div class="connection-visual">
            <div class="v-line"></div>
            <div class="v-dot"></div>
            <div class="v-line"></div>
          </div>

          <div class="content-block matched-block" v-if="selectedSample.matched_content">
            <div class="block-label">Associated Knowledge</div>
            <div class="markdown-view" v-html="renderMarkdown(selectedSample.matched_content)"></div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Split View Root */
.split-view-root {
  display: flex;
  width: 100%;
  height: 100%;
  background: #0f0f0f;
  overflow: hidden;
}

/* Master Pane (List) */
.master-pane {
  width: 380px;
  min-width: 300px;
  border-right: 1px solid #2a2a2a;
  display: flex;
  flex-direction: column;
  background: #141416;
  flex-shrink: 0;
}

.pane-header {
  padding: 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid #2a2a2a;
}

.title-area h2 { margin: 0; font-size: 1.2rem; font-weight: 800; color: #fff; }
.title-area .count { font-size: 0.75rem; color: #666; }

.actions { display: flex; gap: 8px; }
.icon-btn { background: transparent; border: 1px solid #333; color: #888; border-radius: 6px; cursor: pointer; padding: 4px 8px; }
.icon-btn:hover { background: rgba(255,255,255,0.05); color: #fff; }

.list-state { padding: 40px 20px; text-align: center; color: #666; }
.list-state .hint { font-size: 0.8rem; color: #444; }

.list-content {
  flex-grow: 1;
  overflow-y: auto;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

/* List Items */
.list-item {
  padding: 16px;
  background: rgba(255,255,255,0.02);
  border: 1px solid rgba(255,255,255,0.05);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.list-item:hover { background: rgba(255,255,255,0.05); border-color: rgba(255,255,255,0.1); }
.list-item.selected { background: rgba(100, 108, 255, 0.1); border-color: #646cff; }

.item-top { display: flex; justify-content: space-between; align-items: center; }
.app-tag { font-size: 0.75rem; font-weight: 700; color: #ccc; }
.score-tag { font-size: 0.7rem; font-weight: 800; padding: 2px 8px; border-radius: 10px; background: rgba(0,0,0,0.3); }
.score-tag.high { color: #b794ff; }
.score-tag.mid { color: #64d2ff; }
.score-tag.low { color: #888; }

.item-preview {
  font-size: 0.85rem; color: #999; line-height: 1.4;
  display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden;
}
.list-item.selected .item-preview { color: #ccc; }

.item-bottom { display: flex; justify-content: space-between; align-items: center; }
.time { font-size: 0.7rem; color: #555; }


/* Detail Pane (Content) */
.detail-pane {
  flex-grow: 1;
  display: flex;
  flex-direction: column;
  background: #0a0a0c;
  overflow: hidden;
}

.detail-empty {
  flex-grow: 1; display: flex; flex-direction: column; align-items: center; justify-content: center;
  color: #444; font-size: 0.9rem;
}
.empty-illustration { font-size: 3rem; margin-bottom: 16px; opacity: 0.5; color: #646cff; }

.detail-content {
  display: flex; flex-direction: column; height: 100%;
}

.detail-header {
  padding: 24px 40px;
  border-bottom: 1px solid #1a1a1a;
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: #0d0d10;
}

.meta-info { display: flex; align-items: center; gap: 16px; }
.score-badge { font-size: 1.2rem; font-weight: 800; color: #b794ff; font-family: 'JetBrains Mono', monospace; }
.source-info { font-size: 0.85rem; color: #888; background: rgba(255,255,255,0.05); padding: 4px 12px; border-radius: 6px; }
.source-info strong { color: #ccc; }

.synth-btn {
  background: #646cff; color: white; border: none; padding: 12px 24px;
  border-radius: 10px; font-weight: 800; cursor: pointer; transition: all 0.2s;
  box-shadow: 0 4px 15px rgba(100, 108, 255, 0.2);
}
.synth-btn:hover { background: #535bf2; transform: translateY(-1px); }

.markdown-scroller {
  flex-grow: 1;
  overflow-y: auto;
  padding: 40px;
  display: flex;
  flex-direction: column;
}

.content-block {
  background: #141416;
  border: 1px solid #222;
  border-radius: 16px;
  padding: 24px 32px;
}

.captured-block { border-left: 3px solid #646cff; }
.matched-block { border-left: 3px solid #4CAF50; background: #111411; }

.block-label {
  font-size: 0.75rem; font-weight: 900; color: #666; text-transform: uppercase;
  letter-spacing: 1px; margin-bottom: 16px;
}
.captured-block .block-label { color: #646cff; }
.matched-block .block-label { color: #4CAF50; }

/* Markdown Styles */
.markdown-view { color: #ccc; line-height: 1.7; font-size: 1.05rem; }
:deep(.markdown-view p) { margin-top: 0; }
:deep(.markdown-view h1), :deep(.markdown-view h2), :deep(.markdown-view h3) { color: #fff; margin-top: 1.5em; border-bottom: 1px solid #222; padding-bottom: 8px; }
:deep(.markdown-view code) { background: rgba(255, 255, 255, 0.1); color: #fff; padding: 2px 6px; border-radius: 4px; font-family: 'JetBrains Mono', monospace; font-size: 0.9em; }
:deep(.markdown-view pre) { background: #000; padding: 20px; border-radius: 12px; overflow-x: auto; border: 1px solid #222; margin: 16px 0; }
:deep(.markdown-view blockquote) { border-left: 4px solid #4CAF50; padding-left: 20px; color: #999; background: rgba(76, 175, 80, 0.05); padding: 12px 20px; border-radius: 0 8px 8px 0; }

/* Visual Connection */
.connection-visual { display: flex; flex-direction: column; align-items: center; height: 60px; }
.v-line { width: 2px; flex-grow: 1; background: #222; }
.v-dot { width: 8px; height: 8px; background: #444; border-radius: 50%; border: 2px solid #222; }

/* Toast */
.toast {
  position: fixed; bottom: 30px; right: 30px;
  background: #4CAF50; color: white; padding: 12px 24px;
  border-radius: 8px; font-weight: 700; z-index: 3000;
  box-shadow: 0 10px 30px rgba(0,0,0,0.4);
}

.fade-enter-active, .fade-leave-active { transition: opacity 0.3s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
