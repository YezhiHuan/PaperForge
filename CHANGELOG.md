# Changelog

## v2.2.2

### Fixed
- `llm_body_debug_log` no longer panics on UTF-8. The masking walker used `s.replace_range(8..s.len() - 4, "****")`, which crashed with `end of range should be a character boundary` whenever the LLM payload contained Chinese, emoji, or accented Latin text. Two char-counted helpers, `safe_take_chars` and `safe_redact_middle_chars`, now drive both the masker and any future char-bounded previews. New tests cover long Chinese and emoji payloads.

### Changed
- Right panel: the `Cites`, `Claims`, and `Library` tabs are gone. The `ToolTab` type is now `info | agent | references | export`. The top bar keeps only the **References** entry. The `CitationTool`, `LiteratureTool`, and `ClaimTool` React components stay in the source so the underlying references / literature / evidence data paths can be re-wired later without a major refactor.
- The combined draft preview moved out of the export side panel. The Writing page toolbar now has a third tab (`Full Preview`) that runs alongside the existing `Edit` and `File Preview` tabs and renders the same `mergeSections` output that the exporter used to preview.
- The export tool no longer carries a `<details>` block that duplicated the full preview. Export keeps the status pill, cleaned-up path, copy-path, open-folder, and per-warning cards.

### Added
- i18n: `writing.fullPreview`, `writing.fullPreviewEmpty`, and `writing.fullPreviewHint` for English and Chinese. An empty manuscript renders the empty-state card inside the Full Preview tab.
- Three new Rust tests guard the UTF-8 helpers and the masking walker.


## v2.2.0

### Added
- Sidebar now has Writing / Files tabs. Writing mode lists manuscript sections as a clean numbered list; Files mode shows the full project tree. Choice persists across reloads.
- Text file viewer: any text-format file (`.md`, `.json`, `.bib`, `.bibtex`, `.tex`, `.txt`, `.csv`, `.tsv`, `.xml`, `.yaml`, `.yml`, `.toml`, `.log`, `.cfg`, `.ini`, `.rst`, `.html`, `.css`, `.js`, `.ts`, `.tsx`, `.jsx`) can be opened from the file tree. Markdown keeps its edit / preview toggle; other text files show a monospace view with line numbers, switch to a textarea for editing, and save back through the same `writeTextFile` path. Binary / unknown extensions are rendered as a disabled row with a "Binary file, not previewable" tooltip.
- Export result panel: status pill (success / warning / failed / running), per-mode friendly title, cleaned-up output path (Windows `\\?\` prefix stripped, backslashes normalized to forward slashes), copy-path button, "Open output folder", collapsible details with raw log lines, and per-warning cards with severity icons.

### Changed
- AI provider settings no longer expose `temperature` or `max tokens`. The Rust struct keeps both fields with `#[serde(default)]` so existing `settings.json` files continue to parse; the values are always the backend defaults (`0.3` temperature, `2000` max tokens) and are still sent to the provider on each request. No API or wire-shape change.
- Manuscript sections are no longer rendered as a regular directory node in the file tree. They live in the new Writing tab.

### Fixed
- Export result UI: the previous `proposal-card` dumped the raw `\\?\`-prefixed Windows path and a single combined log block. The new panel separates status, path, warnings, and details, with proper iconography and copy-path support.
- AI requests no longer carry `tools`, `functions`, `function_call`, `tool_calls`, `response_format`, or any Responses API field. `call_llm` now builds the OpenAI-compatible Chat Completions body through a single guarded builder, sets `tool_choice: "none"` and `parallel_tool_calls: false` to force providers (Qwen / DashScope / OpenAI-compatible gateways) to stop emitting function calls, and aborts with a clear PaperForge-side error if a forbidden key is ever reintroduced. Anthropic Messages is also guarded. The outgoing payload is logged to stderr with the API key masked and with explicit `has_tools` / `tool_choice` / `response_format` fields. Response parsers now return clear, actionable errors when the model still emits `tool_calls` / `tool_use`, is truncated by `max_tokens`, or is blocked by a content filter.


## v2.1.1

### Added
- Added standalone Settings page with Appearance, Project Defaults, AI Provider, Export, and About sections
- Added AI connection test and model fetching commands for OpenAI-compatible providers
- Added real project file tree backed by the local project filesystem
- Added Markdown file open, edit, save, and preview flow for section and non-section Markdown files
- Added app-level confirm, input, and message dialogs to replace native browser dialogs

### Changed
- Settings are no longer embedded in the right side panel
- File tree now reflects actual project files and can be refreshed manually
- AI proposal generation uses the open Markdown document as context
- Export/open-folder warnings no longer convert successful exports into failures
- Improved sidebar, file tree, and select contrast across themes

### Fixed
- Fixed Word export status handling when output is generated but post-processing/reveal has warnings
- Fixed Word/LaTeX export to keep the success toast when only the post-Pandoc citation-tasks step fails, and surfaced the real Rust error in the frontend catch (Tauri `Result::Err(String)` is no longer hidden behind a generic fallback message)
- Fixed BibTeX reference parser: the previous regex used `(?=...)` lookahead which the standard Rust `regex` crate rejects, causing `list_references` / `scan_citation_tasks` to fail on every project open. Replaced with a two-pass split that needs no look-around
- Fixed localhost native prompt/confirm/alert dialogs during rename/delete/error flows
- Fixed project entry so noncritical side-panel load errors no longer block opening a paper

## v2.1.0

- Connected Agent and AI proposal generation to OpenAI-compatible, OpenAI, and Anthropic providers
- Added LLM provider, temperature, and max token settings
- Added Pandoc-backed Word DOCX and LaTeX export
- Added Windows Pandoc auto-install attempt through `winget` when Pandoc is missing
- Enabled Word and LaTeX export buttons
- Fixed project open to reconcile editable sections from `manuscript/sections/*.md` and sync `paperforge.json`
- Fixed project entry so noncritical side-panel load errors no longer block opening a paper
- Fixed Word and LaTeX Pandoc exports to keep project-relative arguments while returning absolute output paths
- Fixed open folder actions to resolve relative paths before launching the OS file browser
- Updated app version and title to PaperForge v2.1.0

## v2.0.0

- Added Project Agent MVP with Ask, Edit, and Operate modes
- Added built-in Agent Skills for project review, export readiness, academic polish, ZH-EN translation, and figure insertion
- Added safe project filesystem boundaries for Agent file access
- Added diff-first Agent changes with Apply / Reject flow
- Added per-paper `.paperforge/agent.log`, pending Agent run records, and Apply backups
- Updated app version and title to PaperForge v2.0.0
- Ignored local `doc/` planning files

## v1.0.1

- Delete paper now removes the actual paper folder from the workspace
- Changed default workspace folder name from PaperForgeWorkspace to workspace
- Hid the terminal window in production desktop builds
- Replaced development localhost UI text with product-facing PaperForge UI
- Changed the default theme to light
- Added npm run tauri:dev script
- Improved light theme readability

## v1.0.0

- First usable product release
- Added workspace initialization
- Added command-line AI model configuration
- Added paper creation inside workspace
- Added optional paper metadata
- Added manual section management
- Added references and attachments folder structure per paper
- Added Markdown and JSON export
- Added project folder export
- Added Chinese / English UI switch
- Improved sidebar readability
- Reworked settings behavior
- Updated documentation

## v2.2.1

### Fixed
- **Agent tool-call JSON error**: `agent_chat_with_tools` Tauri command builds the OpenAI Chat Completions body with a hard-coded JSON Schema tool array, sends `tool_call.function.arguments` as a JSON string (never an object, never double-encoded), and surfaces any partial or invalid argument string as a clear PaperForge-side error. Tool result `content` is always a string and the assistant `tool_calls` message is followed by the matching `role: "tool"` reply so the gateway never sees a stale `tool_call_id` without a result.
- Provider JSON request debug log now masks the API key even inside long `system` / `user` strings.

### Added
- New `agent_chat_with_tools` Rust Tauri command that loops Chat Completions with `tools: [list_project_files, read_file, write_file, delete_file]`, executes the calls inside `ProjectFileSystem` (no escaping the project root), and returns the final assistant text plus the full tool call trace.
- New `delete_text_file` Tauri command for the Agent `delete_file` tool. Frontend `api.deleteTextFile` and `api.agentChat` bindings.
- **Copilot-style Agent UI**: chat bubbles for user / assistant / tool, a tool call list inside assistant messages that shows the tool name and the parsed arguments, an error bubble for any tool or LLM error, a clear button, a send button, an empty state with three starter chips, and a tool trace summary line. The legacy "Run Skill" panel still lives behind a collapsible "Advanced" section.
- Top bar only shows **Literature** and **References**; the duplicate Settings and New Project buttons were removed. Settings remains in the sidebar footer; New Project remains in the Dashboard hero.
- New Project auto-creates the workspace if it does not exist, so users no longer see the `paperforge init` prompt before their first paper.
- New i18n keys: `actions.literature`, `actions.references`, `actions.clearChat`, `actions.send`, `actions.thinking`, `actions.advancedSkill` (en + zh).
- `agent-subtitle`, `agent-chat-window`, `agent-bubble`, `agent-tool-list`, `agent-args`, `agent-trace-summary`, `agent-chat-input`, `topbar-actions`, `chip`, `chip-row` styles for the new agent UI and top bar.
- APP_VERSION, Cargo crate version, `tauri.conf.json` `version`, and `tauri.conf.json` window `title` all bumped to `PaperForge v2.2.1`.
- Aligned `package.json` `version` with `PaperForge v2.2.1`; it had been left at `2.2.0` in the v2.2.0 bump.
