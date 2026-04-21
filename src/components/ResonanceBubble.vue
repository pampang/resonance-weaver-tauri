<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

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

const closeBubble = async () => {
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
    
    // Start countdown for progress bar
    const start = Date.now();
    const interval = setInterval(() => {
      const elapsed = Date.now() - start;
      timerProgress.value = Math.max(0, 100 - (elapsed / SHOW_DURATION) * 100);
      if (elapsed >= SHOW_DURATION) {
        clearInterval(interval);
        closeBubble();
      }
    }, 50);
  });
});
</script>

<template>
  <div v-if="payload" class="bubble-window" @click="openHub">
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

      <!-- Auto-hide progress bar -->
      <div class="timer-bar">
        <div class="timer-fill" :style="{ width: `${timerProgress}%` }"></div>
      </div>
    </div>
    <button class="close-btn" @click.stop="closeBubble">✕</button>
  </div>
</template>

<style scoped>
.bubble-window {
  width: 100%;
  height: 100%;
  padding: 8px;
  box-sizing: border-box;
  animation: slideUp 0.4s cubic-bezier(0.16, 1, 0.3, 1);
  position: relative;
}

@keyframes slideUp {
  from { transform: translateY(20px); opacity: 0; }
  to { transform: translateY(0); opacity: 1; }
}

.bubble-inner {
  width: 100%;
  height: 100%;
  background: rgba(15, 15, 25, 0.85);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border-radius: 14px;
  position: relative;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.glow-border {
  position: absolute;
  top: 0; left: 0; right: 0; bottom: 0;
  border-radius: 14px;
  pointer-events: none;
  background: linear-gradient(135deg, rgba(100, 108, 255, 0.3) 0%, transparent 50%, rgba(183, 148, 255, 0.3) 100%);
  z-index: 0;
}

.bubble-content {
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  height: 100%;
  z-index: 1;
  position: relative;
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
  gap: 6px;
}

.pulse-dot {
  width: 6px;
  height: 6px;
  background: #646cff;
  border-radius: 50%;
  box-shadow: 0 0 8px #646cff;
}

.app-name {
  font-size: 0.8rem;
  font-weight: 700;
  color: #fff;
  opacity: 0.9;
}

.score-badge {
  font-size: 0.7rem;
  font-weight: 800;
  color: #b794ff;
  background: rgba(183, 148, 255, 0.15);
  padding: 2px 8px;
  border-radius: 6px;
}

.preview {
  margin: 0;
  font-size: 0.85rem;
  color: #ccc;
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
  color: #555;
}

.synthesis-btn {
  background: #646cff;
  color: white;
  border: none;
  padding: 4px 12px;
  border-radius: 6px;
  font-size: 0.75rem;
  font-weight: 700;
  cursor: pointer;
  box-shadow: 0 4px 10px rgba(100, 108, 255, 0.3);
  transition: transform 0.2s;
}

.synthesis-btn:hover {
  background: #535bf2;
  transform: translateY(-1px);
}

.timer-bar {
  position: absolute;
  bottom: 0;
  left: 0;
  width: 100%;
  height: 2px;
  background: rgba(255, 255, 255, 0.05);
}

.timer-fill {
  height: 100%;
  background: #646cff;
  transition: width 0.05s linear;
}

.close-btn {
  position: absolute;
  top: 12px;
  right: 12px;
  background: transparent;
  border: none;
  color: #444;
  cursor: pointer;
  z-index: 10;
  font-size: 0.8rem;
}

.close-btn:hover { color: #fff; }
</style>
