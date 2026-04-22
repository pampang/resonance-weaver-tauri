<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import IconButton from './IconButton.vue';

interface Config {
  kb_sources: string[];
  app_whitelist: string[];
  threshold: number;
}

interface IndexingProgress {
  total_files: number;
  current_file: number;
  file_name: string;
  percentage: number;
  is_complete: boolean;
}

const config = ref<Config>({
  kb_sources: [],
  app_whitelist: [],
  threshold: 0.7
});

const app_whitelist_str = ref('');
const running_apps = ref<string[]>([]);
const is_saving = ref(false);
const is_reindexing = ref(false);
const progress = ref<IndexingProgress | null>(null);

let unlistenProgress: UnlistenFn | null = null;

const loadConfig = async () => {
  const c = await invoke('get_config') as Config;
  config.value = c;
  app_whitelist_str.value = c.app_whitelist.join(', ');
};

const saveConfig = async () => {
  is_saving.value = true;
  config.value.app_whitelist = app_whitelist_str.value.split(',').map(s => s.trim()).filter(s => s !== '');
  try {
    await invoke('save_config', { config: config.value });
    alert('Configuration saved.');
  } finally {
    is_saving.value = false;
  }
};

const pickFolder = async () => {
  const selected = await open({
    directory: true,
    multiple: false,
  });
  if (selected && typeof selected === 'string') {
    config.value.kb_sources.push(selected);
  }
};

const removeSource = (index: number) => {
  config.value.kb_sources.splice(index, 1);
};

const triggerReindex = async () => {
  is_reindexing.value = true;
  progress.value = null;
  try {
    await invoke('reindex');
  } catch (e) {
    alert('Error: ' + e);
  }
};

const fetchRunningApps = async () => {
  try {
    running_apps.value = await invoke('get_running_apps');
  } catch (e) {
    console.error(e);
  }
};

const addToWhitelist = (appName: string) => {
  const current = app_whitelist_str.value.split(',').map(s => s.trim()).filter(s => s !== '');
  if (!current.includes(appName)) {
    current.push(appName);
    app_whitelist_str.value = current.join(', ');
  }
};

onMounted(async () => {
  loadConfig();
  fetchRunningApps();
  
  unlistenProgress = await listen<IndexingProgress>('indexing-progress', (event) => {
    progress.value = event.payload;
    if (event.payload.is_complete) {
      is_reindexing.value = false;
      setTimeout(() => { progress.value = null; }, 3000);
    }
  });
});

onUnmounted(() => {
  if (unlistenProgress) unlistenProgress();
});
</script>

<template>
  <div class="config-panel">
    <div class="header-actions">
      <h2>Configuration</h2>
      <button class="reindex-btn" :disabled="is_reindexing" @click="triggerReindex">
        {{ is_reindexing ? 'Indexing...' : 'Reindex All Now' }}
      </button>
    </div>

    <!-- Progress Overlay -->
    <div v-if="progress" class="progress-section">
      <div class="progress-info">
        <span>Indexing: {{ progress.file_name }}</span>
        <span>{{ progress.current_file }} / {{ progress.total_files }}</span>
      </div>
      <div class="progress-bar-bg">
        <div class="progress-bar-fill" :style="{ width: `${progress.percentage}%` }"></div>
      </div>
    </div>

    <div class="section">
      <h3>Knowledge Sources</h3>
      <div v-for="(source, index) in config.kb_sources" :key="index" class="input-group">
        <input :value="source" readonly />
        <IconButton 
          variant="danger" 
          size="md" 
          title="Remove Source" 
          @click="removeSource(index)" 
        />
      </div>
      <button class="add-btn" @click="pickFolder">+ Add Folder Source</button>
    </div>

    <div class="section">
      <h3>App Whitelist</h3>
      <p class="hint">Comma separated app names to monitor</p>
      <textarea v-model="app_whitelist_str" placeholder="e.g. Slack, Discord, Notes"></textarea>
      
      <div class="running-apps">
        <h4>Running Apps (Click to add):</h4>
        <div class="app-chips">
          <span v-for="app in running_apps" :key="app" class="app-chip" @click="addToWhitelist(app)">
            {{ app }}
          </span>
        </div>
      </div>
    </div>

    <div class="section">
      <h3>Resonance Threshold: {{ config.threshold }}</h3>
      <div class="threshold-control">
        <input type="range" v-model.number="config.threshold" min="0" max="1" step="0.05" />
        <span class="hint">Lower = stricter, Higher = more matches</span>
      </div>
    </div>

    <button class="save-btn" :disabled="is_saving" @click="saveConfig">
      {{ is_saving ? 'Saving...' : 'Save Configuration' }}
    </button>
  </div>
</template>

<style scoped>
.config-panel {
  padding: 24px;
  max-width: 700px;
  margin: 0 auto;
  overflow-y: auto;
}

.header-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.progress-section {
  background: #1e1e2e;
  padding: 16px;
  border-radius: 8px;
  margin-bottom: 24px;
  border: 1px solid #646cff44;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  font-size: 0.85rem;
  color: #ccc;
  margin-bottom: 8px;
}

.progress-bar-bg {
  height: 8px;
  background: #333;
  border-radius: 4px;
  overflow: hidden;
}

.progress-bar-fill {
  height: 100%;
  background: #646cff;
  transition: width 0.3s ease;
}

.section {
  background: #1a1a1a;
  padding: 20px;
  border-radius: 12px;
  margin-bottom: 24px;
  border: 1px solid #333;
}

h2, h3, h4 {
  margin-top: 0;
  color: #fff;
}

h3 {
  font-size: 1.1rem;
  margin-bottom: 12px;
}

.hint {
  font-size: 0.85rem;
  color: #888;
  margin-bottom: 12px;
}

.input-group {
  display: flex;
  margin-bottom: 8px;
  gap: 12px;
  align-items: center;
}

input[readonly] {
  flex-grow: 1;
  padding: 12px;
  background: #2a2a2a;
  border: 1px solid #444;
  color: #ccc;
  border-radius: 8px;
  font-size: 0.9rem;
}

.add-btn {
  width: 100%;
  padding: 12px;
  background: #2a2a2a;
  color: #646cff;
  border: 1px dashed #646cff;
  border-radius: 8px;
  cursor: pointer;
  margin-top: 8px;
  font-weight: 600;
  transition: all 0.2s;
}

.add-btn:hover {
  background: rgba(100, 108, 255, 0.1);
}

textarea {
  width: 100%;
  height: 100px;
  background: #2a2a2a;
  border: 1px solid #444;
  color: white;
  border-radius: 8px;
  padding: 12px;
  font-family: inherit;
  resize: vertical;
}

.running-apps {
  margin-top: 16px;
}

.app-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 8px;
}

.app-chip {
  background: #333;
  color: #aaa;
  padding: 6px 14px;
  border-radius: 20px;
  font-size: 0.8rem;
  cursor: pointer;
  transition: all 0.2s;
}

.app-chip:hover {
  background: #646cff;
  color: white;
}

.threshold-control {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

input[type="range"] {
  width: 100%;
}

.save-btn {
  width: 100%;
  padding: 16px;
  background: #646cff;
  color: white;
  border: none;
  border-radius: 12px;
  font-weight: 800;
  font-size: 1rem;
  cursor: pointer;
  box-shadow: 0 4px 15px rgba(100, 108, 255, 0.3);
  transition: all 0.2s;
}

.save-btn:hover:not(:disabled) {
  background: #535bf2;
  transform: translateY(-1px);
}

.save-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.reindex-btn {
  background: transparent;
  color: #646cff;
  border: 1px solid #646cff;
  padding: 8px 16px;
  border-radius: 8px;
  font-size: 0.85rem;
  font-weight: 600;
  cursor: pointer;
}

.reindex-btn:hover {
  background: rgba(100, 108, 255, 0.1);
}
</style>
