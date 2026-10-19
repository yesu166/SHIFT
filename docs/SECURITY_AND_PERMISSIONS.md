# Security and permissions

## Current design
- Core image conversion, JPEG compression and ZIP processing are local; no cloud API, required account, telemetry or backend integration is present.
- The frontend invokes named Tauri commands for image conversion/compression, ZIP creation, opening output locations and showing/hiding the wheel; no arbitrary shell bridge is exposed.
- File selection uses the Tauri dialog plugin; OS file drops use the typed Tauri webview event API.
- Image conversion, JPEG compression and ZIP work run in blocking workers.
- Selected symbolic-link roots are rejected; symbolic links inside selected folders are skipped.
- Duplicate and overlapping input roots are rejected.
- Image conversions use temporary files beside the input and decode the staged output before finalization. ZIP archives are written to a temporary file in Documents/SHIFT/Archives, reopened, and every regular entry is read before finalization.
- Final output uses a unique path and non-clobbering persistence; existing outputs are not intentionally overwritten.
- Archive-name input rejects control characters, reserved filename characters/device names, path separators, and trailing dots/spaces.
- Enumeration is bounded to 1,000 selected roots and 100,000 entries.
- Session history contains output path, sizes, entry count and timestamp, not document contents or passwords.

## Native permissions
- The Tauri capability grants core defaults and dialog open permission.
- The output folder opens through the Rust open crate.
- A native Windows poller checks the mouse button, cursor movement, modifier keys and common Explorer/Desktop window classes. Escape is registered only while the wheel is visible and is removed when it hides.
- The separate wheel window is intended to be non-focusable so it does not steal focus from Explorer during a drag; this behavior still needs a manual Windows test.
- Closing the main window hides it to the notification-area tray. The tray provides Open SHIFT and Quit SHIFT. No Windows Service Control Manager service, sign-in autostart, file association, or Explorer extension is installed.
- Explorer/Desktop drag-drop is implemented in source using native Tauri events; actual OLE continuity, hover hit testing, mode switching, DPI behavior and tray lifecycle remain manual verification gates.

## Remaining review
- Test Windows path and permission behavior, invalid image inputs, Unicode, large files, locked files, disk-full conditions, monitor/DPI changes, image round-trips and Explorer extraction.
- Review Tauri capabilities against resolved package versions.
- Add bounded/redacted diagnostics before integrating external processes.
- Complete dependency/license inventory and verify network behavior of packaged dependencies.
- Do not add archive extraction before path traversal, symlink, expansion-size, depth/count and collision defenses are tested.
