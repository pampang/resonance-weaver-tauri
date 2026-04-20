# Resonance Weaver Standalone Implementation Plan (Tauri Edition)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a lightweight macOS application (Tauri + Vue 3) that monitors the clipboard for cognitive resonance across white-listed apps and provides a triage interface for deep Gemini synthesis.

**Architecture:** Tauri (Rust) backend for system-level hooks (clipboard, AppleScript) and high-performance vector storage (LanceDB Rust). Vue 3 frontend for the triage hub.

**Tech Stack:** Tauri, Rust, Vue 3, LanceDB (Rust SDK), Ollama, SQLite (rusqlite).

---

### Task 1: Project Scaffolding & Initial Configuration

**Files:**
- Create: `~/workspace/resonance-weaver-tauri/`
- Create: `~/workspace/resonance-weaver-tauri/src-tauri/tauri.conf.json`

- [ ] **Step 1: Initialize Tauri + Vue project**
```bash
mkdir -p ~/workspace/resonance-weaver-tauri && cd ~/workspace/resonance-weaver-tauri
npm create tauri-app@latest . -- --template vue-ts
npm install
cd src-tauri
cargo add lancedb arrow rusqlite chrono serde_json reqwest tokio-chokidar
```

- [ ] **Step 2: Setup local configuration logic**
Implement a `config.rs` to handle `kb_sources`, `app_whitelist`, and `threshold` in `Library/Application Support`.

- [ ] **Step 3: Setup tracing/logging**
Integrate `tauri-plugin-log` or a simple Rust `env_logger` to capture system events.

- [ ] **Step 4: Commit**
```bash
git add .
git commit -m "chore: initial tauri scaffolding with rust dependencies"
```

---

### Task 2: Rust Backend: The Indexer & Vector Store

**Files:**
- Create: `src-tauri/src/services/vector_store.rs`
- Create: `src-tauri/src/services/indexer.rs`

- [ ] **Step 1: Implement LanceDB Indexing in Rust**
Initialize LanceDB. Implement `index_file` that calls Ollama's embedding API via `reqwest` and stores it in LanceDB.

- [ ] **Step 2: Implement Multi-Source Watcher**
Use `notify` or `tokio-chokidar` in Rust to monitor the directory list. Implement "Semantic Dehydration" logic in Rust.

- [ ] **Step 3: Export Search Command**
Create a Tauri command `search_resonance(text: String)` that returns Top 3 matches.

- [ ] **Step 4: Commit**
```bash
git commit -m "feat(rust): lancedb indexer and multi-source file watcher"
```

---

### Task 3: The Daemon: Clipboard Monitor & Funnel

**Files:**
- Create: `src-tauri/src/services/clipboard.rs`
- Create: `src-tauri/src/services/funnel.rs`

- [ ] **Step 1: Implement Rust Clipboard Listener**
Spawn a background tokio task that polls the system clipboard using the `arboard` or `arboard-clipboard` crate.

- [ ] **Step 2: Implement App Source Detection**
Use `std::process::Command` to execute the AppleScript that gets the frontmost app name.

- [ ] **Step 3: Implement The Filtering Funnel**
Apply Tier 1 (Metadata) and Tier 2 (Whitelist) before invoking the heavy Tier 3 (Ollama Embedding).

- [ ] **Step 4: Commit**
```bash
git commit -m "feat(rust): background clipboard listener with app-aware filtering"
```

---

### Task 4: The Triage Buffer (SQLite)

**Files:**
- Create: `src-tauri/src/services/db.rs`

- [ ] **Step 1: Setup rusqlite Schema**
Initialize `buffer.db` with a `samples` table.

- [ ] **Step 2: Implement 7-Day TTL**
Add logic to the app startup to delete records older than `chrono::Utc::now() - 7 days`.

- [ ] **Step 3: Commit**
```bash
git commit -m "feat(rust): triage buffer storage with auto-purge"
```

---

### Task 5: The GUI: Triage Hub & Deep Bridge

**Files:**
- Create: `src/components/TriageList.vue`
- Create: `src/components/ConfigPanel.vue`

- [ ] **Step 1: Implement Triage List**
Use `invoke('get_samples')` to fetch data from Rust SQLite. Show cards with match distance.

- [ ] **Step 2: Implement Deep Bridge Redirect**
A button that calls a Rust command to open the browser with the synthesized Gemini URL or copy to clipboard and open.

- [ ] **Step 3: Commit**
```bash
git commit -m "feat(ui): vue 3 triage hub and prompt synthesis"
```

---

### Task 6: System Tray & Reminders

**Files:**
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Setup Tauri System Tray**
Show icon in the menu bar. Implement "Update Badge" equivalent or a simple "New Resonance Found" tooltip/notification.

- [ ] **Step 2: Implement Reminder Overlays**
Use Tauri's multi-window support to create a small, transparent, click-through window for "Edge Pulse" or "Bubbles".

- [ ] **Step 3: Final Verification & Logs**
Audit all logs for "Selection", "Search", "Match", and "Purge" events.

- [ ] **Step 4: Commit**
```bash
git commit -m "feat(ui): system tray and reminder modes"
```
