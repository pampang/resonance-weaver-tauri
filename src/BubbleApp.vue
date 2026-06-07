<template>
  <div
    class="bubble-root"
    :class="{ 'is-visible': visible, 'is-exiting': exiting }"
    @mouseenter="pauseTimer"
    @mouseleave="resumeTimer"
  >
    <div class="bubble-card">
      <!-- Close button -->
      <button class="close-btn" @click="dismiss" aria-label="Close">×</button>

      <!-- Header -->
      <div class="header">
        <div class="score-badge" :class="scoreClass">
          <span class="score-text">{{ scorePercent }}%</span>
        </div>
        <span class="title">Resonance Detected</span>
        <span class="app-chip">{{ data?.app_name ?? '—' }}</span>
      </div>

      <!-- Content snippets -->
      <div class="snippets">
        <div class="snippet-row">
          <span class="snippet-label">Captured:</span>
          <span class="snippet-text">{{ data?.content ?? '' }}</span>
        </div>
        <div class="snippet-row">
          <span class="snippet-label">Matched:</span>
          <span class="snippet-text">{{ data?.matched_content ?? '' }}</span>
        </div>
      </div>

      <!-- Actions -->
      <div class="actions">
        <button class="btn btn-outline" @click="openHub">Open Hub</button>
        <button class="btn btn-accent" @click="synthesize">✨ Synthesize</button>
      </div>

      <!-- Countdown bar -->
      <div class="countdown-bar-track">
        <div
          class="countdown-bar"
          :style="{ transform: `scaleX(${countdownProgress})` }"
          :class="{ paused: timerPaused }"
        ></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface BubbleData {
  id: number;
  app_name: string;
  score: number;
  content: string;
  matched_content: string;
}

const DISMISS_MS = 8000;
const TICK_MS = 50;

const data = ref<BubbleData | null>(null);
const visible = ref(false);
const exiting = ref(false);
const countdownProgress = ref(1);
const timerPaused = ref(false);

let elapsed = 0;
let tickInterval: ReturnType<typeof setInterval> | null = null;
let unlisten: UnlistenFn | null = null;

const scorePercent = computed(() => {
  if (!data.value) return 0;
  return Math.round(data.value.score * 100);
});

const scoreClass = computed(() => {
  if (!data.value) return "score-gray";
  if (data.value.score > 0.9) return "score-purple";
  if (data.value.score > 0.75) return "score-blue";
  return "score-gray";
});

function startCountdown() {
  elapsed = 0;
  countdownProgress.value = 1;
  tickInterval = setInterval(() => {
    if (timerPaused.value) return;
    elapsed += TICK_MS;
    countdownProgress.value = Math.max(0, 1 - elapsed / DISMISS_MS);
    if (elapsed >= DISMISS_MS) {
      dismiss();
    }
  }, TICK_MS);
}

function pauseTimer() {
  timerPaused.value = true;
}

function resumeTimer() {
  timerPaused.value = false;
}

async function dismiss() {
  if (exiting.value) return;
  exiting.value = true;
  if (tickInterval) {
    clearInterval(tickInterval);
    tickInterval = null;
  }
  // Wait for exit animation
  await new Promise((r) => setTimeout(r, 300));
  await getCurrentWindow().close();
}

async function openHub() {
  try {
    await invoke("show_main_window");
  } catch {
    // Command may not exist yet during development
  }
  await dismiss();
}

async function synthesize() {
  if (!data.value) return;
  try {
    await invoke("open_deep_bridge", {
      content: data.value.content,
      matchedContent: data.value.matched_content,
    });
  } catch {
    // Command may not exist yet during development
  }
  await dismiss();
}

onMounted(async () => {
  unlisten = await listen<BubbleData>("bubble-data", (event) => {
    data.value = event.payload;
    // Trigger entry animation on next tick
    requestAnimationFrame(() => {
      visible.value = true;
    });
    startCountdown();
  });
});

onUnmounted(() => {
  if (tickInterval) clearInterval(tickInterval);
  if (unlisten) unlisten();
});
</script>

<style scoped>
.bubble-root {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: system-ui, -apple-system, BlinkMacSystemFont, sans-serif;
  pointer-events: none;
}

.bubble-card {
  position: relative;
  width: 400px;
  max-width: calc(100vw - 20px);
  background: rgba(15, 15, 20, 0.92);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-radius: 16px;
  border: 1px solid rgba(100, 108, 255, 0.25);
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.5),
    0 0 20px rgba(183, 148, 255, 0.1);
  padding: 20px;
  padding-bottom: 14px;
  color: #e8e8e8;
  overflow: hidden;
  pointer-events: auto;

  /* Entry animation — starts offscreen right */
  transform: translateX(100%);
  opacity: 0;
  transition:
    transform 0.4s cubic-bezier(0.22, 1, 0.36, 1),
    opacity 0.4s ease-out;
}

.bubble-root.is-visible .bubble-card {
  transform: translateX(0);
  opacity: 1;
}

.bubble-root.is-exiting .bubble-card {
  transform: translateX(100%);
  opacity: 0;
  transition:
    transform 0.3s ease-in,
    opacity 0.3s ease-in;
}

/* Close button */
.close-btn {
  position: absolute;
  top: 8px;
  right: 10px;
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.4);
  font-size: 18px;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 4px;
  line-height: 1;
  transition: color 0.2s, background 0.2s;
}
.close-btn:hover {
  color: #fff;
  background: rgba(255, 255, 255, 0.1);
}

/* Header */
.header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 14px;
}

.score-badge {
  width: 38px;
  height: 38px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  animation: pulse 2s ease-in-out infinite;
}
.score-badge.score-purple {
  background: rgba(183, 148, 255, 0.2);
  border: 2px solid #b794ff;
  box-shadow: 0 0 12px rgba(183, 148, 255, 0.35);
}
.score-badge.score-blue {
  background: rgba(100, 210, 255, 0.2);
  border: 2px solid #64d2ff;
  box-shadow: 0 0 12px rgba(100, 210, 255, 0.35);
}
.score-badge.score-gray {
  background: rgba(136, 136, 136, 0.2);
  border: 2px solid #888;
  box-shadow: 0 0 12px rgba(136, 136, 136, 0.2);
}
.score-text {
  font-size: 12px;
  font-weight: 700;
  color: #fff;
}

@keyframes pulse {
  0%,
  100% {
    transform: scale(1);
  }
  50% {
    transform: scale(1.06);
  }
}

.title {
  font-size: 15px;
  font-weight: 600;
  color: #fff;
  flex: 1;
}

.app-chip {
  font-size: 11px;
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 8px;
  padding: 3px 8px;
  color: rgba(255, 255, 255, 0.6);
  white-space: nowrap;
  max-width: 100px;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* Snippets */
.snippets {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
}

.snippet-row {
  display: flex;
  gap: 6px;
  align-items: flex-start;
}

.snippet-label {
  font-size: 11px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.45);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  flex-shrink: 0;
  padding-top: 1px;
}

.snippet-text {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.8);
  line-height: 1.45;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* Actions */
.actions {
  display: flex;
  gap: 10px;
  margin-bottom: 12px;
}

.btn {
  flex: 1;
  padding: 8px 14px;
  border-radius: 10px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  border: none;
  transition:
    background 0.2s,
    transform 0.15s,
    box-shadow 0.2s;
  font-family: inherit;
}
.btn:active {
  transform: scale(0.97);
}

.btn-outline {
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.2);
  color: rgba(255, 255, 255, 0.75);
}
.btn-outline:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.35);
  color: #fff;
}

.btn-accent {
  background: linear-gradient(135deg, #646cff, #b794ff);
  color: #fff;
  box-shadow: 0 2px 12px rgba(100, 108, 255, 0.3);
}
.btn-accent:hover {
  box-shadow: 0 4px 20px rgba(100, 108, 255, 0.5);
}

/* Countdown bar */
.countdown-bar-track {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 3px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 0 0 16px 16px;
  overflow: hidden;
}

.countdown-bar {
  height: 100%;
  width: 100%;
  background: linear-gradient(90deg, #646cff, #b794ff);
  transform-origin: left center;
  transition: transform 0.06s linear;
}
.countdown-bar.paused {
  opacity: 0.5;
}
</style>
