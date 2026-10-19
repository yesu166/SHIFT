# Architecture

## Implemented source flow

React UI (src/App.tsx) and the separate wheel use the central typed registry (src/lib/operations.ts) to filter actions by source extensions and gesture mode. Tauri commands in src-tauri/src/lib.rs dispatch image conversions/JPEG compression to src-tauri/src/conversion_service.rs and ZIP creation to src-tauri/src/zip_service.rs. The image engine writes temporary sibling files, decodes them again for validation, and persists under collision-safe names. The ZIP engine writes a temporary archive, reopens and reads its entries, measures byte sizes, and writes to Documents/SHIFT/Archives.

The main app shows Home, Compress, History, and Settings. The separate native wheel offers Shift for image conversion, Alt+Shift for JPEG compression/ZIP tools, and Ctrl+Shift for copy-path/output-folder actions. PDF, Office and audio/video actions remain unavailable.

## Frontend
- React + TypeScript + Vite.
- Main sections: Home, Compress, History, and Settings.
- A separate WheelOverlay component is loaded from the wheel.html Vite entry in a second native Tauri window.
- The overlay is transparent, undecorated, always-on-top, hidden until activated, and intended to be non-focusable so it doesn't interrupt an external drag.
- Browser preview is layout-only. Browser File objects are not treated as native filesystem paths.

## Gesture detection and resident behavior
- A native Windows polling thread watches for mouse-button hold, pointer movement, and gesture modifiers while Explorer/Desktop is the drag source.
- Shift opens the conversion wheel, Alt+Shift opens tools, and Ctrl+Shift opens file actions. Releasing the modifier does not close the wheel while the file drag continues.
- Escape is registered only while the wheel is visible, rather than reserved globally at all times.
- Closing the main window hides it to the notification-area tray. The tray menu can reopen the main window or quit SHIFT.
- This is one resident Tauri process with two windows, not a separate HTTP server or Windows Service Control Manager service. Sign-in autostart is not implemented.

## Wheel drop flow
- The native wheel listens for Windows drag/drop events, receives native file paths, and offers only operations compatible with the selection. A drop on a conversion action calls the Rust image engine; JPEG compression and ZIP actions use their corresponding Rust services. The gesture-end event is delayed slightly to give a native drop time to start its job.
- Pointer placement and drag/drop event coordinates use Windows/Tauri window positioning APIs. Focus preservation, OLE drag continuity, and DPI/multiple-monitor behavior still require hands-on Windows validation.

## Native layer
- Tauri 2 with narrowly scoped commands for app info, image conversion, JPEG compression, ZIP creation, opening output locations, and showing/hiding the wheel.
- Image conversion, JPEG compression and ZIP work execute on Tauri's blocking task pool.
- Rust identifies regular files/directories, applies traversal bounds, streams regular file bytes into DEFLATE entries, validates the completed ZIP, and chooses a collision-safe output path.
- No arbitrary shell execution command is exposed to the frontend.

## Boundaries and next steps
- UI components submit operation IDs and render job state; they should not contain processing logic.
- Future operations must share the registry, job lifecycle, validation and destination logic.
- External engine adapters need discrete process arguments, timeouts, bounded diagnostics and output checks.
- PDF reader/writer, audio/video codec and Office adapters are not implemented.
- ZIP extraction remains unavailable until traversal, symlink, expansion-limit and overwrite risks are tested.
- The end-to-end external Explorer drag/drop gesture is implemented in source but still needs a manual Windows GUI test; CI cannot prove focus behavior or that Explorer's OLE drag continues uninterrupted.
