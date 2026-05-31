# AGENTS.md

## Project Overview

PaperForge is a local-first AI manuscript writing workspace.

It organizes each paper as a folder-based project and connects Markdown drafts, Word drafts, LaTeX files, Zotero / Better BibTeX references, local PDFs, vector search, and LLM-assisted writing.

PaperForge should not replace Word, LaTeX, or Zotero. It should act as an integration and writing-assistance layer.

## Repository Scope

This repository contains the PaperForge application source code.

Generated paper projects are user data and should not be committed.

Do not commit local workspaces, PDFs, API keys, model caches, vector indexes, or private manuscript data.

The source repository must ignore `workspace/`; generated paper projects live there by default.

## Core Product Principles

- One paper equals one local project folder.
- Local-first by default.
- Do not overwrite user manuscripts silently.
- Keep Word and LaTeX citation workflows separate.
- Word mode uses citation placeholders such as [CITE: Smith2023].
- LaTeX mode may directly use \cite{Smith2023}.
- Markdown / Pandoc mode may use [@Smith2023].
- AI-generated claims must be traceable to literature evidence when evidence mode is enabled.
- Prefer structured Markdown / section JSON as the internal draft representation.
- Export to Word / LaTeX should be derived from the internal draft.
- Generated paper projects should not contain Git logic in the MVP.
- Empty manuscript is a valid project state.
- Section templates are optional user choices.
- Empty title, authors, and journal are valid project states.
- Markdown package export is the primary stable export path.

## Development Rules

- Use TypeScript for frontend code.
- Use clear domain models for Project, ManuscriptSection, ReferenceItem, CitationTask, LiteratureItem, LiteratureChunk, AIProposal, ClaimRecord, and ExportJob.
- Keep business logic separated from UI components.
- Do not hardcode user-specific paths.
- Do not hardcode API keys.
- Use file-system-safe project structures.
- Avoid large monolithic components.
- Prefer small modules under src/features/.
- Every major feature should have a clear data model.
- If implementing a mock, label it clearly as mock and keep the interface replaceable.
- Do not hard-code default manuscript sections during project creation.
- Section titles and section file paths must be persisted in paperforge.project.json.
- Avoid renaming existing section files automatically unless the user explicitly requests that feature.
- Use i18n keys for UI text where practical.
- Word/LaTeX exporters should use staged export architecture, not fragile ad hoc hacks.
- Settings select/toggle changes should apply immediately.
- Do not reintroduce the bottom activity/export status strip unless explicitly requested.
- Do not add dashboard-level export controls unless explicitly requested; prefer project-internal workspace saves and export panel actions.
- Keep sidebar/dropdown contrast accessible in all themes.

## Generated Paper Project Rules

Generated paper projects are user data, not source code.

PaperForge may create project folders such as:

```text
Paper_Project/
├─ paperforge.project.json
├─ project.json
├─ manuscript/
├─ references/
├─ literature/
├─ templates/
├─ figures/
├─ data/
├─ ai/
└─ outputs/
```

Do not initialize Git inside these paper project folders in the MVP.

Manuscript sections are optional. If the user chooses an empty manuscript, create the manuscript/sections/ directory but no section files. Later section creation must update paperforge.project.json and logs/activity.json.

## Citation Rules

Word mode:
- Use [CITE: key] placeholders.
- Do not attempt to generate Zotero Word fields.
- Provide a citation task list so users can insert final references using the Zotero Word plugin.

LaTeX mode:
- Use \cite{key}.
- Use references.bib.

Markdown / Pandoc mode:
- Use [@key] when appropriate.

## UI Guidelines

The UI should feel like a research writing IDE:

- Left panel: project explorer
- Center panel: manuscript editor / preview
- Right panel: AI assistant / citation tasks / literature search
- Settings entry: sidebar footer

The UI should feel fluid and alive:
- Use subtle transitions for panel switching.
- Use hover and active states for cards, buttons, tabs, and file tree items.
- Use smooth expand/collapse animations for file trees and panels.
- Use animated loading states for AI proposals, export jobs, and literature search.
- Use gentle entrance animations for project cards and proposal cards.
- Avoid excessive or distracting animations.
- Prefer fast, subtle transitions around 120-220ms.
- Respect reduced motion if supported.

Do not include Git controls for each paper project in the MVP UI.

## Commands

Use the package manager already configured in the repository.

Expected commands:
- install dependencies
- run dev server
- run typecheck
- run lint if configured
- run tests if configured
- build app

If a command is unavailable, document it in README.md instead of inventing fake success.

## Testing and Acceptance

Before finishing a task:
- Ensure the app starts.
- Ensure TypeScript typecheck passes.
- Ensure important UI routes render.
- Ensure project creation works.
- Ensure generated paper project folder structure is correct.
- Ensure generated paper projects do not contain .git folders.
- Ensure citation placeholder handling works for Word mode.
- Ensure \cite{} generation works for LaTeX mode.
- Ensure README.md is updated.
- Ensure .gitignore excludes generated user data and local secrets.
