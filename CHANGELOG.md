# Changelog

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
