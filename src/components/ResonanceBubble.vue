<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

const appWebviewWindow = getCurrentWebviewWindow();

interface ResonancePayload {
  app_name: string;
  score: number;
  content: string;
}

const payload = ref<ResonancePayload | null>(null);
const visible = ref(false);

const closeBubble = async () => {
  visible.ref = false;
  await appWebviewWindow.hide();
};

const openHub = async () => {
  await invoke('show_main_window');
  await closeBubble();
};

onMounted(async () => {
  await listen<ResonancePayload>('new-resonance', (event) => {
    payload.value = event.payload;
    visible.value = true;
    
    // Auto hide after 8 seconds
    setTimeout(() => {
      closeBubble();
    }, 8000);
  });
});
</script>

<template>
  <div v-if="payload" class="bubble-container" @click="openHub">
    <div class="bubble-content">
      <div class="header">
        <span class="app-tag">{{ payload.app_name }}</span>
        <span class="score-tag">{{ (payload.score * 100).toFixed(0) }}% Resonance</span>
      </div>
      <p class="preview">{{ payload.content.substring(0, 80) }}...</p>
      <div class="action-hint">Click to Triage ↗</div>
    </div>
    <button class="close-x" @click.stop="closeBubble">✕</button>
  </div>
</template>

<style scoped>
.bubble-container {
  width: 100%;
  height: 100%;
  background: rgba(30, 30, 46, 0.95);
  backdrop-filter: blur(10px);
  border: 1px solid #646cff;
  border-radius: 16px;
  padding: 12px 16px;
  color: white;
  display: flex;
  flex-direction: column;
  justify-content: center;
  position: relative;
  cursor: pointer;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  -webkit-app-region: drag;
  animation: slideIn 0.3s ease-out;
}

@keyframes slideIn {
  from { transform: translateX(100%); opacity: 0; }
  to { transform: translateX(0); opacity: 1; }
}

.bubble-content {
  -webkit-app-region: no-drag;
}

.header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
}

.app-tag {
  font-weight: bold;
  color: #646cff;
  font-size: 0.85rem;
}

.score-tag {
  font-size: 0.75rem;
  background: #646cff44;
  padding: 2px 8px;
  border-radius: 10px;
}

.preview {
  margin: 0;
  font-size: 0.85rem;
  line-height: 1.4;
  color: #ccc;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.action-hint {
  font-size: 0.7rem;
  color: #666;
  margin-top: 6px;
  text-align: right;
}

.close-x {
  position: absolute;
  top: 8px;
  right: 8px;
  background: transparent;
  border: none;
  color: #555;
  cursor: pointer;
  -webkit-app-region: no-drag;
}

.close-x:hover {
  color: white;
}
</style>
