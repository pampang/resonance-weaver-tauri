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
  <!-- Main canvas stays transparent so the rounded corners of the card work -->
  <div class="bubble-canvas">
    
    <div v-if="payload" class="solid-dark-card" @click="openHub">
      <div class="header">
        <div class="app-pill">
          <span class="pulse-dot"></span>
          <span class="app-name">{{ payload.app_name }}</span>
        </div>
        <div class="score-badge">{{ (payload.score * 100).toFixed(0) }}% Match</div>
      </div>

      <div class="content-preview">
        {{ payload.content }}
      </div>

      <div class="footer">
        <button class="synth-btn" @click.stop="quickSynthesize">✨ Synthesize</button>
        <div class="timer-track">
          <div class="timer-fill" :style="{ width: `${timerProgress}%` }"></div>
        </div>
      </div>

      <div class="close-hit">
        <IconButton size="sm" @click="closeBubble" />
      </div>
    </div>

  </div>
</template>

<style scoped>
.bubble-canvas {
  width: 100vw; height: 100vh;
  background: transparent !important;
  display: flex; align-items: center; justify-content: center;
  padding: 12px; overflow: hidden;
  user-select: none;
}

/* The Solid Dark Card replaces the tricky Glassmorphism */
.solid-dark-card {
  width: 100%; height: 100%;
  background: #181822; /* Solid dark color */
  border: 1px solid rgba(100, 108, 255, 0.4); /* Colored border for distinction */
  border-radius: 16px;
  display: flex; flex-direction: column;
  padding: 16px; position: relative;
  box-shadow: 0 12px 30px rgba(0, 0, 0, 0.8);
  animation: popIn 0.4s cubic-bezier(0.16, 1, 0.3, 1);
  cursor: pointer;
}

@keyframes popIn {
  from { transform: scale(0.95) translateY(10px); opacity: 0; }
  to { transform: scale(1) translateY(0); opacity: 1; }
}

.header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
.app-pill { display: flex; align-items: center; gap: 8px; background: rgba(255,255,255,0.08); padding: 4px 10px; border-radius: 12px; }
.pulse-dot { width: 6px; height: 6px; background: #646cff; border-radius: 50%; box-shadow: 0 0 8px #646cff; }
.app-name { font-size: 0.8rem; font-weight: 800; color: #fff; }
.score-badge { font-size: 0.7rem; color: #b794ff; font-weight: 900; }

.content-preview {
  font-size: 0.9rem; color: #eee; line-height: 1.4;
  display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical;
  overflow: hidden; flex-grow: 1; margin-bottom: 12px;
}

.footer { display: flex; align-items: center; gap: 12px; }
.synth-btn {
  background: #646cff; color: #fff; border: none; padding: 6px 14px; border-radius: 8px;
  font-size: 0.75rem; font-weight: 800; cursor: pointer; box-shadow: 0 4px 10px rgba(0,0,0,0.3);
  transition: background 0.2s;
}

.synth-btn:hover { background: #535bf2; }

.timer-track { flex-grow: 1; height: 3px; background: rgba(255,255,255,0.1); border-radius: 2px; overflow: hidden; }
.timer-fill { height: 100%; background: #646cff; }

.close-hit { position: absolute; top: 12px; right: 12px; }
</style>
