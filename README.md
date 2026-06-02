# PaperForge

Current version: `2.3.0`

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

Lint is not configured in v2.3.0.

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
  "version": "2.3.0",
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

## Sidebar Modes

The left sidebar has two tabs:

- **Writing** (default) shows only the manuscript sections as a clean numbered list. Use this when you are focused on writing.
- **Files** shows the full project tree (`manuscript/`, `references/`, `attachments/`, `exports/`).

The selected tab is persisted in `AppSettings` and survives reloads.

## Text File Viewer

Any text-format file under the project root can be opened from the **Files** tab: `.md`, `.json`, `.bib`, `.bibtex`, `.tex`, `.txt`, `.csv`, `.tsv`, `.xml`, `.yaml`, `.yml`, `.toml`, `.log`, `.cfg`, `.ini`, `.rst`, `.html`, `.css`, `.js`, `.ts`, `.tsx`, `.jsx`. Markdown keeps its edit / preview toggle. Other text files open in a monospace view with line numbers and can be edited and saved through the same `writeTextFile` path used for section Markdown files. Binary / unknown extensions are rendered as a disabled row with a "Binary file, not previewable" tooltip.

## Export Result Panel

After running any export from Settings, PaperForge shows a single export result with `Export succeeded` / `Export failed`, a cleaned-up output path (the Windows `\\?\` prefix is stripped, backslashes are normalized to forward slashes), a **Copy path** button, an **Open output folder** button, collapsible logs / stderr, and per-warning cards with severity icons. The toast in the activity log no longer shows the raw `\\?\`-prefixed path.

## v2.3.0 Agent, Ref/Literature, Full Draft, And Pandoc Templates

- **Agent UI**: the Agent panel now uses a compact Codex / VSCode Chat style: simple `CHAT` title, centered empty state, safety note, tip, multiline input, Auto chip, and Send button. Export, reference, preview, claim, and skill controls are no longer shown inside Agent.
- **Right panel entries**: the third column top keeps only **Ref** / **引用** and **Literature** / **文献**. The old cite / claim / library / references duplicate entries are hidden.
- **Ref** handles citation work for the current Markdown document: search citation keys, insert formatted citations, show currently cited keys, and surface missing reference metadata.
- **Literature** handles the full literature / PDF / attachment record workflow.
- **Full Draft** / **总体** now appears as a normal virtual node in the first-column Writing and Files lists. Clicking it renders the merged manuscript preview in the center editor area.
- **Pandoc templates**: Settings now persists a Pandoc executable path, Word `.docx` reference-doc template, and LaTeX `.tex` template. Word export passes `--reference-doc`; LaTeX export passes `--template`. Blank template fields use default Pandoc arguments.
- **Export UI**: duplicate export rendering is removed. The active export controls and result live in one Settings export section.


## v2.2.2 Navigation, Full Preview And UTF-8 Logging

- **UTF-8 safe LLM payload logging**: the LLM debug log walker no longer uses ``s.replace_range(8..s.len() - 4, "****")``, which used to panic with ``end of range should be a character boundary`` whenever the body contained Chinese, emoji, or accented Latin text. Two char-counted helpers (``safe_take_chars`` and ``safe_redact_middle_chars``) now drive the masker and the preview length cap. Regression tests cover long Chinese and emoji payloads.
- **Full Preview lives in Writing**: the combined draft preview moved from the bottom-right export strip into a third tab in the Writing toolbar. The Writing page now switches between **Edit**, **File Preview** and **Full Preview**; the right panel no longer duplicates the full draft inside the Export tab.
- **Right panel and top bar trimmed**: the right-side ``Cites``, ``Claims``, and ``Library`` tabs are gone. The ``ToolTab`` type is now ``info | agent | references | export``, and the top bar keeps only the **References** entry. The ``CitationTool``, ``LiteratureTool``, and ``ClaimTool`` components stay in the source so the underlying data paths can be re-wired later.
- **Plain chat is tools-free by design**: ``build_openai_chat_body`` keeps sending ``tool_choice: "none"``, ``parallel_tool_calls: false``, and no ``tools`` key, so a "你好" prompt never carries tool calls and never trips the ``invalid function arguments json string`` error from OpenAI-compatible gateways. The ``agent_chat_with_tools`` route still ships a hard-coded JSON Schema tool array when the user explicitly opts into file actions.
- i18n: added ``writing.fullPreview``, ``writing.fullPreviewEmpty``, and ``writing.fullPreviewHint`` in English and Chinese.
## v2.2.1 Agent And Top Bar

- **Agent tool call JSON fixed**: `agent_chat_with_tools` builds the OpenAI Chat Completions body with a hard-coded JSON Schema tool array (`list_project_files`, `read_file`, `write_file`, `delete_file`) and sends `tool_call.function.arguments` as a JSON string. No more "invalid function arguments json string" errors from gateways.
- **Copilot-style agent UI**: chat bubbles for user / assistant / tool, a tool call list inside assistant messages, an error bubble for any tool or LLM failure, a clear button, an empty state with three starter chips, and a tool trace summary. The legacy "Run Skill" panel still lives behind a collapsible **Advanced** section.
- **Top bar slimmed down**: only **Literature** and **References** sit next to the project title. Settings remains in the sidebar footer; New Project remains in the Dashboard hero.
- **Auto workspace init**: clicking **New Paper Project** now calls `paperforge init` automatically if the workspace does not exist yet, so first-time users no longer see a separate initialization step.
- i18n for the new strings is in both English and Chinese.
## LLM Settings

v2.3.0 keeps Settings as a standalone page. Open Settings from the sidebar footer; it no longer occupies the manuscript editor or right panel.

PaperForge connects the Agent and AI proposal flow to real model providers.

Supported providers:

- OpenAI-compatible: `POST {baseUrl}/chat/completions`
- OpenAI: `POST https://api.openai.com/v1/chat/completions`
- Anthropic: `POST {baseUrl}/messages`

Settings include provider, base URL, API key, model, temperature, and max tokens. If the Settings API key is empty, PaperForge tries the default model in `workspace/.paperforge/ai-models.json`.

The Settings page can test the AI connection and fetch model IDs from OpenAI-compatible `/models` endpoints. Desktop LLM requests require `curl` on PATH. API keys stay in local settings or workspace model config and must not be committed. They are not included in export packages, logs, or README examples.

OpenAI-compatible chat completion support is the primary supported provider. OpenAI uses the same chat completion shape. Anthropic has basic completion support; model fetching for Anthropic is reported as unsupported in this build. Local provider settings are placeholders for OpenAI-compatible local servers.

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
         ├─ history.log
         ├─ agent.log
         ├─ agent-runs/
         └─ backups/
```

`paperforge.json` keeps metadata and section paths. Empty manuscript is valid; PaperForge creates `manuscript/sections/` but no fixed default sections.

When a paper opens, PaperForge scans `manuscript/sections/*.md`, merges readable files with `paperforge.json` section metadata, hides missing section files from the editable list, and writes the synced section list back to the manifest.

The left file tree reads the actual paper folder from disk. Markdown files in `manuscript/sections/` or other project folders can be opened, edited, saved, and previewed. Markdown files outside the manuscript manifest open as regular documents.

## Project Agent

PaperForge connects the Project Agent to real LLM providers. The right panel contains an Agent Panel with Ask, Edit, and Operate modes.

Built-in Skills:

- `ask.project-review`
- `ask.export-readiness`
- `edit.academic-polish`
- `edit.translate-zh-en`
- `operate.insert-figure`

Agent file operations are limited to the current paper project folder. The safe filesystem blocks path traversal, absolute paths, `ai-models.json`, API keys, project-external files, and dangerous write targets such as `.pdf`, `.docx`, `.exe`, `.dll`, `.msi`, and `.zip`.

Ask mode is read-only and returns a report. Edit and Operate modes prepare a diff first; the app writes files only after Apply. Before Apply, PaperForge creates a backup in `.paperforge/backups/` and records the run in `.paperforge/agent.log`.

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

v2.3.0 supports:

- Export JSON: writes current paper config to `exports/json/paperforge.json`
- Export Markdown: writes a package under `exports/markdown/` with `paper.md`, sections, references, attachments, claims, and `export-report.json`
- Export Project Folder: writes a project folder snapshot under `exports/project-folder/`
- Export Word Draft: uses Pandoc to write `exports/word/paper.docx` and keeps `[CITE: key]` placeholders
- Export LaTeX Project: uses Pandoc to write `exports/latex/paper.tex` and copies `references/bib/references.bib` when available
- Project Agent with safe Ask / Edit / Operate workflows and built-in Skills

Export jobs return absolute output paths in desktop mode. Open output folder and open project folder resolve relative workspace paths before launching the OS file browser.

Word export status is separated from folder reveal/open warnings. If the DOCX is generated, the export is shown as successful; reveal/open-folder failures are reported as warnings.

Word and LaTeX export require Pandoc. On Windows, if `pandoc --version` fails, PaperForge tries:

```powershell
winget install --id JohnMacFarlane.Pandoc -e --source winget --accept-package-agreements --accept-source-agreements --silent
```

If automatic install fails, run that command manually or install Pandoc from https://pandoc.org/installing.html, then retry export.

If PowerShell can find Pandoc but PaperForge cannot, set an explicit executable path in Settings. The same Settings export area accepts a Word reference `.docx` template and a LaTeX `.tex` template.

Word template command shape: `pandoc input.md -o output.docx --reference-doc template.docx`

LaTeX template command shape: `pandoc input.md -o output.tex --template template.tex`

PaperForge also checks common Windows install locations such as `C:\Program Files\Pandoc\pandoc.exe`, `%LOCALAPPDATA%\Pandoc\pandoc.exe`, `%LOCALAPPDATA%\Programs\Pandoc\pandoc.exe`, winget package folders, and Scoop shims.

## UI And Theme

The app title displays `PaperForge v2.3.0`.

Default theme is light. Settings can switch light, dark, or eye-care theme. Production Windows desktop builds do not show an extra terminal window.

PaperForge uses app-level dialogs for confirmations, text input, and errors. Native browser `alert`, `prompt`, and `confirm` dialogs are not used.

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

## v2.3.0 Limits

- Direct global `paperforge` command may require `npm link` or package installation; npm scripts work from the clone.
- File picker is not implemented; workspace/project paths can be typed.
- Custom Skill loading, RAG, online literature search, and Skill marketplace are not implemented.
- Word and LaTeX export depend on Pandoc availability.
- PDF parsing, vector search, Zotero local API, and secure OS secret storage are not implemented.
- Project folder export is a snapshot, not a Git operation.


