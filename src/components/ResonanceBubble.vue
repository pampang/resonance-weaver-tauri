<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import IconButton from './IconButton.vue';

const appWebviewWindow = getCurrentWebviewWindow();

interface ResonancePayload {
  id: number;
  app_name: string;
  score: number;
  content: string;
  matched_content: string;
}

const payload = ref<ResonancePayload | null>(null);
const timerProgress = ref(100);
const SHOW_DURATION = 8000;
let timerInterval: any = null;

const closeBubble = async () => {
  if (timerInterval) clearInterval(timerInterval);
  payload.value = null;
  await appWebviewWindow.hide();
};

const openHub = async () => {
  await invoke('show_main_window');
  await closeBubble();
};

const quickSynthesize = async () => {
  if (payload.value) {
    await invoke('open_deep_bridge', { 
      content: payload.value.content, 
      matchedContent: payload.value.matched_content 
    });
    await closeBubble();
  }
};

onMounted(async () => {
  await listen<ResonancePayload>('new-resonance', (event) => {
    payload.value = event.payload;
    timerProgress.value = 100;
    
    if (timerInterval) clearInterval(timerInterval);

    const start = Date.now();
    timerInterval = setInterval(() => {
      const elapsed = Date.now() - start;
      timerProgress.value = Math.max(0, 100 - (elapsed / SHOW_DURATION) * 100);
      if (elapsed >= SHOW_DURATION) {
        closeBubble();
      }
    }, 50);
  });
});
</script>

<template>
  <div class="bubble-window-root">
    <div v-if="payload" class="bubble-container" @click="openHub">
      <div class="bubble-inner">
        <div class="glow-border"></div>
        
        <div class="bubble-content">
          <div class="header">
            <div class="app-info">
              <div class="pulse-dot"></div>
              <span class="app-name">{{ payload.app_name }}</span>
            </div>
            <div class="score-badge">
              {{ (payload.score * 100).toFixed(0) }}% Resonance
            </div>
          </div>

          <p class="preview">{{ payload.content }}</p>

          <div class="footer">
            <button class="synthesis-btn" @click.stop="quickSynthesize">
              ✨ Synthesize
            </button>
            <span class="hint">Click to Open Hub</span>
          </div>
        </div>

        <div class="timer-bar">
          <div class="timer-fill" :style="{ width: `${timerProgress}%` }"></div>
        </div>
      </div>
      <div class="abs-close">
        <IconButton size="sm" @click="closeBubble" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.bubble-window-root {
  width: 100%;
  height: 100%;
  background: transparent;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.bubble-container {
  width: 100%;
  height: 100%;
  padding: 10px;
  animation: slideUp 0.4s cubic-bezier(0.16, 1, 0.3, 1);
  position: relative;
  background: transparent;
}

@keyframes slideUp {
  from { transform: translateY(20px); opacity: 0; }
  to { transform: translateY(0); opacity: 1; }
}

.bubble-inner {
  width: 100%;
  height: 100%;
  background: rgba(20, 20, 35, 0.85);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-radius: 16px;
  position: relative;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.6);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.glow-border {
  position: absolute;
  top: 0; left: 0; right: 0; bottom: 0;
  border-radius: 16px;
  pointer-events: none;
  background: linear-gradient(135deg, rgba(100, 108, 255, 0.4) 0%, transparent 50%, rgba(183, 148, 255, 0.4) 100%);
  z-index: 0;
}

.bubble-content {
  padding: 12px 16px;
  display: flex;
  flex-direction: column;
  height: 100%;
  z-index: 1;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.app-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.pulse-dot {
  width: 7px;
  height: 7px;
  background: #646cff;
  border-radius: 50%;
  box-shadow: 0 0 10px #646cff;
}

.app-name {
  font-size: 0.85rem;
  font-weight: 700;
  color: #fff;
}

.score-badge {
  font-size: 0.7rem;
  font-weight: 800;
  color: #fff;
  background: linear-gradient(90deg, #646cff, #b794ff);
  padding: 2px 10px;
  border-radius: 20px;
}

.preview {
  margin: 0;
  font-size: 0.85rem;
  color: #ddd;
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  flex-grow: 1;
}

.footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 8px;
}

.hint {
  font-size: 0.65rem;
  color: #777;
}

.synthesis-btn {
  background: #646cff;
  color: white;
  border: none;
  padding: 5px 14px;
  border-radius: 8px;
  font-size: 0.75rem;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.2s;
}

.synthesis-btn:hover {
  background: #535bf2;
  transform: scale(1.05);
}

.timer-bar {
  position: absolute;
  bottom: 0;
  left: 0;
  width: 100%;
  height: 3px;
  background: rgba(255, 255, 255, 0.05);
}

.timer-fill {
  height: 100%;
  background: #646cff;
}

.abs-close {
  position: absolute;
  top: 14px;
  right: 14px;
  z-index: 100;
}
</style>
