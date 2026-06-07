# Resonance Weaver

> Transform the act of copying into intelligent knowledge discovery.

Resonance Weaver is a macOS desktop application that monitors your clipboard in real-time, detects meaningful text from whitelisted applications, and semantically matches it against your local knowledge base. When a significant connection is found, it surfaces the insight through an elegant floating notification — enabling serendipitous discovery and deeper thinking without interrupting your workflow.

## ✨ Features

### Core Intelligence
- **Clipboard Monitoring** — Continuously watches for meaningful text copied from whitelisted apps (Slack, Discord, Notes, etc.)
- **Semantic Search** — Uses local Ollama embedding models to find deep connections between captured text and your knowledge base
- **Multi-Source Indexing** — Index multiple local directories with automatic file watching and real-time updates
- **Smart Filtering Funnel** — Three-tier filtering (length → app whitelist → semantic threshold) minimizes noise

### User Experience  
- **Floating Bubble Notifications** — Non-intrusive, glassmorphism-styled overlays appear when resonances are detected, showing match quality and enabling quick actions
- **Triage Hub** — Master-detail split view for reviewing, exploring, and managing captured resonances with Markdown rendering
- **Deep Bridge Synthesis** — One-click prompt synthesis and handoff to Gemini for deeper exploration
- **Native Notifications** — System-level alerts complement the in-app experience
- **System Tray Integration** — Background operation with quick access via menu bar

### Configuration
- **Knowledge Sources** — Add/remove folders via native folder picker
- **App Whitelist** — Control which apps trigger monitoring, with live detection of running apps
- **Embedding Models** — Choose between `nomic-embed-text` (fast) and `bge-m3` (multilingual)
- **Resonance Threshold** — Fine-tune match sensitivity with a precision slider

## 🏗️ Architecture

```
┌──────────────────────────────────────────────────┐
│                    Tauri v2                       │
│  ┌────────────────────────────────────────────┐  │
│  │              Vue 3 Frontend                │  │
│  │  ┌──────────┐ ┌──────────┐ ┌───────────┐  │  │
│  │  │ Triage   │ │  Config  │ │  Bubble   │  │  │
│  │  │  Hub     │ │  Panel   │ │  Overlay  │  │  │
│  │  └──────────┘ └──────────┘ └───────────┘  │  │
│  └────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────┐  │
│  │              Rust Backend                  │  │
│  │  ┌──────────┐ ┌──────────┐ ┌───────────┐  │  │
│  │  │Clipboard │ │  Funnel  │ │  Indexer  │  │  │
│  │  │ Listener │→│ Pipeline │ │  + Watch  │  │  │
│  │  └──────────┘ └──────────┘ └───────────┘  │  │
│  │  ┌──────────┐ ┌──────────┐                │  │
│  │  │ LanceDB  │ │  SQLite  │                │  │
│  │  │ Vectors  │ │  Buffer  │                │  │
│  │  └──────────┘ └──────────┘                │  │
│  └────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────┘
         │                    │
    Ollama API          File System
   (Embeddings)       (Knowledge Base)
```

## 🛠️ Tech Stack

| Layer | Technology |
|-------|------------|
| Framework | Tauri v2 |
| Frontend | Vue 3 + TypeScript (Vite) |
| Backend | Rust |
| Vector DB | LanceDB (embedded, local) |
| Relational DB | SQLite (rusqlite) |
| Embeddings | Ollama (local API) |
| Clipboard | arboard crate |
| File Watching | notify crate |

## 📋 Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (v18+)
- [Ollama](https://ollama.ai/) with an embedding model:
  ```bash
  ollama pull nomic-embed-text
  # or for multilingual support:
  ollama pull bge-m3
  ```

## 🚀 Getting Started

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## 📁 Project Structure

```
resonance-weaver-tauri/
├── src/                    # Vue 3 frontend
│   ├── components/         # Vue components
│   │   ├── TriageList.vue  # Main triage hub view
│   │   ├── ConfigPanel.vue # Configuration interface
│   │   └── IconButton.vue  # Reusable icon button
│   ├── App.vue             # Root app with tab navigation
│   ├── BubbleApp.vue       # Floating notification overlay
│   └── main.ts             # App entry point
├── src-tauri/              # Rust backend
│   └── src/
│       ├── services/
│       │   ├── clipboard.rs  # Clipboard polling daemon
│       │   ├── funnel.rs     # Capture → Embed → Search → Save pipeline
│       │   ├── indexer.rs    # File indexer with fs watcher
│       │   ├── vector_store.rs # LanceDB vector operations
│       │   └── db.rs         # SQLite triage buffer
│       ├── config.rs         # Configuration management
│       └── lib.rs            # Tauri app setup & commands
├── docs/                   # Design specs & plans
└── bubble.html             # Bubble notification entry point
```

## 🔒 Privacy

Resonance Weaver is **100% local**. All text analysis, embeddings, and storage remain on your machine. No data is sent to external servers. The only network request is to your local Ollama instance.

## 📄 License

MIT
