# PaperForge

PaperForge is a local-first AI manuscript writing workspace. One paper is one local folder containing drafts, references, literature records, outputs, templates, figures, data, and AI writing history.

PaperForge is an integration layer, not a Word, LaTeX, or Zotero replacement.

## Core Features

- Project dashboard with animated project cards and create-project modal.
- Project manifest export and safe project removal from the app list.
- Paper project generator with manuscript, references, literature, templates, figures, data, AI, and outputs folders.
- Three-panel research writing IDE: explorer, manuscript editor, assistant/tools.
- Markdown section editing, preview, save flow, and citation insertion.
- Better BibTeX paste/import parser for citekey, title, author, year, journal, DOI.
- Word citation task scanner for `[CITE: key]` placeholders.
- Literature PDF record library with mock search and replaceable embedding status.
- Mock-first AI assistant with OpenAI-compatible provider settings.
- Claims list for evidence-based writing groundwork.
- Word, LaTeX, and Markdown/Pandoc export panels.
- Local settings for workspace root, manuscript mode, provider config, citation style, and export mode.
- Dark, light, and eye-care color themes.

## Tech Stack

- Desktop shell: Tauri 2
- Frontend: React, TypeScript, Vite
- Styling: Tailwind CSS plus scoped app CSS
- Motion: Framer Motion and CSS transitions
- Icons: lucide-react
- Local persistence: Tauri file commands, browser fallback via localStorage
- Vector search: mock interface now, replaceable later
- LLM: OpenAI-compatible provider config with mock response fallback

## Install

```bash
npm install
```

## Development Commands

```bash
npm run dev
npm run typecheck
npm run build
npm run tauri dev
```

Lint is not configured in this MVP. Use `npm run typecheck` and `npm run build` for current verification.

## Create Paper Project

Open the app, select **New Project**, then enter title, author, target journal, manuscript mode, and optional workspace root.

Generated structure:

```text
Paper_Project/
├─ project.json
├─ manuscript/
│  ├─ sections/
│  ├─ paper.docx
│  └─ main.tex
├─ references/
├─ literature/
├─ templates/
├─ figures/
├─ data/
├─ ai/
└─ outputs/
```

PaperForge currently does not initialize Git repositories inside paper project folders.

## Word Citation Workflow

Word mode uses placeholders such as `[CITE: Zhang2023]`.

PaperForge does not write Zotero Word citation fields. Users should use Zotero Word plugin later to insert final references and bibliography.

Exports keep placeholders and generate `citation_tasks.json` so pending citation work is visible.

## LaTeX Citation Workflow

LaTeX mode inserts `\cite{Zhang2023}` and writes `references.bib`. Export generates `main.tex` using current Markdown sections and a basic template.

## Markdown / Pandoc Workflow

Markdown mode inserts `[@Zhang2023]`. Export generates `combined.md` and a Pandoc command text for later execution.

## Zotero / Better BibTeX

Recommended flow:

1. Manage real references in Zotero.
2. Export Better BibTeX as `references.bib`.
3. Paste or import BibTeX into PaperForge.
4. Insert citation syntax appropriate to manuscript mode.

## AI Provider Settings

Settings support:

- Base URL
- API key
- Model

API keys are not stored in `project.json`. MVP stores settings in local app config or browser localStorage fallback. Later versions should use OS secure storage.

When API key is missing, AI actions return clearly labeled mock proposals.

## UI Design

The UI uses a dense research IDE layout:

- Left: project tree
- Center: section editor and preview
- Right: AI, references, citations, literature, export, settings
- Bottom: compact citation queue, export state, app activity, and recent log chips
- Design/build notes can be written under `logs/`; that folder is ignored by Git.

Motion is restrained: project card entrance, file tree expand/collapse, editor/preview fade, modal overlay/content transitions, AI proposal reveal, loading skeletons, export running dots.

## No Paper Git Controls

MVP intentionally omits per-paper Git features:

- No paper project `git init`
- No paper Git status
- No paper Git commits
- No recent commit panel
- No paper version panel

## MVP Limits

- PDF handling records file metadata only; no full-text parsing.
- Literature search is mock/local search, not embedding-backed.
- AI provider abstraction exists, but missing API key uses mock responses.
- Word export keeps citation placeholders.
- Settings storage is local app config/localStorage, not secure OS keychain.
- Project removal from the dashboard removes the project from PaperForge's list; it does not delete local manuscript files.
- SQLite interface is reserved for later migration.

## Roadmap

- SQLite-backed app DB.
- Secure secret storage for API keys.
- Real PDF parsing and chunking.
- LanceDB/Chroma/Qdrant/SQLite-vss backend adapter.
- Zotero local API integration.
- Pandoc execution with validation.
- Claim-to-evidence verification.
- Template manager for journal-specific exports.
