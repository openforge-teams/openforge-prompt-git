# Prompt Git

**Prompt Git for AI Developers** — a lightweight, 100% local open-source desktop tool for Prompt version control, multi-model comparison, and batch variable testing.

> No cloud backend. All data stays on your machine (`~/.prompt-git/`).

## Features (MVP v0.1.0)

- **Prompt versioning (Git-like)** — commit snapshots, history, diff, tags, rollback, remarks
- **Multi-model compare** — OpenAI / DeepSeek / Claude / Qwen / Doubao / Wenxin / Ollama / custom OpenAI-compatible APIs
- **Batch variable tests** — `{{var}}` templates, cartesian product, CSV import, result matrix
- **Export** — Python / JS / TS / Go / Java / Rust / cURL snippets, JSON, YAML, Markdown reports
- **Privacy** — SQLite local DB, AES-256-GCM encrypted API keys via OS keyring, optional app password

## Tech Stack

- **Frontend**: React 19 + TypeScript + Vite + Tailwind CSS + Zustand
- **Desktop**: Tauri 2
- **Backend**: Rust + SQLite (rusqlite) + reqwest

## Prerequisites

- Node.js 20+
- pnpm
- Rust stable + platform Tauri dependencies  
  See: https://v2.tauri.app/start/prerequisites/

## Develop

```bash
pnpm install
pnpm tauri dev
```

## Build

```bash
pnpm tauri build
```

## Data location

```
~/.prompt-git/
├── data.db
├── config.json
├── backups/
├── exports/
└── cache/
```

## Shortcuts

- `⌘/Ctrl + S` — open commit dialog

## License

MIT
