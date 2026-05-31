# PaperForge

Current version: `1.0.1`

PaperForge is a local-first AI manuscript writing desktop app. It organizes papers as local folders and connects Markdown sections, references, attachments, exports, and replaceable AI model settings.

PaperForge does not replace Word, LaTeX, or Zotero. It is an integration and writing-assistance layer.

## Install And Run

```bash
git clone https://github.com/YezhiHuan/PaperForge.git
cd PaperForge
npm install
npm run tauri:dev
```

Raw Tauri CLI passthrough still works:

```bash
npm run tauri dev
```

Useful checks:

```bash
npm run typecheck
npm run build
npm run tauri build
```

Lint is not configured in v1.0.1.

## Workspace Init

`paperforge init` initializes the global PaperForge workspace. It does not create a paper project.

Portable npm script:

```bash
npm run paperforge:init
```

Default workspace:

```text
workspace/
├─ .paperforge/
│  ├─ workspace.json
│  ├─ ai-models.json
│  ├─ settings.json
│  └─ history.log
└─ papers/
```

`workspace/.paperforge/workspace.json` defaults:

```json
{
  "version": "1.0.1",
  "workspaceName": "workspace",
  "createdAt": "...",
  "updatedAt": "...",
  "papersDir": "papers",
  "defaultLanguage": "en"
}
```

`workspace/.paperforge/settings.json` defaults:

```json
{
  "language": "en",
  "theme": "light"
}
```

Existing user settings are respected.

## AI Model CLI

AI model config is saved to:

```text
workspace/.paperforge/ai-models.json
```

Commands:

```bash
npm run paperforge:model:add -- --provider openai-compatible --name local --base-url https://api.example.com/v1 --api-key YOUR_API_KEY --model llama3
npm run paperforge:model:list
npm run paperforge:model:set-default -- local
npm run paperforge:model:remove -- local
```

Supported providers: `openai-compatible`, `openai`, `anthropic`.

Do not commit `ai-models.json`; it may contain API keys.

## Paper Projects

In the app, click New Paper. Title, authors, and journal are optional.

New paper structure:

```text
workspace/
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

`paperforge.json` keeps metadata and section paths. Empty manuscript is valid; PaperForge creates `manuscript/sections/` but no fixed default sections.

## Delete Paper

Deleting a paper in the app deletes the actual local paper folder under `workspace/papers/`.

The confirmation dialog shows the exact folder path. Cancel means no files are deleted. If deletion fails, check file permissions or close files opened by another program, then retry.

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

Word mode: `[CITE: Smith2023]`

LaTeX mode: `\cite{Smith2023}`

Markdown / Pandoc mode: `[@Smith2023]`

PaperForge does not generate Zotero Word fields. Use Zotero Word plugin for final Word citations.

## Export

v1.0.1 supports:

- Export JSON: writes current paper config to `exports/json/paperforge.json`
- Export Markdown: writes a package under `exports/markdown/` with `paper.md`, sections, references, attachments, claims, and `export-report.json`
- Export Project Folder: writes a project folder snapshot under `exports/project-folder/`

Word and LaTeX buttons are placeholders marked Coming soon.

## UI And Theme

The app title displays `PaperForge v1.0.1`.

Default theme is light. Settings can switch light, dark, or eye-care theme. Production Windows desktop builds do not show an extra terminal window.

## Persistence

Desktop mode uses Tauri filesystem commands. Browser fallback uses `localStorage` for previews but cannot perform real paper folder deletion.

Workspace path defaults to `workspace`.

Paper projects live under:

```text
workspace/papers/
```

AI model config lives under:

```text
workspace/.paperforge/ai-models.json
```

The repository ignores generated workspaces and local secrets.

## v1.0.1 Limits

- Direct global `paperforge` command may require `npm link` or package installation; npm scripts work from the clone.
- File picker is not implemented; workspace/project paths can be typed.
- AI calls remain mock/provider-abstraction; model config is persisted but providers are not fully wired.
- Word export and LaTeX export are Coming soon.
- PDF parsing, vector search, Zotero local API, and secure OS secret storage are not implemented.
- Project folder export is a snapshot, not a Git operation.
