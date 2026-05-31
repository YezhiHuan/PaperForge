# AGENTS.md

## Project Overview

PaperForge is a local-first AI manuscript writing desktop app. It integrates local paper folders, Markdown sections, references, attachments, exports, Zotero / Better BibTeX workflows, and replaceable AI model settings.

PaperForge is not a Word, LaTeX, or Zotero replacement. It is a writing-assistance and integration layer.

## Repository Scope

This repository contains PaperForge source code only.

Generated workspaces, paper projects, PDFs, API keys, model caches, vector indexes, and private manuscript data are user data and must not be committed.

The source repository must ignore `workspace/` and `PaperForgeWorkspace/`.

## Current Release Rules

- Current product version is `1.0.0`.
- App name is `PaperForge`.
- App title must show `PaperForge v1.0.0`.
- Every source change must update `CHANGELOG.md`.
- User-facing feature changes must update `README.md`.
- `npm run tauri dev` must remain runnable after changes.
- Do not commit API keys.
- Do not hard-code user paper content into source.
- Do not default-create fixed manuscript sections.
- `paperforge init` initializes only a global workspace and AI model config.
- Single-paper directories are created only by New Paper in the app.
- Git version control is only for PaperForge source, not generated paper projects.

## Workspace Rules

`paperforge init` creates a global workspace:

```text
PaperForgeWorkspace/
├─ .paperforge/
│  ├─ workspace.json
│  ├─ ai-models.json
│  ├─ settings.json
│  └─ history.log
└─ papers/
```

It must not create `manuscript/`, `references/`, `attachments/`, or `exports/` for a paper.

`ai-models.json` may contain API keys and must stay out of Git.

## Paper Project Rules

New Paper creates one paper folder under `<workspaceRoot>/papers/`:

```text
papers/MyPaper/
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

Do not initialize Git inside paper folders in the MVP.

Empty manuscript is valid. New paper creation must create `manuscript/sections/` but no section files unless user explicitly adds sections.

Section titles and section file paths must be persisted in `paperforge.json`.

Avoid renaming section files automatically unless user explicitly requests file rename support.

## Citation Rules

Word mode:
- Use `[CITE: key]` placeholders.
- Do not generate Zotero Word fields.
- Provide citation tasks so users can insert final references with Zotero Word plugin.

LaTeX mode:
- Use `\cite{key}`.
- Use `references/bib/references.bib`.

Markdown / Pandoc mode:
- Use `[@key]` when appropriate.

## UI Rules

The UI should feel like a research writing IDE:

- Left panel: workspace and project explorer
- Center panel: manuscript editor / preview
- Right panel: AI assistant / citation tasks / literature search / export / settings
- Settings entry: sidebar footer

Keep sidebar/dropdown contrast accessible in all themes.

Settings select/toggle changes apply immediately.

Do not reintroduce the bottom activity/export status strip unless explicitly requested.

Do not add dashboard-level export controls unless explicitly requested; prefer project-internal export panel actions.

Do not include per-paper Git controls in MVP UI.

## Development Rules

- Use TypeScript for frontend code.
- Keep business logic separated from UI components.
- Prefer small modules under `src/features/` for new major features.
- Every major feature should have a clear data model.
- If implementing a mock, label it clearly and keep the interface replaceable.
- Word/LaTeX exporters should use staged export architecture, not fragile ad hoc hacks.
- Do not hardcode user-specific paths.
- Do not hardcode API keys.
- Use file-system-safe project structures.

## Commands

Use the package manager already configured in this repo.

Expected checks:

```bash
npm install
npm run typecheck
npm run build
npm run tauri dev
npm run tauri build
```

If a command is unavailable in the current environment, document that fact instead of claiming success.

## Acceptance Before Finishing

- App starts.
- TypeScript typecheck passes.
- Important UI routes render.
- Workspace init works.
- Project creation works.
- Generated paper project structure is correct.
- Generated paper projects contain no `.git`.
- Word `[CITE: key]` handling works.
- LaTeX `\cite{key}` generation works.
- README and CHANGELOG are updated.
- `.gitignore` excludes generated user data and local secrets.
