# Resonance Weaver Standalone - Design Specification

> **Vibe:** "A unified terminal for system-wide cognitive resonance and inspiration triage."

## 1. Vision
Transform the act of "copying" into an intelligent knowledge-bridging event. This standalone application monitors the macOS clipboard across white-listed apps, identifies connections to multiple knowledge base directories, and provides a low-friction interface for deep synthesis via Gemini.

## 2. Core Features
- **Multi-Source Knowledge Base**: Monitor a list of local directories to build a comprehensive vector index.
- **Filtering Funnel**: A tiered approach (Metadata -> App Source -> Semantic) to minimize system load and noise.
- **Triage Buffer**: Sampled inspirations are stored in a local database for 7 days before auto-purging.
- **Scalable Reminders**: Toggle between Menu Bar Badge, Ambient Edge Pulse, and Active Floating Bubbles.
- **Deep Bridge**: Synthesis of Super Prompts for one-click deep dialogue in Gemini.

## 3. Architecture

### 3.1 Tech Stack
- **Framework**: Tauri or Electron (Vue 3 for the frontend).
- **Vector DB**: LanceDB (embedded, local storage).
- **Embeddings**: Ollama (`bge-m3` model, local API).
- **System Hooks**: Node.js `clipboardy` or native macOS APIs for clipboard and AppleScript for app detection.

### 3.2 System Components
1. **The Daemon**: A background process handling the clipboard listener and the filtering funnel.
2. **The Indexer**: Watches the configured directory list, extracting content and updating the vector store in real-time.
3. **The Buffer (SQLite/LanceDB)**: Stores recent "sparks" with a timestamp-based TTL (7 days).
4. **The Triage UI**: A central hub to review, synthesize, or discard stored inspirations.

## 4. Logical Flow

### 4.1 The Filtering Funnel
1. **Tier 1 (Instant)**: Ignore copies less than 30 characters, non-text formats, or from blacklisted apps (e.g., password managers).
2. **Tier 2 (Contextual)**: Check if the frontmost app is in the user-defined whitelist (e.g., WeRead, Books, Arc).
3. **Tier 3 (Semantic)**: Generate embedding via local Ollama and query the LanceDB index. If distance < user-threshold, it's a "Match".

### 4.2 Reminder Mechanisms
- **Quiet**: Menu bar icon shows a badge with the count of new matches.
- **Ambient**: A 2-second blue pulse animation on the right edge of the screen.
- **Active**: A small floating bubble appears briefly near the cursor or bottom-right corner.

## 5. Configuration (JSON/YAML)
```json
{
  "kb_sources": [
    "/path/to/obsidian/03_Nodes/Atoms",
    "/path/to/obsidian/03_Nodes/Stories"
  ],
  "app_whitelist": ["WeRead", "Books", "Arc"],
  "resonance_threshold": 0.45,
  "reminder_mode": ["badge", "pulse"],
  "buffer_ttl_days": 7
}
```

## 6. Privacy & Security
- **Strictly Local**: All text analysis and embeddings remain on the user's machine.
- **App Awareness**: The daemon only processes clipboard content when a whitelisted application is in focus.

---
*Created by Gemini @ 2026-04-19 | Version 1.0 (Standalone Evolution)*
