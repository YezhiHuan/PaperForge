# Changelog

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
