# PaperForge

Current version: `1.0.0`

PaperForge is a local-first AI manuscript writing desktop app. It organizes papers as local folders and connects Markdown sections, references, attachments, exports, and replaceable AI model settings.

PaperForge does not replace Word, LaTeX, or Zotero. It is an integration and writing-assistance layer.

## Install

```bash
git clone https://github.com/YezhiHuan/PaperForge.git
cd PaperForge
npm install
```

## Development

```bash
npm run tauri dev
```

Useful checks:

```bash
npm run typecheck
npm run build
npm run tauri build
```

Lint is not configured in v1.0.0.

## Workspace Init

`paperforge init` initializes the global PaperForge workspace. It does not create a paper project.

Portable npm script:

```bash
npm run paperforge:init
```

Direct CLI entry is also included as `scripts/paperforge.mjs` and exposed through package `bin` for future global installs.

Default workspace:

```text
PaperForgeWorkspace/
├─ .paperforge/
│  ├─ workspace.json
│  ├─ ai-models.json
│  ├─ settings.json
│  └─ history.log
└─ papers/
```

`papers/` starts empty. Paper folders are created only from New Paper in the app.

`workspace.json`:

```json
{
  "version": "1.0.0",
  "workspaceName": "PaperForge Workspace",
  "createdAt": "...",
  "updatedAt": "...",
  "papersDir": "papers",
  "defaultLanguage": "en"
}
```

## AI Model CLI

AI model config is saved to:

```text
PaperForgeWorkspace/.paperforge/ai-models.json
```

Commands:

```bash
npm run paperforge:model:add -- --provider openai-compatible --name local --base-url http://localhost:11434/v1 --api-key YOUR_API_KEY --model llama3
npm run paperforge:model:list
npm run paperforge:model:set-default -- local
npm run paperforge:model:remove -- local
```

Supported providers:

- `openai-compatible`
- `openai`
- `anthropic`

Model fields:

- `provider`
- `name`
- `baseUrl`
- `apiKey`
- `model`
- `temperature`
- `maxTokens`

Do not commit `ai-models.json`; it may contain API keys.

## Paper Projects

In the app, click New Paper. Title, authors, and journal are optional.

New paper structure:

```text
PaperForgeWorkspace/
└─ papers/
   └─ MyPaper/
      ├─ paperforge.json
      ├─ manuscript/
      │  └─ sections/
      ├─ references/
      │  ├─ papers/
      │  ├─ bib/
      │  └─ notes/
      ├─ attachments/
      │  ├─ figures/
      │  ├─ tables/
      │  ├─ raw-data/
      │  └─ supplementary/
      ├─ exports/
      │  ├─ markdown/
      │  ├─ json/
      │  ├─ word/
      │  └─ latex/
      └─ .paperforge/
         └─ history.log
```

`paperforge.json` keeps metadata and section paths:

```json
{
  "version": "1.0.0",
  "title": "Untitled Paper",
  "authors": [],
  "journal": "",
  "language": "en",
  "createdAt": "...",
  "updatedAt": "...",
  "sections": []
}
```

Implementation keeps compatibility fields such as `targetJournal` and `manuscript.sections` for current app code.

Empty manuscript is valid. PaperForge creates `manuscript/sections/` but does not create `01_abstract.md`, `02_introduction.md`, or fixed default sections.

## References And Attachments

References:

- PDFs or literature records: `references/papers/`
- BibTeX: `references/bib/references.bib`
- Notes: `references/notes/`

Attachments:

- Figures: `attachments/figures/`
- Tables: `attachments/tables/`
- Raw data: `attachments/raw-data/`
- Supplementary files: `attachments/supplementary/`

## Citation Modes

Word mode uses:

```text
[CITE: Smith2023]
```

LaTeX mode uses:

```tex
\cite{Smith2023}
```

Markdown / Pandoc mode uses:

```text
[@Smith2023]
```

PaperForge does not generate Zotero Word fields. Use Zotero Word plugin for final Word citations.

## Export

v1.0.0 supports:

- Export JSON: writes current paper config to `exports/json/paperforge.json`
- Export Markdown: writes a package under `exports/markdown/` with `paper.md`, sections, references, attachments, claims, and `export-report.json`
- Export Project Folder: writes a project folder snapshot under `exports/project-folder/`

Word and LaTeX buttons are placeholders marked Coming soon.

## UI

The app title displays `PaperForge v1.0.0`.

Layout:

- Left sidebar: Workspace, Papers, Manuscript, References, Attachments, AI Models, Export
- Center: manuscript editor / preview
- Right: AI assistant, references, citation tasks, literature, export, settings
- Settings: sidebar footer, saved immediately
- Language: English / Chinese switch

## Persistence

Desktop mode uses Tauri filesystem commands. Browser fallback uses `localStorage`.

Workspace path defaults to `PaperForgeWorkspace`.

Paper projects live under:

```text
PaperForgeWorkspace/papers/
```

AI model config lives under:

```text
PaperForgeWorkspace/.paperforge/ai-models.json
```

The repository ignores generated workspaces and local secrets.

## v1.0.0 Limits

- Direct global `paperforge` command may require `npm link` or package installation; npm scripts work from the clone.
- Tauri file picker is not implemented; workspace/project paths can be typed.
- AI calls remain mock/provider-abstraction; model config is persisted but providers are not fully wired.
- Word export and LaTeX export are Coming soon.
- PDF parsing, vector search, Zotero local API, and secure OS secret storage are not implemented.
- Project folder export is a snapshot, not a Git operation.
- Dashboard project list is app-local registry backed by Tauri/localStorage.

## Roadmap

- Secure API key storage.
- Real AI provider calls using configured models.
- Zotero local API and Better BibTeX sync.
- PDF parsing, chunking, and vector search.
- Pandoc-backed Word and LaTeX exporters.
- File picker for workspace selection.
- SQLite app registry.
