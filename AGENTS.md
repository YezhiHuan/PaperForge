# AGENTS.md

## Project Overview

PaperForge is a local-first AI manuscript writing desktop app. It integrates local paper folders, Markdown sections, references, attachments, exports, Zotero / Better BibTeX workflows, and replaceable AI model settings.

PaperForge is not a Word, LaTeX, or Zotero replacement. It is a writing-assistance and integration layer.

## Repository Scope

This repository contains PaperForge source code only.

Generated workspaces, paper projects, PDFs, API keys, model caches, vector indexes, and private manuscript data are user data and must not be committed.

The source repository must ignore `workspace/`.

## Current Release Rules

- Current product version is `2.1.1`.
- App name is `PaperForge`.
- App title must show `PaperForge v2.1.1`.
- Every source change must update `CHANGELOG.md`.
- User-facing feature changes must update `README.md`.
- `npm run tauri dev` and `npm run tauri:dev` must remain runnable.
- `package.json` must keep `npm run tauri:dev`.
- Do not commit API keys.
- Do not hard-code user paper content into source.
- Do not default-create fixed manuscript sections.
- `paperforge init` initializes only a global workspace and AI model config.
- `paperforge init` default workspace directory name must be `workspace`.
- Single-paper directories are created only by New Paper in the app.
- Git version control is only for PaperForge source, not generated paper projects.
- Default theme is `light`.
- Windows release builds should not show a terminal window.
- User-visible UI must not show `tauri://localhost`, `localhost`, `127.0.0.1`, `Vite`, `dev server`, or similar development-environment text.

## Workspace Rules

`paperforge init` creates a global workspace:

```text
workspace/
├─ .paperforge/
│  ├─ workspace.json
│  ├─ ai-models.json
│  ├─ settings.json
│  └─ history.log
└─ papers/
```

`workspace.json` default `workspaceName` is `workspace`.

`settings.json` default theme is `light`.

Workspace init must not create `manuscript/`, `references/`, `attachments/`, or `exports/` for a paper.

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

Deleting paper/project must delete the actual paper folder. Do not only mutate UI state, localStorage, or app registry.

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

- Left panel: workspace and project explorer
- Center panel: manuscript editor / preview
- Right panel: AI assistant / citation tasks / literature search / export
- Settings entry: sidebar footer
- Settings must remain a standalone page unless explicitly changed.
- Keep sidebar/dropdown contrast accessible in all themes.
- Settings select/toggle changes apply immediately.
- Do not use `window.alert`, `window.prompt`, or `window.confirm`.
- Use app-level modal/dialog components for confirmations, errors, and text input.
- Do not reintroduce the bottom activity/export status strip unless explicitly requested.
- Do not add dashboard-level export controls unless explicitly requested.
- Do not include per-paper Git controls in MVP UI.

## Development Rules

- Use TypeScript for frontend code.
- Keep business logic separated from UI components.
- Prefer small modules under `src/features/` for new major features.
- Every major feature should have a clear data model.
- If implementing a mock, label it clearly and keep the interface replaceable.
- Word/LaTeX exporters should use staged export architecture, not fragile ad hoc hacks.
- Do not hardcode user-specific paths.
- Do not hardcode API keys.
- AI API keys must not be logged, exported, or committed.
- File tree must reflect real project files and stay scoped to the current project root.
- Markdown files should be openable even if they are not manuscript sections.
- Export success must not be converted to failure because of post-processing or open-folder warnings.
- Use file-system-safe project structures.

## Commands

Expected checks:

```bash
npm install
npm run typecheck
npm run build
npm run tauri:dev
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
- Delete paper removes the real paper folder.
- Generated paper projects contain no `.git`.
- Word `[CITE: key]` handling works.
- LaTeX `\cite{key}` generation works.
- README and CHANGELOG are updated.
- `.gitignore` excludes generated user data and local secrets.
