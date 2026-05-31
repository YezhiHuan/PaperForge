# PaperForge

Current version: `0.3.2`

PaperForge is a local-first AI manuscript writing workspace. One paper is one local folder containing drafts, references, literature records, outputs, templates, figures, data, and AI writing history.

PaperForge is an integration layer, not a Word, LaTeX, or Zotero replacement.

## Core Features

- Project dashboard with animated project cards and create-project modal.
- English and Chinese UI language switching from Settings, persisted locally.
- Import existing project folder flow and safe project removal from the app list.
- Paper project generator with manuscript, references, literature, templates, figures, data, AI, and outputs folders.
- Optional paper title, author, and journal metadata with safe defaults.
- Editable paper title from the dashboard, IDE header, and Project Info panel.
- Optional manuscript section initialization: empty by default, template-based, or custom section names.
- Three-panel research writing IDE: explorer, manuscript editor, assistant/tools.
- Markdown section editing, preview, workspace-backed save flow, section creation/rename, and citation insertion.
- Better BibTeX paste/import parser for citekey, title, author, year, journal, DOI.
- Word citation task scanner for `[CITE: key]` placeholders.
- Literature PDF record library with mock search and replaceable embedding status.
- Mock-first AI assistant with OpenAI-compatible provider settings.
- Claims list for evidence-based writing groundwork.
- Markdown package export as the current stable export path.
- Word and LaTeX export placeholders with staged architecture for future exporters.
- Export validation warnings and output-folder opening in desktop mode.
- Persistent app activity logs stored in local app config.
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

Open the app, select **New Project**, then enter any metadata you want. Title, authors, target journal, citation style, writing mode, export mode, and sections are all optional.

If title, authors, or journal are blank, PaperForge writes safe defaults to `paperforge.project.json`:

- title: `Untitled Paper`
- authors: `[]`
- journal: `Unspecified Journal`

Manuscript sections are optional. Default is **Empty manuscript**, so PaperForge creates `manuscript/sections/` but does not force `01_abstract.md`, `02_introduction.md`, or other standard section files.

You can choose a section template or customize names:

- Empty manuscript
- Standard research paper
- Engineering simulation paper
- Review paper

Custom section names support Chinese and English. Blank section names are ignored. Duplicate generated filenames get a suffix, such as `introduction_2.md`.

Section file naming supports:

- `numbered`: `01_abstract.md`, `02_introduction.md`
- `slug only`: `abstract.md`, `introduction.md`

For Chinese or non-slug titles, PaperForge uses safe fallbacks such as `01_section.md` or `section-001.md`.

Generated structure:

```text
Paper_Project/
├─ paperforge.project.json
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

By default, project folders are created under `workspace/`. Each paper is a child folder, such as:

```text
workspace/
└─ Untitled_Paper/
   ├─ paperforge.project.json
   └─ manuscript/
      └─ sections/
         └─ 01_introduction.md
```

The repository `.gitignore` excludes `workspace/`, so generated manuscript projects stay out of source control.

After project creation, the IDE can add new sections from the project tree or empty manuscript state. Pressing **Save** writes the active Markdown section to `manuscript/sections/*.md` inside that paper's workspace folder. Section rename changes the title in `paperforge.project.json`; existing Markdown file paths are kept unless explicit file rename support is added later.

Paper title edits update the current UI, dashboard list, `paperforge.project.json`, and `updatedAt`. PaperForge does not rename the local project folder automatically.

Section structure is persisted in `paperforge.project.json`:

```json
{
  "manuscript": {
    "sectionNaming": "numbered",
    "sections": []
  }
}
```

## Import Existing Project

Use **Import Existing** on the dashboard and enter a PaperForge project folder path.

If `paperforge.project.json` or legacy `project.json` exists, PaperForge registers that project. If no manifest exists, PaperForge creates a minimal project manifest and missing MVP folders without overwriting existing manuscript files.

## Language and Settings

Settings live in the sidebar footer. Select and toggle settings apply immediately and persist locally, including theme, language, default writing mode, citation style, and export mode. Text settings are written through the same settings persistence path and are not stored in paper project manifests.

## Word Citation Workflow

Word mode uses placeholders such as `[CITE: Zhang2023]`.

PaperForge does not write Zotero Word citation fields. Users should use Zotero Word plugin later to insert final references and bibliography.

Markdown package export keeps Word placeholders when the manuscript mode uses them. Word `.docx` generation is intentionally staged for a later release.

## LaTeX Citation Workflow

LaTeX mode inserts `\cite{Zhang2023}` and writes `references.bib`. Full LaTeX project export is intentionally staged for a later release.

## Markdown / Pandoc Workflow

Markdown mode inserts `[@Zhang2023]`. The stable export is **Export Markdown Package**, which creates:

```text
outputs/
└─ paperforge-export-YYYYMMDD-HHMMSS/
   ├─ manifest.json
   ├─ manuscript.md
   ├─ sections/
   ├─ references/
   ├─ literature/
   ├─ figures/
   ├─ data/
   ├─ claims/
   └─ export-report.json
```

Missing optional files are recorded in `export-report.json` instead of failing the export.

Future Word route: Markdown package -> Pandoc -> DOCX, optionally with `reference.docx`. Zotero Word fields should still be inserted in Word through the Zotero plugin; PaperForge keeps `[CITE: key]` placeholders.

Future LaTeX route: generate `main.tex`, section `.tex` files, copy `references.bib`, and copy figures. Markdown-to-LaTeX conversion should use Pandoc or a staged converter, not fragile ad hoc string hacks.

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

API keys are not stored in `paperforge.project.json` or `project.json`. MVP stores settings in local app config or browser localStorage fallback. Later versions should use OS secure storage.

When API key is missing, AI actions return clearly labeled mock proposals.

## UI Design

The UI uses a dense research IDE layout:

- Left: project tree
- Center: section editor and preview
- Right: AI, references, citations, literature, export, settings
- Sidebar footer: Settings
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

## Version History

### 0.3.2

- Dashboard project cards now show edit/delete icon actions only.
- Removed dashboard export action.
- Clarified and tested workspace-backed Markdown saves.
- Removed placeholder MVP folder labels from the project tree.

### 0.3.1

- Added English/Chinese UI switching.
- Made paper title, author, and journal optional with persisted defaults.
- Added title editing from dashboard, IDE header, and Project Info.
- Added Markdown package export and future Word/LaTeX exporter placeholders.
- Removed the bottom status strip and improved sidebar/dropdown contrast.
- Settings now apply immediately.

### 0.3.0

- Added optional manuscript section initialization.
- Empty manuscript is default for new projects.
- Added section templates and custom section names.
- Added section naming mode: numbered or slug only.
- Added add-section and rename-section support in the IDE.
- Persisted section title/path/order/status in `paperforge.project.json`.

### 0.2.0

- Added persistent app activity logs.
- Added import/open existing project folder flow.
- Added export validation warnings and desktop output-folder opener.
- Improved citation conversion for Word, LaTeX, and Markdown/Pandoc export.
- Added visible app version.

### 0.1.0

- Initial MVP: project dashboard, folder generator, writing IDE, citations, references, mock literature search, AI proposals, claims, exports, settings, and themes.

## Roadmap

- SQLite-backed app DB.
- Secure secret storage for API keys.
- Real PDF parsing and chunking.
- LanceDB/Chroma/Qdrant/SQLite-vss backend adapter.
- Zotero local API integration.
- Pandoc execution with validation.
- Claim-to-evidence verification.
- Template manager for journal-specific exports.
