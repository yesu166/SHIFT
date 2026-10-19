# Architecture

## Implemented source flow

React UI (src/App.tsx) uses the typed operation registry (src/lib/operations.ts), which calls a narrow Tauri command in src-tauri/src/lib.rs. The native command invokes the Rust ZIP service in src-tauri/src/zip_service.rs. That service creates the output folder, gathers and validates source paths, writes a temporary compressed archive, reopens and reads its entries, measures byte sizes, and persists to a unique target path. The result is returned to the UI for session history.

The main app shows only the Home, Compress, History, and Settings sections. The separate native wheel currently exposes the working ZIP action; PDF and Office actions remain unavailable.

## Frontend
- React + TypeScript + Vite.
- Main sections: Home, Compress, History, and Settings.
- A separate WheelOverlay component is loaded in a second native Tauri window via the wheel query parameter.
- The overlay is transparent, undecorated, always-on-top, hidden until activated, and intended to be non-focusable so it doesn't interrupt an external drag.
- Browser preview is layout-only. Browser File objects are not treated as native filesystem paths.

## Global activation and resident behavior
- Tauri's global-shortcut plugin activates the wheel using an F6–F12 key; F8 is the default. The selection is persisted under the app config folder.
- If the saved activation key is unavailable at startup, SHIFT tries other supported function keys.
- Escape is temporarily registered while the wheel is visible, rather than globally reserved all the time.
- Closing the main window hides it to the notification-area tray. The tray menu can reopen the main window or quit SHIFT.
- This is one resident Tauri process with two windows, not a separate HTTP service or Windows Service Control Manager service. Sign-in autostart is not implemented.

## Wheel drop flow
- The native wheel listens for Windows drag/drop events, highlights Compress to ZIP on hover, and calls the existing Rust ZIP command only after a drop on that action.
- Pointer placement and drag/drop event coordinates use Windows/Tauri window positioning APIs. Focus preservation, OLE drag continuity, and DPI/multiple-monitor behavior still require hands-on Windows validation.

## Native layer
- Tauri 2 with narrowly scoped commands for app info, ZIP creation, opening the output folder, and showing/hiding/configuring the wheel.
- ZIP work executes via Tauri's blocking task pool.
- Rust identifies regular files/directories, applies traversal bounds, streams regular file bytes into DEFLATE entries, validates the completed ZIP, and chooses a collision-safe output path.
- No arbitrary shell execution command is exposed to the frontend.

## Boundaries and next steps
- UI components submit operation IDs and render job state; they should not contain processing logic.
- Future operations must share the registry, job lifecycle, validation and destination logic.
- External engine adapters need discrete process arguments, timeouts, bounded diagnostics and output checks.
- qpdf/PDF writer and LibreOffice adapters are not implemented.
- ZIP extraction remains unavailable until traversal, symlink, expansion-limit and overwrite risks are tested.
- The end-to-end external Explorer drag/drop gesture is implemented in source but still needs a manual Windows GUI test; CI cannot prove focus behavior or that Explorer's OLE drag continues uninterrupted.
