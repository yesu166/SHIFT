# Architecture

## Implemented source flow

React UI (src/App.tsx) uses the typed operation registry (src/lib/operations.ts), which calls a narrow Tauri command in src-tauri/src/lib.rs. The native command invokes the Rust ZIP service in src-tauri/src/zip_service.rs. That service creates the output folder, gathers and validates source paths, writes a temporary compressed archive, reopens and reads its entries, measures byte sizes, and persists to a unique target path. The result is returned to the UI for session history.

The UI and quick-action wheel use the same registry. Only ZIP creation is marked available; planned actions are not executable placeholders.

## Frontend
- React + TypeScript + Vite.
- Main sections: Home, Convert, Organize, Compress, Edit PDF, Protect, Batch & History, Settings.
- The in-app quick-action wheel supports Escape; no system-wide Shift-drag gesture is claimed.
- Native file selection uses the Tauri dialog API. OS file drops use the Tauri webview drag/drop API.
- Theme and reduced-motion preferences are stored locally; session job history stores metadata only.
- Browser preview is layout-only. Browser File objects are not treated as native filesystem paths.

## Native layer
- Tauri 2 with narrowly scoped commands for app info, ZIP creation, and opening the output folder.
- ZIP work executes via Tauri's blocking task pool.
- Rust identifies regular files/directories, applies traversal bounds, streams regular file bytes into DEFLATE entries, validates the completed ZIP, and chooses a collision-safe output path.
- No arbitrary shell execution command is exposed to the frontend.

## Boundaries and next steps
- UI components submit operation IDs and render job state; they should not contain processing logic.
- Future operations must share the registry, job lifecycle, validation and destination logic.
- External engine adapters need discrete process arguments, timeouts, bounded diagnostics and output checks.
- qpdf/PDF writer and LibreOffice adapters are not implemented.
- ZIP extraction remains unavailable until traversal, symlink, expansion-limit and overwrite risks are tested.
- Cross-application Shift-drag has no proof of concept. A global hotkey alone does not provide another process's active drag payload.
