<p align="center">
  <img src="docs/assets/logo.png" alt="Prompt Git mascot — a blue and pink seahorse" width="148" />
</p>

<h1 align="center">Prompt Git</h1>

<p align="center">
  <strong>Prompt Git for AI Developers</strong><br />
  The lightweight, local-first desktop workbench for Prompt versioning,<br />
  multi-model evaluation, and batch variable testing.
</p>

<p align="center">
  <a href="#features"><img src="https://img.shields.io/badge/version-0.1.0-3d9cf0?style=flat-square" alt="version" /></a>
  <a href="#license"><img src="https://img.shields.io/badge/license-MIT-22c55e?style=flat-square" alt="license" /></a>
  <a href="#architecture"><img src="https://img.shields.io/badge/stack-Tauri%202%20%2B%20React%20%2B%20Rust-1a2330?style=flat-square" alt="stack" /></a>
  <a href="#privacy--security"><img src="https://img.shields.io/badge/privacy-100%25%20local-e6b450?style=flat-square" alt="privacy" /></a>
  <a href="docs/zh-CN.md"><img src="https://img.shields.io/badge/docs-%E4%B8%AD%E6%96%87-8b9bb4?style=flat-square" alt="Chinese docs" /></a>
</p>

<p align="center">
  <a href="#why-prompt-git">Why</a> ·
  <a href="#features">Features</a> ·
  <a href="#quick-start">Quick Start</a> ·
  <a href="#workflow">Workflow</a> ·
  <a href="#architecture">Architecture</a> ·
  <a href="#privacy--security">Privacy</a> ·
  <a href="#roadmap">Roadmap</a>
</p>

---

## Why Prompt Git

Modern LLM apps live and die by their prompts — yet most teams still manage them like sticky notes:

| Pain point | What usually happens | What Prompt Git does |
|---|---|---|
| **No version history** | Prompts are hardcoded or scattered across docs | Git-like commits, tags, diffs, and one-click rollback |
| **Slow multi-model checks** | Copy-paste the same prompt into every console | Concurrent compare across OpenAI, Claude, DeepSeek, Ollama, and more |
| **Painful variable sweeps** | Manual find-replace for every `{{var}}` combo | Template parsing, CSV import, cartesian product, result matrix |
| **Cloud-only SaaS tax** | Keys and IP leave your machine | 100% local storage — no backend, no telemetry upload path |

Prompt Git is deliberately small: a native desktop app that feels like **Git for prompts**, not another heavyweight MLOps suite.

---

## Features

### 1. Prompt version control (Git-like)

- Create **projects**, **folders**, and **prompt files**
- **Commit** system + user prompts with required messages
- Browse **history** (newest first) with hash, message, timestamp
- **Diff** any two versions line-by-line (insert / delete / equal)
- Attach **tags** (e.g. Production / Testing) with custom colors
- **Rollback** with an automatic pre-rollback snapshot
- Add **remarks** for evaluation notes and release decisions

### 2. Multi-model effect comparison

- Fire the same prompt at multiple configured models **in parallel**
- Built-in adapters:
  - **Commercial**: OpenAI, Anthropic Claude, DeepSeek, Qwen, Doubao, Wenxin
  - **Local**: Ollama (auto-discover installed models)
  - **Custom**: any OpenAI-compatible base URL
- Shared or per-run inference params: `temperature`, `max_tokens`, `top_p`, `frequency_penalty`
- Score outputs on accuracy / instruction / format / speed, mark a **best** result
- Persist compare tasks and reopen history anytime
- Export a **Markdown compare report** in one click

### 3. Batch variable testing

- Template syntax: `{{name}}` and `{{name:default}}`
- Auto-detect variables from the editor
- Generate cases via:
  - multi-line value lists → **cartesian product**
  - **CSV import** (header row required)
- Configurable **concurrency** to respect rate limits
- Result matrix with latency, scores, and “save best case as new version”

### 4. One-click export & integration

| Format | Use case |
|---|---|
| **Code snippets** | Python, JavaScript, TypeScript, Go, Java, Rust, cURL |
| **JSON / YAML** | Portable prompt + params config |
| **Markdown report** | Shareable evaluation write-up |
| **Plain text** | Single-version prompt dump |

### 5. Desktop UX built for developers

- Classic three-pane layout: project tree · editor · history / variables
- Dark / light / system themes
- `⌘/Ctrl + S` opens the commit dialog
- Optional **app password** gate on launch

---

## Quick Start

### Prerequisites

| Tool | Version |
|---|---|
| Node.js | **20+** (see `.nvmrc`) |
| pnpm | 9+ |
| Rust | stable toolchain |
| OS deps | [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) |

### Install & run (dev)

```bash
git clone <your-repo-url> prompt-git
cd prompt-git

# optional but recommended
nvm use

pnpm install
pnpm tauri dev
```

### Production build

```bash
pnpm tauri build
```

Installers are emitted by Tauri under `src-tauri/target/release/bundle/`.

### First 60 seconds

1. **Settings** → add at least one model (API key is encrypted locally)
2. Create a **project** and a **prompt file**
3. Edit system / user prompts → **Commit** (`⌘/Ctrl + S`)
4. Open **Compare** or **Batch Test** to evaluate
5. **Export** code or JSON into your product repo

---

## Workflow

```text
                 ┌──────────────────────┐
                 │   Prompt Editor      │
                 │  system + user text  │
                 └──────────┬───────────┘
                            │ commit / tag / rollback
                            ▼
                 ┌──────────────────────┐
                 │  Version Snapshots   │
                 │  hash · diff · notes │
                 └──────────┬───────────┘
              ┌─────────────┼─────────────┐
              ▼             ▼             ▼
        ┌──────────┐  ┌──────────┐  ┌──────────┐
        │ Compare  │  │  Batch   │  │  Export  │
        │ N models │  │ {{vars}} │  │ code/MD  │
        └──────────┘  └──────────┘  └──────────┘
```

Typical loop:

1. Iterate the prompt in the editor  
2. Commit a named snapshot when quality improves  
3. Diff against the last stable tag  
4. Compare across models or sweep variables  
5. Promote the winner with a Production tag and export

---

## Architecture

```text
┌─────────────────────────────────────────────────┐
│              Frontend (React + TS)              │
│   pages · Zustand store · Diff UI · tables      │
└───────────────────────┬─────────────────────────┘
                        │ Tauri IPC (invoke)
┌───────────────────────┴─────────────────────────┐
│              Rust core (Tauri 2)                │
│  version engine · LLM adapters · batch · export │
├─────────────────────────────────────────────────┤
│  SQLite · filesystem · AES-GCM + OS keyring     │
├─────────────────────────────────────────────────┤
│  Model APIs (HTTPS) · Ollama (localhost)        │
└─────────────────────────────────────────────────┘
```

| Layer | Choices |
|---|---|
| Desktop shell | Tauri 2 |
| UI | React 19, TypeScript, Vite, Tailwind CSS 4, Zustand |
| Storage | SQLite via `rusqlite` (bundled) |
| HTTP | `reqwest` (rustls) |
| Crypto | AES-256-GCM + `keyring` (macOS Keychain / Windows Credential Manager / Linux Secret Service) |

Data root (created automatically):

```text
~/.prompt-git/
├── data.db          # primary SQLite database
├── config.json      # theme, concurrency, Ollama base URL
├── backups/         # manual / scheduled DB snapshots
├── exports/         # temporary export artifacts
└── cache/           # request cache directory
```

---

## Privacy & Security

Prompt Git is designed so sensitive material never needs a SaaS account:

- **No cloud backend** — prompts, scores, and configs stay on disk
- **API keys encrypted at rest** (AES-256-GCM); master key lives in the OS keyring
- **Optional launch password** before the workspace unlocks
- Model calls go **directly** to the vendor / Ollama endpoint over HTTPS (or local HTTP for Ollama)

> You own the data path. Back up `~/.prompt-git/` like any other local project vault.

---

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `⌘/Ctrl + S` | Open **Commit Version** dialog |

More shortcuts are planned as the editor matures.

---

## Project Layout

```text
prompt-git/
├── src/                 # React app (editor, compare, batch, settings)
├── src-tauri/           # Rust backend, IPC commands, LLM adapters
├── docs/
│   ├── assets/logo.png  # project mascot
│   └── zh-CN.md         # Chinese getting-started guide
├── package.json
└── README.md
```

---

## Roadmap

### v0.1.0 — MVP (current)

- [x] Project / file management  
- [x] Commit · history · diff · rollback · tags  
- [x] OpenAI-compatible + Ollama adapters  
- [x] Multi-model compare + scoring  
- [x] Variable batch tests + CSV  
- [x] Code / JSON / YAML / Markdown export  
- [x] Local encryption + optional app password  

### v1.0.0 — Production polish

- [ ] Branching & merge for parallel prompt lines  
- [ ] Streaming responses in compare view  
- [ ] Stronger CSV / regression suite UX  
- [ ] Hardened backup / restore flows  
- [ ] Bilingual docs + demo assets  

### Later

- [ ] LLM-as-a-Judge auto scoring  
- [ ] Prompt optimization suggestions  
- [ ] Plugin system for custom models / exporters  
- [ ] Optional Git repo sync for teams  

---

## Contributing

Issues and PRs are welcome. For larger changes, open an issue first so we can align on scope.

Suggested local checklist before a PR:

```bash
pnpm exec tsc --noEmit
cd src-tauri && cargo check
pnpm tauri dev   # smoke-test the critical path
```

---

## License

Released under the [MIT License](LICENSE).

---

<p align="center">
  <sub>Built for developers who treat prompts like production code.</sub><br />
  <sub><strong>Prompt Git</strong> — local, private, versioned.</sub>
</p>
